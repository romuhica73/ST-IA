use crate::domain::model::{
    self, ModelError, ModelErrorCode, ModelKind, ModelManifest, ModelStatus,
};
use crate::model as model_manager;
use crate::pipeline::JobState;
use tauri::{AppHandle, State};

/// Async on purpose. Detection hashes the model file — 574 MB for the
/// transcription model, 3.1 GB for the translation one — and a synchronous
/// Tauri command runs on the main thread, where that hash would freeze the
/// whole app (measured: ~9s of blocked IPC for the larger model, delaying
/// even unrelated startup work). `spawn_blocking` keeps the hashing off both
/// the main thread and the async runtime's polling threads.
#[tauri::command]
pub async fn get_model_status(app: AppHandle, kind: ModelKind) -> Result<ModelStatus, String> {
    tauri::async_runtime::spawn_blocking(move || model_manager::get_or_detect_status(&app, kind))
        .await
        .map_err(|e| format!("model detection task failed: {e}"))?
}

#[tauri::command]
pub fn get_model_manifest(kind: ModelKind) -> ModelManifest {
    model::manifest(kind)
}

/// Refuses to download while a transcription is running: the two would
/// compete for disk, and for the transcription model the file is exactly
/// what the running job is reading. `install_model` itself already guards
/// against a second concurrent download of the same model.
#[tauri::command]
pub async fn install_model(
    app: AppHandle,
    job: State<'_, JobState>,
    kind: ModelKind,
) -> Result<(), ModelError> {
    if job.is_active() {
        return Err(ModelError {
            code: ModelErrorCode::WriteError,
            message: "Une transcription est en cours. Réessayez une fois qu'elle est terminée."
                .to_string(),
        });
    }
    model_manager::install_model(app, kind).await
}
