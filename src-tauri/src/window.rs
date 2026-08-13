//! Creation of ST-IA's single native window.
//!
//! One window for the whole session (see ADR-009 and ADR-011). The startup
//! intro is a layer *inside* it, rendered by the frontend, so there is no
//! second window, no handover between windows, and no geometry to keep in
//! sync — the native frame and its title bar are the same from the first
//! visible frame to the last.
//!
//! Built at runtime rather than declared in `tauri.conf.json` for one
//! reason: the boot background colour depends on the resolved theme, and it
//! has to be set *before* the window is first shown. A window declared in
//! the config is created after `setup` returns, which is already too late to
//! avoid a flash of the wrong colour.

use crate::domain::settings::{Settings, ThemePreference};
use crate::domain::shell;
use tauri::window::Color;
use tauri::{AppHandle, Theme, WebviewUrl, WebviewWindowBuilder};

pub const MAIN_LABEL: &str = "main";

/// The colour the native window and webview are filled with before the page
/// has painted anything.
///
/// These are `--bg` from `global.css` in each theme. Duplicated here rather
/// than shared because they are needed by the native layer, before any
/// stylesheet exists — and a mismatch shows up immediately as a flash at
/// startup, which is exactly what this is for.
const BOOT_BG_LIGHT: Color = Color(255, 255, 255, 255);
const BOOT_BG_DARK: Color = Color(26, 28, 32, 255);

/// Builds and shows the application window.
///
/// The window is created hidden and shown a few statements later, entirely
/// within `setup`: that is not a two-phase reveal, it is what lets the OS
/// theme be read (`theme()` needs a window) so the correct boot colour is in
/// place on the first frame the user actually sees.
pub fn create(app: &AppHandle, settings: &Settings, size: shell::Size) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, MAIN_LABEL, WebviewUrl::App("index.html".into()))
        .title("ST-IA")
        .inner_size(size.width, size.height)
        .resizable(false)
        .center()
        .visible(false)
        .build()?;

    let dark = prefers_dark(settings.theme, window.theme().ok());
    let background = if dark { BOOT_BG_DARK } else { BOOT_BG_LIGHT };
    let _ = window.set_background_color(Some(background));

    window.show()?;

    eprintln!(
        "[st-ia] window: {}x{} (theme={}, boot background={})",
        size.width,
        size.height,
        settings.theme.as_str(),
        if dark { "dark" } else { "light" }
    );
    Ok(())
}

/// Whether the window should boot dark. An explicit preference wins; only
/// "system" asks the OS, and an OS that will not answer is treated as light —
/// the same default the stylesheet falls back to.
fn prefers_dark(preference: ThemePreference, os_theme: Option<Theme>) -> bool {
    match preference {
        ThemePreference::Dark => true,
        ThemePreference::Light => false,
        ThemePreference::System => matches!(os_theme, Some(Theme::Dark)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_preference_overrides_the_operating_system() {
        // Forcing dark on a light Mac must boot dark, or the window flashes
        // white before the stylesheet corrects it.
        assert!(prefers_dark(ThemePreference::Dark, Some(Theme::Light)));
        assert!(!prefers_dark(ThemePreference::Light, Some(Theme::Dark)));
    }

    #[test]
    fn the_system_preference_follows_the_operating_system() {
        assert!(prefers_dark(ThemePreference::System, Some(Theme::Dark)));
        assert!(!prefers_dark(ThemePreference::System, Some(Theme::Light)));
    }

    #[test]
    fn an_unknown_system_theme_falls_back_to_light() {
        // Matches the stylesheet's own default, so the two cannot disagree.
        assert!(!prefers_dark(ThemePreference::System, None));
    }

    #[test]
    fn an_explicit_preference_does_not_need_the_operating_system_at_all() {
        assert!(prefers_dark(ThemePreference::Dark, None));
        assert!(!prefers_dark(ThemePreference::Light, None));
    }

    #[test]
    fn the_boot_colours_are_opaque() {
        // A translucent boot colour would let the desktop show through
        // before the page paints.
        assert_eq!(BOOT_BG_LIGHT.3, 255);
        assert_eq!(BOOT_BG_DARK.3, 255);
    }
}
