//! Pins the capability surface so a window can never silently gain access it
//! was not designed to have.
//!
//! M8 established that the frontend is treated as hostile. A second window
//! (the splash) exists, and the property that matters is that it stays
//! *nearly inert*: it displays local assets and reports exactly one event.
//!
//! Two things make that property real, and both are easy to undo by accident:
//!
//! 1. `build.rs` declares an app ACL manifest. **Without it, Tauri allows
//!    every command registered via `invoke_handler` in every window**, and
//!    the capability files below would only gate plugin permissions — the
//!    splash could call `start_transcription` and the ACL would not care.
//! 2. Each capability names its windows explicitly. Dropping the `windows`
//!    key alone grants that capability to every window.
//!
//! Neither mistake fails to compile. These tests are the guard.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

const SPLASH_LABEL: &str = "splashscreen";
const MAIN_LABEL: &str = "main";

/// The one command the splash is allowed to call.
const SPLASH_PERMISSION: &str = "allow-notify-splash-finished";

/// Commands that must never be reachable from the splash window. Not an
/// exhaustive list of the app's commands — a list of the ones whose exposure
/// would actually matter.
const PRIVILEGED_PERMISSIONS: &[&str] = &[
    "allow-start-transcription",
    "allow-cancel-transcription",
    "allow-open-output-folder",
    "allow-install-model",
    "allow-inspect-media",
    "allow-save-settings",
];

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

/// Every permission granted to `label`, across all capability files, as plain
/// identifier strings (object-form permissions contribute their identifier).
fn permissions_for(label: &str) -> HashSet<String> {
    let mut granted = HashSet::new();
    for (name, capability) in capability_files() {
        let windows = capability["windows"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: `windows` must be an array"));
        if !windows.iter().any(|w| w.as_str() == Some(label)) {
            continue;
        }
        for permission in capability["permissions"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: `permissions` must be an array"))
        {
            let identifier = permission
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    permission
                        .get("identifier")
                        .and_then(|i| i.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| panic!("{name}: permission with no identifier"));
            granted.insert(identifier);
        }
    }
    granted
}

#[test]
fn the_app_declares_an_acl_manifest_for_its_own_commands() {
    // The load-bearing one. Without `AppManifest::commands`, Tauri treats
    // every app command as allowed everywhere and the files below become
    // decorative as far as application commands are concerned.
    let build_rs = fs::read_to_string("build.rs").expect("read build.rs");
    assert!(
        build_rs.contains("AppManifest::new()") && build_rs.contains(".commands(&["),
        "build.rs must declare an app ACL manifest, or app commands are \
         callable from every window regardless of capabilities"
    );
}

#[test]
fn every_app_command_is_declared_in_the_acl_manifest() {
    // A command in `invoke_handler` but not in the manifest is callable from
    // no window at all — which fails loudly in development, the safe
    // direction. This test makes it fail at `cargo test` instead.
    let lib_rs = fs::read_to_string("src/lib.rs").expect("read src/lib.rs");
    let build_rs = fs::read_to_string("build.rs").expect("read build.rs");

    let handler = lib_rs
        .split_once("invoke_handler(tauri::generate_handler![")
        .expect("invoke_handler block")
        .1
        .split_once("])")
        .expect("end of handler list")
        .0;

    for entry in handler.split(',') {
        let Some(command) = entry.trim().rsplit("::").next() else {
            continue;
        };
        if command.is_empty() {
            continue;
        }
        assert!(
            build_rs.contains(&format!("\"{command}\"")),
            "command `{command}` is registered but missing from build.rs's ACL manifest"
        );
    }
}

#[test]
fn every_capability_is_explicitly_scoped_to_named_windows() {
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
fn the_splash_window_holds_exactly_one_permission() {
    // It reports that its animation ended. That is the entire handshake.
    let granted = permissions_for(SPLASH_LABEL);
    assert_eq!(
        granted,
        HashSet::from([SPLASH_PERMISSION.to_string()]),
        "the splash window's capability surface changed"
    );
}

#[test]
fn the_splash_window_cannot_reach_anything_privileged() {
    let granted = permissions_for(SPLASH_LABEL);
    for permission in PRIVILEGED_PERMISSIONS {
        assert!(
            !granted.contains(*permission),
            "the splash must not be able to call {permission}"
        );
    }
    // Nor any plugin: no dialog, no shell/sidecar, no opener, and not even
    // core defaults (which would bring event listening and window control).
    for prefix in ["core:", "shell:", "dialog:", "opener:", "fs:", "http:"] {
        assert!(
            !granted.iter().any(|p| p.starts_with(prefix)),
            "the splash must hold no {prefix} permission, got {granted:?}"
        );
    }
}

#[test]
fn the_main_window_still_holds_the_application_capability() {
    // The mirror image: proving the splash has almost nothing is only
    // meaningful if the main window still has what the app needs —
    // otherwise an empty capabilities/ directory would pass.
    let granted = permissions_for(MAIN_LABEL);
    for permission in PRIVILEGED_PERMISSIONS {
        assert!(
            granted.contains(*permission),
            "the main window lost {permission}"
        );
    }
    assert!(granted.contains("core:default"));
    assert!(granted.contains("shell:allow-execute"));
}

#[test]
fn the_main_window_does_not_hold_the_splash_only_permission() {
    // Not a security property, a design one: the splash signal comes from
    // the splash. Granting it to main too would let the readiness handshake
    // be short-circuited from the wrong side.
    assert!(!permissions_for(MAIN_LABEL).contains(SPLASH_PERMISSION));
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
