//! Pins the capability surface so a window can never silently gain access it
//! was not designed to have.
//!
//! M8 established that the frontend is treated as hostile, and that has not
//! changed now that the startup intro is a layer inside the application
//! rather than a second window: the webview is still the untrusted side of
//! the boundary, and the ACL is still what bounds it.
//!
//! Two things make that real, and both are easy to undo by accident:
//!
//! 1. `build.rs` declares an app ACL manifest. **Without it, Tauri allows
//!    every command registered via `invoke_handler` in every window**, and
//!    the capability files below would only gate plugin permissions.
//! 2. Each capability names its windows explicitly. Dropping the `windows`
//!    key alone grants that capability to every window, including any window
//!    added later.
//!
//! Neither mistake fails to compile. These tests are the guard.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

const MAIN_LABEL: &str = "main";

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
                "{name}: wildcard window pattern {label:?} would match any window added later"
            );
        }
    }
}

#[test]
fn the_only_window_holds_exactly_the_application_capability() {
    // Without this, an empty capabilities/ directory would satisfy every
    // other test in this file.
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

#[test]
fn only_the_single_application_window_is_granted_anything() {
    // The startup intro is a layer inside this window now. If a second
    // window is ever introduced, it must be given its permissions
    // deliberately rather than inheriting them from a broad capability.
    for (name, capability) in capability_files() {
        let windows = capability["windows"].as_array().expect("windows array");
        let labels: Vec<&str> = windows.iter().filter_map(|w| w.as_str()).collect();
        assert_eq!(
            labels,
            vec![MAIN_LABEL],
            "{name} grants permissions to a window other than `{MAIN_LABEL}`"
        );
    }
}
