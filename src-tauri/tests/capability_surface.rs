//! Pins the capability surface so a window can never silently gain access it
//! was not designed to have.
//!
//! M8 established that the frontend is treated as hostile. M9 adds a second
//! window (the splash), and the property that matters is that it stays
//! *inert*: it displays local assets and nothing else. A capability file is
//! easy to widen by accident — dropping the `windows` key alone grants the
//! capability to every window, including the splash — and nothing about that
//! mistake would fail to compile. These tests are the guard.

use std::fs;
use std::path::Path;

const SPLASH_LABEL: &str = "splashscreen";
const MAIN_LABEL: &str = "main";

fn capability_files() -> Vec<(String, serde_json::Value)> {
    let dir = Path::new("capabilities");
    let entries = fs::read_dir(dir).expect("read capabilities/");
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).expect("read capability file");
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
        files.push((path.display().to_string(), parsed));
    }
    assert!(!files.is_empty(), "no capability files found");
    files
}

#[test]
fn every_capability_is_explicitly_scoped_to_named_windows() {
    // A capability with no `windows` key applies to *all* windows. That is
    // the single most likely way for the splash to accidentally inherit the
    // sidecar and dialog permissions.
    for (name, capability) in capability_files() {
        let windows = capability
            .get("windows")
            .unwrap_or_else(|| panic!("{name}: no `windows` key — would apply to every window"));
        let windows = windows
            .as_array()
            .unwrap_or_else(|| panic!("{name}: `windows` must be an array"));
        assert!(!windows.is_empty(), "{name}: `windows` must not be empty");

        for window in windows {
            let label = window
                .as_str()
                .unwrap_or_else(|| panic!("{name}: non-string window"));
            assert!(
                !label.contains('*'),
                "{name}: wildcard window pattern {label:?} would match the splash"
            );
        }
    }
}

#[test]
fn the_splash_window_holds_no_capability_at_all() {
    // The splash is presentational: it shows local HTML/CSS and is closed
    // from Rust. It must not be able to invoke a command, listen to an
    // event, open a dialog, spawn a sidecar or reveal a path.
    for (name, capability) in capability_files() {
        let windows = capability["windows"].as_array().expect("windows array");
        for window in windows {
            assert_ne!(
                window.as_str(),
                Some(SPLASH_LABEL),
                "{name} grants permissions to the splash window; it must stay inert"
            );
        }
    }
}

#[test]
fn the_main_window_still_holds_the_application_capability() {
    // The mirror image of the test above: proving the splash has nothing is
    // only meaningful if we also prove the main window still has what the
    // app needs — otherwise an empty capabilities/ directory would pass.
    let granted = capability_files().into_iter().any(|(_, capability)| {
        capability["windows"]
            .as_array()
            .is_some_and(|w| w.iter().any(|win| win.as_str() == Some(MAIN_LABEL)))
    });
    assert!(granted, "no capability targets the main window");
}

#[test]
fn sidecar_execution_stays_limited_to_the_two_known_binaries() {
    // Regression guard for the M8 finding: shell:allow-execute must never
    // become a generic "run a program" permission.
    let mut checked = false;
    for (name, capability) in capability_files() {
        let permissions = capability["permissions"]
            .as_array()
            .expect("permissions array");
        for permission in permissions {
            if permission.get("identifier").and_then(|i| i.as_str()) != Some("shell:allow-execute")
            {
                continue;
            }
            checked = true;
            let allowed: Vec<&str> = permission["allow"]
                .as_array()
                .expect("allow array")
                .iter()
                .map(|entry| entry["name"].as_str().expect("named binary"))
                .collect();
            assert_eq!(
                allowed,
                vec!["whisper-cli", "ffmpeg"],
                "{name}: unexpected executable allow-list"
            );
            for entry in permission["allow"].as_array().unwrap() {
                assert_eq!(
                    entry["sidecar"].as_bool(),
                    Some(true),
                    "{name}: every allowed executable must be a bundled sidecar"
                );
            }
        }
    }
    assert!(checked, "shell:allow-execute permission not found");
}
