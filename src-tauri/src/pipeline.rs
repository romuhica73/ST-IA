use crate::domain::transcription::{
    build_ffmpeg_args, build_whisper_args, parse_segment_end_seconds, resolve_output_dir,
    wav_duration_secs, JobStatus, OutputFile, OutputKind, OutputSelection, TranscribingPhase,
    TranscriptionError,
};
use crate::model;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub const EVENT_NAME: &str = "transcription://event";

/// Single in-memory job slot — M2 supports exactly one transcription at a
/// time, per mission scope (no job queue, no database).
#[derive(Default)]
pub struct JobState(pub Mutex<JobStatus>);

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::Idle
    }
}

pub struct StartRequest {
    pub media_path: PathBuf,
    pub outputs: OutputSelection,
}

/// Removes the job's temporary directory when dropped, so cleanup happens
/// on every exit path (success, business error, or early return) without
/// duplicating the removal call at each return site.
struct TempJobDir(PathBuf);

impl Drop for TempJobDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn emit_status(app: &AppHandle, state: &JobState, status: JobStatus) {
    if let Ok(mut guard) = state.0.lock() {
        *guard = status.clone();
    }
    let _ = app.emit(EVENT_NAME, &status);
}

fn failed(err: TranscriptionError) -> JobStatus {
    JobStatus::Failed {
        code: err.code,
        message: err.message,
    }
}

pub async fn run(app: AppHandle, request: StartRequest) {
    let state = app.state::<JobState>();

    // Defense in depth: re-validate on the Rust side even though the
    // frontend already disables the launch button for an empty selection.
    if request.outputs.is_empty() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::no_output_selected()),
        );
        return;
    }

    if !model::model_is_installed(&app).unwrap_or(false) {
        emit_status(&app, &state, failed(TranscriptionError::model_missing()));
        return;
    }
    let model_path = match model::model_path(&app) {
        Ok(path) => path,
        Err(_) => {
            emit_status(&app, &state, failed(TranscriptionError::model_missing()));
            return;
        }
    };

    let job_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    );
    let temp_root = std::env::temp_dir().join("ST-IA").join(job_id);
    if std::fs::create_dir_all(&temp_root).is_err() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::audio_preparation_failed()),
        );
        return;
    }
    let _cleanup = TempJobDir(temp_root.clone());

    emit_status(&app, &state, JobStatus::PreparingAudio);

    let wav_path = temp_root.join("audio.wav");
    if let Err(err) = run_ffmpeg(&app, &request.media_path, &wav_path).await {
        emit_status(&app, &state, failed(err));
        return;
    }

    let total_duration_secs = std::fs::read(&wav_path)
        .ok()
        .and_then(|bytes| wav_duration_secs(&bytes));

    emit_status(
        &app,
        &state,
        JobStatus::Transcribing {
            phase: TranscribingPhase::LoadingModel,
            progress: None,
        },
    );

    let transcript_prefix = temp_root.join("transcript");
    if let Err(err) = run_whisper(
        &app,
        &state,
        &model_path,
        &wav_path,
        &transcript_prefix,
        request.outputs,
        total_duration_secs,
    )
    .await
    {
        emit_status(&app, &state, failed(err));
        return;
    }

    emit_status(&app, &state, JobStatus::WritingOutputs);

    match write_outputs(&request.media_path, &transcript_prefix, request.outputs) {
        Ok((output_dir, files)) => {
            let transcript_text = files
                .iter()
                .find(|f| f.kind == OutputKind::Txt)
                .and_then(|f| std::fs::read_to_string(&f.path).ok());
            emit_status(
                &app,
                &state,
                JobStatus::Completed {
                    output_dir: output_dir.to_string_lossy().to_string(),
                    files,
                    transcript_text,
                },
            );
        }
        Err(_) => {
            emit_status(&app, &state, failed(TranscriptionError::write_failed()));
        }
    }
}

// Sidecar names here are bare ("ffmpeg", not "binaries/ffmpeg"): tauri-build's
// copy_binaries step strips both the source subdirectory and the
// -aarch64-apple-darwin suffix, placing the binary flat next to the app
// executable (target/debug/ffmpeg in dev). `externalBin` in tauri.conf.json
// keeps the "binaries/" prefix — that one is a source-tree path, not the
// runtime lookup name.
async fn run_ffmpeg(
    app: &AppHandle,
    input: &Path,
    output_wav: &Path,
) -> Result<(), TranscriptionError> {
    let args = build_ffmpeg_args(input, output_wav);
    eprintln!("[st-ia] ffmpeg args: {args:?}");
    let sidecar = app.shell().sidecar("ffmpeg").map_err(|e| {
        eprintln!("[st-ia] ffmpeg sidecar() resolution failed: {e}");
        TranscriptionError::audio_preparation_failed()
    })?;
    let output = sidecar.args(args).output().await.map_err(|e| {
        eprintln!("[st-ia] ffmpeg spawn/output() failed: {e}");
        TranscriptionError::audio_preparation_failed()
    })?;

    eprintln!(
        "[st-ia] ffmpeg exit status: {:?}, output_wav exists: {}",
        output.status,
        output_wav.is_file()
    );

    if output.status.success() && output_wav.is_file() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    eprintln!("[st-ia] ffmpeg failed: {stderr}");
    if stderr.contains("does not contain any stream") || stderr.contains("matches no streams") {
        return Err(TranscriptionError::no_audio_track());
    }
    Err(TranscriptionError::audio_preparation_failed())
}

async fn run_whisper(
    app: &AppHandle,
    state: &JobState,
    model: &Path,
    wav: &Path,
    output_prefix: &Path,
    outputs: OutputSelection,
    total_duration_secs: Option<f32>,
) -> Result<(), TranscriptionError> {
    let args = build_whisper_args(model, wav, output_prefix, outputs);
    eprintln!("[st-ia] whisper-cli args: {args:?}");
    let sidecar = app.shell().sidecar("whisper-cli").map_err(|e| {
        eprintln!("[st-ia] whisper-cli sidecar() resolution failed: {e}");
        TranscriptionError::transcription_failed()
    })?;
    let (mut rx, _child) = sidecar.args(args).spawn().map_err(|e| {
        eprintln!("[st-ia] whisper-cli spawn() failed: {e}");
        TranscriptionError::transcription_failed()
    })?;

    let mut succeeded = false;
    let mut stderr_log = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                if let Some(end_secs) = parse_segment_end_seconds(&line) {
                    let progress = total_duration_secs
                        .filter(|total| *total > 0.0)
                        .map(|total| (end_secs / total).clamp(0.0, 1.0));
                    emit_status(
                        app,
                        state,
                        JobStatus::Transcribing {
                            phase: TranscribingPhase::Processing,
                            progress,
                        },
                    );
                }
            }
            CommandEvent::Stderr(bytes) => {
                stderr_log.push_str(&String::from_utf8_lossy(&bytes));
                stderr_log.push('\n');
            }
            CommandEvent::Terminated(payload) => {
                succeeded = payload.code == Some(0);
            }
            CommandEvent::Error(_) => {
                succeeded = false;
            }
            _ => {}
        }
    }

    if succeeded {
        Ok(())
    } else {
        eprintln!("[st-ia] whisper-cli failed:\n{stderr_log}");
        Err(TranscriptionError::transcription_failed())
    }
}

fn write_outputs(
    source_media: &Path,
    transcript_prefix: &Path,
    outputs: OutputSelection,
) -> std::io::Result<(PathBuf, Vec<OutputFile>)> {
    let output_dir = resolve_output_dir(source_media, |p| p.exists());
    std::fs::create_dir_all(&output_dir)?;

    let stem = source_media
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sous-titres");
    let mut files = Vec::new();

    if outputs.srt {
        files.push(copy_output(
            transcript_prefix,
            "srt",
            &output_dir,
            stem,
            OutputKind::Srt,
        )?);
    }
    if outputs.txt {
        files.push(copy_output(
            transcript_prefix,
            "txt",
            &output_dir,
            stem,
            OutputKind::Txt,
        )?);
    }

    Ok((output_dir, files))
}

fn copy_output(
    transcript_prefix: &Path,
    extension: &str,
    output_dir: &Path,
    stem: &str,
    kind: OutputKind,
) -> std::io::Result<OutputFile> {
    let src = transcript_prefix.with_extension(extension);
    let file_name = format!("{stem}.{extension}");
    let dest = output_dir.join(&file_name);
    std::fs::copy(&src, &dest)?;
    let size_bytes = std::fs::metadata(&dest)?.len();
    Ok(OutputFile {
        kind,
        file_name,
        path: dest.to_string_lossy().to_string(),
        size_bytes,
    })
}
