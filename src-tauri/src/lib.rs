mod cleanup;
mod commands;
mod domain;
mod migration;
mod model;
mod pipeline;

use tauri::RunEvent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(pipeline::JobState::default())
        .manage(model::ModelManagerState::default())
        .setup(|app| {
            // Recover from a previous run that was killed mid-job before the
            // window is even shown.
            cleanup::run(app.handle());
            // Adopt an existing model from the pre-release bundle identifier
            // before the model manager runs its first detection, so an
            // upgrading user is never asked to download it again.
            migration::run(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::media::inspect_media,
            commands::transcription::start_transcription,
            commands::transcription::get_transcription_status,
            commands::transcription::cancel_transcription,
            commands::transcription::open_output_folder,
            commands::model::get_model_status,
            commands::model::get_model_manifest,
            commands::model::install_model,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    // Killing the active sidecar has to happen while we still own the
    // handle. `shutdown` is idempotent and never blocks, so running it on
    // both events is safe and covers window close as well as Cmd+Q.
    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { .. } | RunEvent::Exit => pipeline::shutdown(app_handle),
        _ => {}
    });
}
