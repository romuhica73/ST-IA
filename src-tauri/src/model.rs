use crate::domain::model::{
    self, ModelError, ModelErrorCode, ModelKind, ModelStatus, ModelStatusEvent,
};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub const EVENT_NAME: &str = "model://event";

/// Session-cached status, per model. Re-hashing a 574 MB (or 3.1 GB) file on
/// every check would be wasteful; a file only changes as a result of
/// `install_model` (which updates this cache itself), so a lazily-computed,
/// session-lived cache is safe and avoids duplicating the detection logic at
/// every call site (pipeline included).
#[derive(Default)]
struct Cache {
    transcription: Option<ModelStatus>,
    translation: Option<ModelStatus>,
}

impl Cache {
    fn get(&self, kind: ModelKind) -> Option<&ModelStatus> {
        match kind {
            ModelKind::Transcription => self.transcription.as_ref(),
            ModelKind::Translation => self.translation.as_ref(),
        }
    }

    fn set(&mut self, kind: ModelKind, status: ModelStatus) {
        match kind {
            ModelKind::Transcription => self.transcription = Some(status),
            ModelKind::Translation => self.translation = Some(status),
        }
    }
}

#[derive(Default)]
pub struct ModelManagerState(Mutex<Cache>);

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("could not resolve app data directory: {e}"))?;
    Ok(app_data_dir.join("models"))
}

/// Canonical on-disk location for a model:
/// `{Application Support}/<bundle-id>/models/<file>`.
///
/// Resolved via Tauri's path API — never hardcoded to a developer or CI
/// machine's home/workspace path.
pub fn model_path(app: &AppHandle, kind: ModelKind) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join(model::spec(kind).file_name))
}

fn temp_model_path(app: &AppHandle, kind: ModelKind) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join(model::temp_file_name(kind)))
}

pub(crate) fn compute_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Reads a model file from disk and verifies it against the pinned manifest.
/// Never trusts the file's mere presence: size and SHA-256 must both match
/// exactly for `Ready`.
fn detect_status_uncached(app: &AppHandle, kind: ModelKind) -> Result<ModelStatus, String> {
    let spec = model::spec(kind);
    let path = model_path(app, kind)?;
    let metadata = match std::fs::metadata(&path) {
        Ok(m) if m.is_file() => m,
        _ => return Ok(ModelStatus::Missing),
    };

    if metadata.len() != spec.expected_size {
        eprintln!(
            "[st-ia] model detect ({}): size mismatch ({} vs expected {}) -> Corrupted",
            kind.as_str(),
            metadata.len(),
            spec.expected_size
        );
        return Ok(ModelStatus::Corrupted);
    }

    let hash = compute_sha256(&path).map_err(|e| format!("failed to hash model file: {e}"))?;
    let valid = model::is_valid(kind, metadata.len(), &hash);
    eprintln!(
        "[st-ia] model detect ({}): size={} hash={hash} valid={valid}",
        kind.as_str(),
        metadata.len()
    );
    Ok(if valid {
        ModelStatus::Ready
    } else {
        ModelStatus::Corrupted
    })
}

pub fn get_or_detect_status(app: &AppHandle, kind: ModelKind) -> Result<ModelStatus, String> {
    let state = app.state::<ModelManagerState>();
    {
        let guard = state.0.lock().map_err(|_| "model state lock poisoned")?;
        if let Some(status) = guard.get(kind) {
            return Ok(status.clone());
        }
    }
    let status = detect_status_uncached(app, kind)?;
    state
        .0
        .lock()
        .map_err(|_| "model state lock poisoned")?
        .set(kind, status.clone());
    Ok(status)
}

/// Used by the transcription pipeline: trusts the session cache rather than
/// re-hashing before every job, but the cache is only ever populated by an
/// actually-verified detection or a just-completed install.
pub fn model_is_installed(app: &AppHandle, kind: ModelKind) -> Result<bool, String> {
    Ok(matches!(
        get_or_detect_status(app, kind)?,
        ModelStatus::Ready
    ))
}

fn emit_status(app: &AppHandle, kind: ModelKind, status: ModelStatus) {
    if let Ok(mut guard) = app.state::<ModelManagerState>().0.lock() {
        guard.set(kind, status.clone());
    }
    let _ = app.emit(EVENT_NAME, &ModelStatusEvent { kind, status });
}

/// Downloads a model to a temporary file, verifies its SHA-256, and only then
/// atomically renames it to the canonical name. The final file never
/// represents a partial or unverified download — a crash or interruption at
/// any point leaves at most a `.download` temp file, never a corrupt "final"
/// model.
pub async fn install_model(app: AppHandle, kind: ModelKind) -> Result<(), ModelError> {
    {
        let state = app.state::<ModelManagerState>();
        let guard = state.0.lock().map_err(|_| ModelError {
            code: ModelErrorCode::WriteError,
            message: "État du gestionnaire de modèle inaccessible.".to_string(),
        })?;
        // Per model: downloading the translation model while the
        // transcription one is already installed is a normal flow, but two
        // downloads of the *same* model are not.
        if matches!(
            guard.get(kind),
            Some(ModelStatus::Downloading { .. }) | Some(ModelStatus::Verifying)
        ) {
            return Err(ModelError {
                code: ModelErrorCode::WriteError,
                message: "Un téléchargement est déjà en cours.".to_string(),
            });
        }
    }

    emit_status(
        &app,
        kind,
        ModelStatus::Downloading {
            downloaded_bytes: 0,
            total_bytes: None,
            progress: None,
        },
    );

    let dir = models_dir(&app).map_err(ModelError::write)?;
    std::fs::create_dir_all(&dir).map_err(|e| ModelError::write(e.to_string()))?;

    let tmp_path = temp_model_path(&app, kind).map_err(ModelError::write)?;
    // Never trust a leftover temp file from a previous interrupted attempt.
    let _ = std::fs::remove_file(&tmp_path);

    let result = download_to_temp(&app, kind, &tmp_path).await;

    match result {
        Ok(()) => {
            emit_status(&app, kind, ModelStatus::Verifying);

            let spec = model::spec(kind);
            let size = std::fs::metadata(&tmp_path).map(|m| m.len()).unwrap_or(0);
            let hash = compute_sha256(&tmp_path)
                .map_err(|e| ModelError::write(format!("failed to hash download: {e}")))?;
            eprintln!(
                "[st-ia] model verify ({}): size={size} expected={} hash={hash}",
                kind.as_str(),
                spec.expected_size
            );

            if !model::is_valid(kind, size, &hash) {
                eprintln!("[st-ia] model verify FAILED (size or hash mismatch)");
                let _ = std::fs::remove_file(&tmp_path);
                emit_status(&app, kind, ModelStatus::Corrupted);
                return Err(ModelError {
                    code: ModelErrorCode::IntegrityMismatch,
                    message: "Le fichier téléchargé est invalide.".to_string(),
                });
            }

            let final_path = model_path(&app, kind).map_err(ModelError::write)?;
            std::fs::rename(&tmp_path, &final_path)
                .map_err(|e| ModelError::write(format!("failed to finalize model file: {e}")))?;
            eprintln!("[st-ia] model promoted to {}", final_path.display());

            emit_status(&app, kind, ModelStatus::Ready);
            Ok(())
        }
        Err(err) => {
            emit_status(
                &app,
                kind,
                ModelStatus::Failed {
                    code: err.code,
                    message: err.message.clone(),
                },
            );
            Err(err)
        }
    }
}

async fn download_to_temp(
    app: &AppHandle,
    kind: ModelKind,
    tmp_path: &Path,
) -> Result<(), ModelError> {
    let spec = model::spec(kind);
    eprintln!(
        "[st-ia] model download starting ({}): {}",
        kind.as_str(),
        spec.download_url
    );
    // `https_only` is the meaningful setting here: the pinned URL is HTTPS,
    // but the default redirect policy would happily follow a 30x to plain
    // http, and Hugging Face does redirect this URL to its CDN. The SHA-256
    // check downstream already makes a swapped file unusable, so this is
    // about not silently downgrading the transport — not about integrity.
    // The redirect limit is kept (a CDN hop is expected) but bounded, and
    // the connect timeout stops a black-holed host from hanging the UI's
    // "Downloading" state forever. No timeout on the body: a multi-gigabyte
    // download on a slow link is legitimately long.
    let client = reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| ModelError::network(format!("client HTTP indisponible : {e}")))?;
    let response = client
        .get(spec.download_url)
        .send()
        .await
        .map_err(|e| ModelError::network(format!("échec de la requête réseau : {e}")))?;

    if !response.status().is_success() {
        eprintln!("[st-ia] model download HTTP status: {}", response.status());
        return Err(ModelError::network(format!(
            "le serveur a répondu avec le statut {}",
            response.status()
        )));
    }

    let total_bytes = response.content_length();
    eprintln!("[st-ia] model download content-length: {total_bytes:?}");
    let mut file = std::fs::File::create(tmp_path).map_err(|e| ModelError::write(e.to_string()))?;
    let mut downloaded: u64 = 0;
    let mut last_logged_mb: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| ModelError::network(format!("connexion interrompue : {e}")))?;
        downloaded += chunk.len() as u64;
        // Stop the moment the response exceeds the one size we will ever
        // accept. Without this the loop writes whatever the server chooses
        // to keep sending — a hostile or compromised endpoint could fill the
        // disk long before the SHA-256 check at the end ever got to reject
        // the result. Integrity was already covered; this bounds the cost of
        // finding out.
        if downloaded > spec.expected_size {
            eprintln!(
                "[st-ia] model download aborted: server sent more than the expected {} bytes",
                spec.expected_size
            );
            return Err(ModelError::network(
                "le fichier reçu dépasse la taille attendue.".to_string(),
            ));
        }
        file.write_all(&chunk)
            .map_err(|e| ModelError::write(e.to_string()))?;

        let downloaded_mb = downloaded / (1024 * 1024);
        if downloaded_mb >= last_logged_mb + 50 {
            eprintln!("[st-ia] model download progress: {downloaded_mb} MiB");
            last_logged_mb = downloaded_mb;
        }

        let progress = model::compute_progress(downloaded, total_bytes);
        emit_status(
            app,
            kind,
            ModelStatus::Downloading {
                downloaded_bytes: downloaded,
                total_bytes,
                progress,
            },
        );
    }

    Ok(())
}
