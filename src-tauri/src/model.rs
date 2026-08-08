use crate::domain::transcription::MODEL_FILE_NAME;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Canonical on-disk location for the Whisper model:
/// `{Application Support}/<bundle-id>/models/ggml-large-v3-turbo-q5_0.bin`.
///
/// Resolved via Tauri's path API — never hardcoded to a developer or CI
/// machine's home/workspace path. Model download/verification is M3's
/// responsibility; M2 only reads from this location.
pub fn model_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("could not resolve app data directory: {e}"))?;
    Ok(app_data_dir.join("models").join(MODEL_FILE_NAME))
}

pub fn model_is_installed(app: &AppHandle) -> Result<bool, String> {
    let path = model_path(app)?;
    match std::fs::metadata(&path) {
        Ok(metadata) => Ok(metadata.is_file() && metadata.len() > 0),
        Err(_) => Ok(false),
    }
}
