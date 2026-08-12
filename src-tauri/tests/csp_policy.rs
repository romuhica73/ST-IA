//! Pins the Content-Security-Policy established in M8.
//!
//! M9 adds a second HTML document (the splash window). The obvious way to
//! make a new page "just work" is to loosen the policy — allow an inline
//! `<style>`, allow a remote font, drop the policy entirely — and none of
//! those would fail a build. The splash was instead written to fit the
//! existing policy: external stylesheet, external module script, no remote
//! asset, no inline anything. These tests keep it that way.

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
fn the_splash_document_carries_no_inline_script_or_style() {
    // The policy above only helps if the page actually complies; a violation
    // would show up at runtime as a blank splash, which is exactly the kind
    // of thing that is easy to miss on a fast machine.
    let html = fs::read_to_string("../splash.html").expect("read splash.html");
    assert!(
        !html.contains("<style"),
        "splash.html must not contain an inline <style> block"
    );
    assert!(
        !html.contains("style=\""),
        "splash.html must not contain a style attribute"
    );
    // The only <script> tag allowed is the external module entry point.
    let script_tags = html.matches("<script").count();
    assert_eq!(
        script_tags, 1,
        "splash.html must have exactly one script tag"
    );
    assert!(
        html.contains(r#"<script type="module" src="/src/splash/main.ts">"#),
        "splash.html's script must be an external module, not inline"
    );
}

#[test]
fn the_splash_document_references_no_remote_asset() {
    let html = fs::read_to_string("../splash.html").expect("read splash.html");
    let css = fs::read_to_string("../src/splash/splash.css").expect("read splash.css");
    for source in [&html, &css] {
        for scheme in ["http://", "https://", "//fonts.", "url(http"] {
            assert!(
                !source.contains(scheme),
                "splash assets must be fully local; found {scheme:?}"
            );
        }
    }
}
