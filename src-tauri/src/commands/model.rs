use crate::domain::model::{self, ModelError, ModelErrorCode, ModelManifest, ModelStatus};
use crate::model as model_manager;
use crate::pipeline::JobState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_model_status(app: AppHandle) -> Result<ModelStatus, String> {
    model_manager::get_or_detect_status(&app)
}

#[tauri::command]
pub fn get_model_manifest() -> ModelManifest {
    model::manifest()
}

/// Refuses to download while a transcription is running: the two would
/// compete for disk and the model file is exactly what the running job is
/// reading. `install_model` itself already guards against a second
/// concurrent download.
#[tauri::command]
pub async fn install_model(app: AppHandle, job: State<'_, JobState>) -> Result<(), ModelError> {
    if job.is_active() {
        return Err(ModelError {
            code: ModelErrorCode::WriteError,
            message: "Une transcription est en cours. Réessayez une fois qu'elle est terminée."
                .to_string(),
        });
    }
    model_manager::install_model(app).await
}
