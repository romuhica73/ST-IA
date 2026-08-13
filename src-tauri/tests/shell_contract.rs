//! Pins the fixed-shell contract (ADR-011).
//!
//! The previous direction sized the native window from measured DOM content.
//! It was replaced, not patched, and the risk now is reintroduction by
//! habit: a resize call added "just for this one screen" would quietly undo
//! the whole decision. These tests fail if the dynamic-fit machinery comes
//! back, and if the declared window stops matching the shell constants.

use std::fs;

fn config() -> serde_json::Value {
    let raw = fs::read_to_string("tauri.conf.json").expect("read tauri.conf.json");
    serde_json::from_str(&raw).expect("valid JSON")
}

fn main_window() -> serde_json::Value {
    config()["app"]["windows"]
        .as_array()
        .expect("windows array")
        .iter()
        .find(|w| w["label"] == "main")
        .expect("a window labelled main")
        .clone()
}

#[test]
fn the_declared_window_matches_the_shell_target() {
    // The config and the Rust constants describe the same window; a drift
    // would mean the app opens at one size and computes another.
    let window = main_window();
    assert_eq!(window["width"].as_f64(), Some(900.0));
    assert_eq!(window["height"].as_f64(), Some(640.0));
}

#[test]
fn the_window_is_hidden_and_centred_at_creation() {
    // Hidden because the splash covers startup; centred so the splash and
    // the app occupy the same place on screen.
    let window = main_window();
    assert_eq!(window["visible"].as_bool(), Some(false));
    assert_eq!(window["center"].as_bool(), Some(true));
}

#[test]
fn the_window_is_not_user_resizable() {
    // The product decision behind the fixed shell: every screen is composed
    // for one surface. Accessibility is served by the panels' own overflow,
    // not by letting the shell change shape.
    assert_eq!(main_window()["resizable"].as_bool(), Some(false));
}

#[test]
fn no_command_resizes_the_window_from_content() {
    // The dynamic-fit command is gone; nothing should register a
    // replacement. Checked against build.rs, which is the ACL manifest and
    // therefore the authoritative list of what the frontend can call.
    let build_rs = fs::read_to_string("build.rs").expect("read build.rs");
    for gone in ["fit_window", "resize_window", "set_window_size"] {
        assert!(
            !build_rs.contains(gone),
            "`{gone}` is registered again; the shell must not follow content"
        );
    }
}

#[test]
fn the_frontend_never_asks_the_window_to_resize() {
    // The other half: even without a command, a stray `set_size` through the
    // core window permission would defeat the fixed shell.
    let capability =
        fs::read_to_string("capabilities/main.json").expect("read capabilities/main.json");
    for gone in ["allow-fit-window", "core:window:allow-set-size"] {
        assert!(
            !capability.contains(gone),
            "`{gone}` is granted again; the shell must not follow content"
        );
    }
}

#[test]
fn the_removed_window_fit_module_is_really_gone() {
    // Deleted rather than left dormant "in case we go back" — a dead
    // architecture that still compiles is the one that gets revived.
    assert!(!std::path::Path::new("src/domain/window_fit.rs").exists());
    assert!(!std::path::Path::new("src/commands/window.rs").exists());
}
