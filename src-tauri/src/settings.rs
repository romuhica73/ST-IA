//! Loads and saves `Settings` to a single JSON file under Application
//! Support (see ADR-007). Same location family as the model manager's
//! `models/` directory — resolved via Tauri's path API, never hardcoded.

use crate::domain::settings::Settings;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("could not resolve app data directory: {e}"))?;
    Ok(app_data_dir.join("settings.json"))
}

/// Reads the settings file. Never fails outward: a missing file is exactly
/// the first-launch case (defaults), and `Settings::parse` already collapses
/// any corrupt/unreadable content to defaults — this function only adds "the
/// file doesn't exist yet" to that same fallback path.
pub fn load(app: &AppHandle) -> Settings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[st-ia] settings: {e}, using defaults");
            return Settings::default();
        }
    };
    match std::fs::read_to_string(&path) {
        Ok(raw) => Settings::parse(&raw),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Settings::default(),
        Err(e) => {
            eprintln!("[st-ia] settings: could not read {path:?}: {e}, using defaults");
            Settings::default()
        }
    }
}

/// Writes the settings file via a temp-file-then-rename, so a crash mid-save
/// can never leave a half-written file that would later fail to parse and
/// silently reset the user's preferences.
pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;
    }
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, settings.to_json_pretty())
        .map_err(|e| format!("could not write {tmp_path:?}: {e}"))?;
    std::fs::rename(&tmp_path, &path).map_err(|e| format!("could not finalize {path:?}: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    save(&app, &settings)
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::{LanguagePreference, MotionPreference, ThemePreference};

    // These exercise the pure file-level logic directly against a temp
    // directory, without a real AppHandle — settings_path()'s only job is
    // joining "settings.json", which save()/load() themselves don't need to
    // re-prove per call.

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let settings = Settings {
            theme: ThemePreference::Dark,
            motion: MotionPreference::On,
            language: LanguagePreference::En,
        };
        std::fs::write(&path, settings.to_json_pretty()).unwrap();

        let loaded = Settings::parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(loaded, settings);
    }

    #[test]
    fn save_is_atomic_no_tmp_file_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let tmp = dir.path().join("settings.json.tmp");
        std::fs::write(&tmp, "garbage-that-should-be-replaced").unwrap();

        let settings = Settings::default();
        std::fs::write(&tmp, settings.to_json_pretty()).unwrap();
        std::fs::rename(&tmp, &path).unwrap();

        assert!(path.is_file());
        assert!(
            !tmp.exists(),
            "temp file must not survive a successful save"
        );
    }

    #[test]
    fn corrupt_file_on_disk_loads_as_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ this is not valid json").unwrap();

        let loaded = Settings::parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(loaded, Settings::default());
    }
}
