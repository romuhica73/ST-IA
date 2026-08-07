use crate::domain::media::{validate_media_path, MediaError, MediaInfo};

#[tauri::command]
pub fn inspect_media(path: String) -> Result<MediaInfo, MediaError> {
    validate_media_path(&path)
}
