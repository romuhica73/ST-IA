use crate::domain::model::{self, ModelError, ModelManifest, ModelStatus};
use crate::model as model_manager;
use tauri::AppHandle;

#[tauri::command]
pub fn get_model_status(app: AppHandle) -> Result<ModelStatus, String> {
    model_manager::get_or_detect_status(&app)
}

#[tauri::command]
pub fn get_model_manifest() -> ModelManifest {
    model::manifest()
}

#[tauri::command]
pub async fn install_model(app: AppHandle) -> Result<(), ModelError> {
    model_manager::install_model(app).await
}
