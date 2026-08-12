fn main() {
    // Opt ST-IA's own commands into the capability system.
    //
    // This is not cosmetic. Without an app ACL manifest, Tauri allows every
    // command registered via `invoke_handler` in *every* window and webview —
    // the capability files only gate plugin commands. Declaring them here is
    // what makes `capabilities/*.json` authoritative for application commands
    // too, and therefore what makes the splash window's isolation real rather
    // than merely intended: it holds a capability for exactly one command and
    // cannot reach transcription, the model manager, settings or Finder.
    //
    // Adding a command to `invoke_handler` without adding it here would leave
    // it callable from no window at all, which fails loudly in development —
    // the safe direction for this list to be wrong in.
    let app = tauri_build::AppManifest::new().commands(&[
        "inspect_media",
        "start_transcription",
        "get_transcription_status",
        "cancel_transcription",
        "open_output_folder",
        "get_model_status",
        "get_model_manifest",
        "install_model",
        "get_settings",
        "save_settings",
        "get_app_version",
        "notify_ui_ready",
        "notify_splash_finished",
    ]);

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app))
        .expect("failed to run tauri-build");
}
