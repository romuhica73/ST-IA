mod cleanup;
mod commands;
mod domain;
mod migration;
mod model;
mod pipeline;
mod settings;
mod splash;

use tauri::{Manager, RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(pipeline::JobState::default())
        .manage(model::ModelManagerState::default())
        .manage(splash::SplashState::default())
        .setup(|app| {
            // The splash window is built first so it is on screen for
            // whatever the rest of startup costs. `main` is declared hidden
            // in tauri.conf.json and is only shown at the handover.
            let settings = settings::load(app.handle());
            if let Err(e) = splash::create(app.handle(), &settings) {
                // A splash that cannot be built must never prevent the app
                // from starting: show the main window and carry on.
                eprintln!("[st-ia] splash: could not create window ({e}), showing main directly");
                splash::on_splash_destroyed(app.handle());
            } else {
                splash::arm_watchdog(app.handle().clone());
            }

            // Recover from a previous run that was killed mid-job before the
            // window is even shown.
            cleanup::run(app.handle());
            // Adopt an existing model from the pre-release bundle identifier
            // before the model manager runs its first detection, so an
            // upgrading user is never asked to download it again.
            migration::run(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            // If the splash goes away before the handover, the main window
            // would otherwise stay hidden forever and the app would look
            // like it failed to launch.
            if window.label() == splash::SPLASH_LABEL && matches!(event, WindowEvent::Destroyed) {
                splash::on_splash_destroyed(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::media::inspect_media,
            commands::transcription::start_transcription,
            commands::transcription::get_transcription_status,
            commands::transcription::cancel_transcription,
            commands::transcription::open_output_folder,
            commands::model::get_model_status,
            commands::model::get_model_manifest,
            commands::model::get_model_cards,
            commands::model::install_model,
            settings::get_settings,
            settings::save_settings,
            settings::get_app_version,
            splash::notify_ui_ready,
            splash::notify_splash_finished,
            commands::window::fit_window,
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
