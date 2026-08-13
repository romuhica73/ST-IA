//! Pins the Content-Security-Policy established in M8.
//!
//! The obvious way to make new UI "just work" is to loosen the policy —
//! allow an inline `<style>`, allow a remote font, drop the policy entirely —
//! and none of those would fail a build. Everything shipped since, including
//! the boot splash now rendered inside the application, was written to fit
//! the existing policy instead: bundled stylesheets, no remote asset, no
//! inline anything. These tests keep it that way.

use std::fs;

fn csp() -> String {
    let raw = fs::read_to_string("tauri.conf.json").expect("read tauri.conf.json");
    let config: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");
    config["app"]["security"]["csp"]
        .as_str()
        .expect("a CSP must be configured — `null` disables it entirely")
        .to_string()
}

#[test]
fn csp_is_configured_and_defaults_to_self() {
    let csp = csp();
    assert!(csp.contains("default-src 'self'"), "CSP: {csp}");
}

#[test]
fn csp_forbids_inline_and_evaluated_code() {
    let csp = csp();
    for forbidden in ["unsafe-eval", "unsafe-inline"] {
        assert!(
            !csp.contains(forbidden),
            "CSP must not allow {forbidden}; found in: {csp}"
        );
    }
}

#[test]
fn csp_confines_scripts_and_styles_to_bundled_assets() {
    let csp = csp();
    assert!(csp.contains("script-src 'self'"), "CSP: {csp}");
    assert!(csp.contains("style-src 'self'"), "CSP: {csp}");
}

#[test]
fn csp_allows_no_network_destination_beyond_the_ipc_bridge() {
    // The model download runs in Rust, never in the webview, so the webview
    // needs no remote origin at all. `ipc:`/`http://ipc.localhost` are
    // Tauri's own local bridge.
    let csp = csp();
    let connect = csp
        .split(';')
        .map(str::trim)
        .find(|directive| directive.starts_with("connect-src"))
        .expect("connect-src must be set explicitly");
    assert_eq!(
        connect, "connect-src 'self' ipc: http://ipc.localhost",
        "connect-src was widened"
    );
}

#[test]
fn csp_keeps_embedding_and_navigation_locked_down() {
    let csp = csp();
    for directive in [
        "object-src 'none'",
        "frame-src 'none'",
        "child-src 'none'",
        "worker-src 'none'",
        "base-uri 'self'",
        "form-action 'none'",
        "frame-ancestors 'none'",
    ] {
        assert!(csp.contains(directive), "CSP lost `{directive}`: {csp}");
    }
}

#[test]
fn the_single_document_carries_no_inline_script_or_style() {
    // The policy above only helps if the page actually complies; a violation
    // would show up at runtime as a blank or unstyled window, which is easy
    // to miss on a fast machine.
    let html = fs::read_to_string("../index.html").expect("read index.html");
    assert!(
        !html.contains("<style"),
        "index.html must not contain an inline <style> block"
    );
    assert!(
        !html.contains("style=\""),
        "index.html must not contain a style attribute"
    );
    let script_tags = html.matches("<script").count();
    assert_eq!(
        script_tags, 1,
        "index.html must have exactly one script tag"
    );
    assert!(
        html.contains(r#"<script type="module" src="/src/main.tsx">"#),
        "index.html's script must be an external module, not inline"
    );
}

#[test]
fn the_boot_splash_references_no_remote_asset() {
    // The intro is the newest UI and the most likely place for a stray font
    // or image URL to appear.
    let css =
        fs::read_to_string("../src/features/boot/bootSplash.css").expect("read bootSplash.css");
    for scheme in ["http://", "https://", "//fonts.", "url(http"] {
        assert!(
            !css.contains(scheme),
            "boot splash assets must be fully local; found {scheme:?}"
        );
    }
}
