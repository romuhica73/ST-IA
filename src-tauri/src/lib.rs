mod commands;
mod domain;
mod model;
mod pipeline;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(pipeline::JobState::default())
        .manage(model::ModelManagerState::default())
        .invoke_handler(tauri::generate_handler![
            commands::media::inspect_media,
            commands::transcription::start_transcription,
            commands::transcription::get_transcription_status,
            commands::transcription::open_output_folder,
            commands::model::get_model_status,
            commands::model::get_model_manifest,
            commands::model::install_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
