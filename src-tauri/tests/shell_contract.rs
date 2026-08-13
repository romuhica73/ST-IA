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

#[test]
fn no_window_is_declared_in_the_configuration() {
    // The single window is built in `window::create`, because its boot
    // background colour depends on the resolved theme and has to be set
    // before it is first shown — which a config-declared window, created
    // after `setup` returns, cannot do.
    let windows = config()["app"]["windows"]
        .as_array()
        .expect("windows array")
        .len();
    assert_eq!(windows, 0, "the window is created at runtime, not declared");
}

#[test]
fn exactly_one_window_is_created_and_it_is_the_fixed_shell() {
    // The whole same-window design rests on there being one native window.
    // A second `WebviewWindowBuilder` anywhere would reintroduce the frame
    // swap this architecture exists to remove.
    let src = fs::read_to_string("src/window.rs").expect("read src/window.rs");
    assert_eq!(
        src.matches("WebviewWindowBuilder::new").count(),
        1,
        "exactly one window may be built"
    );
    assert!(src.contains(".resizable(false)"), "the shell is fixed");
    assert!(src.contains(".center()"));

    for module in ["src/lib.rs", "src/splash.rs"] {
        let Ok(other) = fs::read_to_string(module) else {
            continue;
        };
        assert!(
            !other.contains("WebviewWindowBuilder::new"),
            "{module} builds a second window"
        );
    }
}

#[test]
fn the_window_size_comes_from_the_shell_constants() {
    // The window and the sizing policy must describe the same surface.
    let shell = fs::read_to_string("src/domain/shell.rs").expect("read shell.rs");
    assert!(shell.contains("pub const SHELL_WIDTH: f64 = 900.0;"));
    assert!(shell.contains("pub const SHELL_HEIGHT: f64 = 640.0;"));

    let window = fs::read_to_string("src/window.rs").expect("read window.rs");
    assert!(
        window.contains(".inner_size(size.width, size.height)"),
        "the window must be sized from the shell policy, not from literals"
    );
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
fn the_removed_architectures_are_really_gone() {
    // Deleted rather than left dormant "in case we go back" — a dead
    // architecture that still compiles is the one that gets revived. Covers
    // both the content-driven sizing and the separate splash window.
    for gone in [
        "src/domain/window_fit.rs",
        "src/commands/window.rs",
        "src/splash.rs",
        "capabilities/splash.json",
        "../splash.html",
    ] {
        assert!(
            !std::path::Path::new(gone).exists(),
            "{gone} still exists; it should have been removed"
        );
    }
}

#[test]
fn the_splash_handshake_commands_are_gone() {
    // The intro no longer talks to Rust at all: it is a layer the frontend
    // owns. Any surviving handshake command would be an unused privileged
    // surface.
    let build_rs = fs::read_to_string("build.rs").expect("read build.rs");
    for gone in ["notify_ui_ready", "notify_splash_finished"] {
        assert!(
            !build_rs.contains(gone),
            "`{gone}` is still registered but nothing calls it"
        );
    }
}
