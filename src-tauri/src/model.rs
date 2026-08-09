use crate::domain::model::{self, ModelError, ModelErrorCode, ModelStatus};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub const EVENT_NAME: &str = "model://event";

/// Session-cached model status. Re-hashing a ~550MB file on every check
/// would be wasteful; the file only changes as a result of `install_model`
/// (which updates this cache itself), so a lazily-computed, session-lived
/// cache is safe and avoids duplicating the detection logic at every call
/// site (pipeline included).
#[derive(Default)]
pub struct ModelManagerState(pub Mutex<Option<ModelStatus>>);

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("could not resolve app data directory: {e}"))?;
    Ok(app_data_dir.join("models"))
}

/// Canonical on-disk location for the Whisper model:
/// `{Application Support}/<bundle-id>/models/ggml-large-v3-turbo-q5_0.bin`.
///
/// Resolved via Tauri's path API — never hardcoded to a developer or CI
/// machine's home/workspace path.
pub fn model_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join(model::MODEL_FILE_NAME))
}

fn temp_model_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join(model::temp_file_name()))
}

fn compute_sha256(path: &Path) -> std::io::Result<String> {
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

/// Reads the model file from disk and verifies it against the pinned
/// manifest. Never trusts the file's mere presence: size and SHA-256 must
/// both match exactly for `Ready`.
fn detect_status_uncached(app: &AppHandle) -> Result<ModelStatus, String> {
    let path = model_path(app)?;
    let metadata = match std::fs::metadata(&path) {
        Ok(m) if m.is_file() => m,
        _ => return Ok(ModelStatus::Missing),
    };

    if metadata.len() != model::MODEL_EXPECTED_SIZE {
        eprintln!(
            "[st-ia] model detect: size mismatch ({} vs expected {}) -> Corrupted",
            metadata.len(),
            model::MODEL_EXPECTED_SIZE
        );
        return Ok(ModelStatus::Corrupted);
    }

    let hash = compute_sha256(&path).map_err(|e| format!("failed to hash model file: {e}"))?;
    let valid = model::is_valid(metadata.len(), &hash);
    eprintln!(
        "[st-ia] model detect: size={} hash={hash} valid={valid}",
        metadata.len()
    );
    Ok(if valid {
        ModelStatus::Ready
    } else {
        ModelStatus::Corrupted
    })
}

pub fn get_or_detect_status(app: &AppHandle) -> Result<ModelStatus, String> {
    let state = app.state::<ModelManagerState>();
    {
        let guard = state.0.lock().map_err(|_| "model state lock poisoned")?;
        if let Some(status) = guard.as_ref() {
            return Ok(status.clone());
        }
    }
    let status = detect_status_uncached(app)?;
    *state.0.lock().map_err(|_| "model state lock poisoned")? = Some(status.clone());
    Ok(status)
}

/// Used by the transcription pipeline (M2): trusts the session cache rather
/// than re-hashing before every job, but the cache is only ever populated
/// by an actually-verified detection or a just-completed install.
pub fn model_is_installed(app: &AppHandle) -> Result<bool, String> {
    Ok(matches!(get_or_detect_status(app)?, ModelStatus::Ready))
}

fn emit_status(app: &AppHandle, state: &ModelManagerState, status: ModelStatus) {
    if let Ok(mut guard) = state.0.lock() {
        *guard = Some(status.clone());
    }
    let _ = app.emit(EVENT_NAME, &status);
}

/// Downloads the model to a temporary file, verifies its SHA-256 while
/// streaming (one pass, no re-read), and only then atomically renames it to
/// the canonical name. The final file never represents a partial or
/// unverified download — a crash or interruption at any point leaves at
/// most a `.download` temp file, never a corrupt "final" model.
pub async fn install_model(app: AppHandle) -> Result<(), ModelError> {
    let state = app.state::<ModelManagerState>();

    {
        let guard = state.0.lock().map_err(|_| ModelError {
            code: ModelErrorCode::WriteError,
            message: "État du gestionnaire de modèle inaccessible.".to_string(),
        })?;
        if matches!(
            *guard,
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
        &state,
        ModelStatus::Downloading {
            downloaded_bytes: 0,
            total_bytes: None,
            progress: None,
        },
    );

    let dir = models_dir(&app).map_err(ModelError::write)?;
    std::fs::create_dir_all(&dir).map_err(|e| ModelError::write(e.to_string()))?;

    let tmp_path = temp_model_path(&app).map_err(ModelError::write)?;
    // Never trust a leftover temp file from a previous interrupted attempt.
    let _ = std::fs::remove_file(&tmp_path);

    let result = download_to_temp(&app, &state, &tmp_path).await;

    match result {
        Ok(()) => {
            emit_status(&app, &state, ModelStatus::Verifying);

            let size = std::fs::metadata(&tmp_path).map(|m| m.len()).unwrap_or(0);
            let hash = compute_sha256(&tmp_path)
                .map_err(|e| ModelError::write(format!("failed to hash download: {e}")))?;
            eprintln!(
                "[st-ia] model verify: size={size} expected={} hash={hash}",
                model::MODEL_EXPECTED_SIZE
            );

            if !model::is_valid(size, &hash) {
                eprintln!("[st-ia] model verify FAILED (size or hash mismatch)");
                let _ = std::fs::remove_file(&tmp_path);
                emit_status(&app, &state, ModelStatus::Corrupted);
                return Err(ModelError {
                    code: ModelErrorCode::IntegrityMismatch,
                    message: "Le fichier téléchargé est invalide.".to_string(),
                });
            }

            let final_path = model_path(&app).map_err(ModelError::write)?;
            std::fs::rename(&tmp_path, &final_path)
                .map_err(|e| ModelError::write(format!("failed to finalize model file: {e}")))?;
            eprintln!("[st-ia] model promoted to {}", final_path.display());

            emit_status(&app, &state, ModelStatus::Ready);
            Ok(())
        }
        Err(err) => {
            emit_status(
                &app,
                &state,
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
    state: &ModelManagerState,
    tmp_path: &Path,
) -> Result<(), ModelError> {
    eprintln!(
        "[st-ia] model download starting: {}",
        model::MODEL_DOWNLOAD_URL
    );
    let client = reqwest::Client::new();
    let response = client
        .get(model::MODEL_DOWNLOAD_URL)
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
        file.write_all(&chunk)
            .map_err(|e| ModelError::write(e.to_string()))?;
        downloaded += chunk.len() as u64;

        let downloaded_mb = downloaded / (1024 * 1024);
        if downloaded_mb >= last_logged_mb + 20 {
            eprintln!("[st-ia] model download progress: {downloaded_mb} MiB");
            last_logged_mb = downloaded_mb;
        }

        let progress = model::compute_progress(downloaded, total_bytes);
        emit_status(
            app,
            state,
            ModelStatus::Downloading {
                downloaded_bytes: downloaded,
                total_bytes,
                progress,
            },
        );
    }

    Ok(())
}
