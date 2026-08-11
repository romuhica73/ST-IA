//! Fails the build if `package.json`, `Cargo.toml` and `tauri.conf.json`
//! ever disagree on the app version. No silent sync at build time — a
//! mismatch is a mistake to catch and fix by hand, not paper over.

use std::fs;

fn extract_json_string_field(raw: &str, field: &str) -> String {
    let needle = format!("\"{field}\"");
    let after_key = raw
        .split_once(&needle)
        .unwrap_or_else(|| panic!("field {field:?} not found"))
        .1;
    let after_colon = after_key.split_once(':').unwrap().1.trim_start();
    let inner = after_colon
        .strip_prefix('"')
        .unwrap_or_else(|| panic!("field {field:?} is not a JSON string"));
    let end = inner.find('"').unwrap();
    inner[..end].to_string()
}

#[test]
fn package_cargo_and_tauri_config_agree_on_version() {
    // CARGO_PKG_VERSION is Cargo.toml's own version, resolved at compile
    // time — no file read needed for that one.
    let cargo_version = env!("CARGO_PKG_VERSION");

    let package_json = fs::read_to_string("../package.json").expect("read ../package.json");
    let package_version = extract_json_string_field(&package_json, "version");

    let tauri_conf = fs::read_to_string("tauri.conf.json").expect("read tauri.conf.json");
    let tauri_version = extract_json_string_field(&tauri_conf, "version");

    assert_eq!(
        package_version, cargo_version,
        "package.json version ({package_version}) != Cargo.toml version ({cargo_version})"
    );
    assert_eq!(
        tauri_version, cargo_version,
        "tauri.conf.json version ({tauri_version}) != Cargo.toml version ({cargo_version})"
    );
}
