//! Splash window lifecycle (see ADR-009).
//!
//! Two real windows, not an overlay inside the main one: `main` is created
//! hidden and `splashscreen` is created visible, and the handover — show
//! main, close splash — happens exactly once, in Rust.
//!
//! The splash is granted **no capability whatsoever** (its label appears in
//! no capability file), so it cannot invoke a command, listen to an event,
//! spawn a sidecar, touch the filesystem or reach the network. It is a
//! purely presentational window whose lifecycle is driven from here. The
//! only handshake is one command on the *main* window — which already holds
//! the app's capabilities — reporting that the UI is ready to be shown.

use crate::domain::settings::Settings;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const SPLASH_LABEL: &str = "splashscreen";
pub const MAIN_LABEL: &str = "main";

/// How long the splash stays up at minimum, so a fast startup still reads as
/// a deliberate intro rather than a flash of a window. Sized to just outrun
/// the CSS composition (bars 0–380ms, lines 300–700ms, wordmark 480–800ms).
pub const MIN_VISIBLE: Duration = Duration::from_millis(820);

/// The same floor when the user asked for reduced motion: enough to avoid a
/// visual glitch, short enough not to be a decorative delay — which is
/// exactly what reduced motion is asking us not to impose.
pub const MIN_VISIBLE_REDUCED: Duration = Duration::from_millis(160);

/// Upper bound on the whole splash phase. If the frontend never reports
/// ready — a corrupt settings file, a webview that failed to boot, a bug we
/// have not thought of — the app must still become usable rather than sit
/// behind a splash forever.
pub const WATCHDOG: Duration = Duration::from_secs(10);

struct Inner {
    shown_at: Instant,
    /// Set by whichever path gets there first (frontend ready, watchdog, or
    /// the splash being destroyed). Makes the handover idempotent.
    handed_over: bool,
}

pub struct SplashState(Mutex<Inner>);

impl Default for SplashState {
    fn default() -> Self {
        Self(Mutex::new(Inner {
            shown_at: Instant::now(),
            handed_over: false,
        }))
    }
}

impl SplashState {
    /// How much of the minimum display time is left. `None` once the floor
    /// has already elapsed — the caller then hands over immediately.
    fn remaining_hold(&self, floor: Duration) -> Option<Duration> {
        let elapsed = self.0.lock().ok()?.shown_at.elapsed();
        floor.checked_sub(elapsed).filter(|d| !d.is_zero())
    }

    /// Test-and-set: returns true only for the first caller, so two paths
    /// racing to finish the splash cannot both show the main window.
    fn claim(&self) -> bool {
        match self.0.lock() {
            Ok(mut inner) if !inner.handed_over => {
                inner.handed_over = true;
                true
            }
            _ => false,
        }
    }

    pub fn already_handed_over(&self) -> bool {
        self.0.lock().map(|inner| inner.handed_over).unwrap_or(true)
    }
}

/// The splash window's URL, carrying the two display preferences.
///
/// They go in the URL *fragment*, deliberately. Tauri's embedded asset
/// resolver matches the request path verbatim, so `splash.html?theme=light`
/// resolves to no asset at all and the window loads a blank page — observed
/// on a packaged build, not theorised. A fragment is never part of the
/// request, so the document always resolves and the values still reach the
/// page.
fn splash_url(settings: &Settings) -> String {
    format!(
        "splash.html#theme={}&motion={}",
        settings.theme.as_str(),
        settings.motion.as_str()
    )
}

/// Builds the splash window. Called first thing in `setup`, before any other
/// startup work, so it is on screen for whatever that work costs.
///
/// The stored theme/motion *preferences* travel in the URL rather than being
/// read by the splash itself: that is what lets the window keep zero
/// capabilities while still honouring a forced dark theme or a forced
/// reduced-motion setting on the very first frame. They are preferences, not
/// user data — no path, no filename, no transcript ever reaches this window.
pub fn create(app: &AppHandle, settings: &Settings) -> tauri::Result<()> {
    let url = splash_url(settings);

    WebviewWindowBuilder::new(app, SPLASH_LABEL, WebviewUrl::App(url.into()))
        // A window that opens is not a window that displays. The splash holds
        // no capability, so it cannot report for itself; this is the only
        // screenshot-free signal that its document actually resolved and
        // finished loading, rather than 404-ing to a blank page — the exact
        // failure a mistyped asset URL produces, silently, while `url()` still
        // reports something perfectly normal.
        .on_page_load(|webview, payload| {
            eprintln!(
                "[st-ia] splash: page {:?} <{}> (window {})",
                payload.event(),
                payload.url(),
                webview.label()
            );
        })
        .title("ST-IA")
        .inner_size(400.0, 260.0)
        .resizable(false)
        .decorations(false)
        .center()
        .always_on_top(true)
        .shadow(true)
        .focused(true)
        .build()?;

    eprintln!(
        "[st-ia] splash: window created (theme={}, motion={})",
        settings.theme.as_str(),
        settings.motion.as_str()
    );
    Ok(())
}

/// Shows the main window and closes the splash, at most once.
///
/// Ordering is deliberate: the main window is shown *before* the splash is
/// closed, so there is never a frame with no ST-IA window on screen (which
/// reads as a flicker, and on macOS briefly bounces focus to another app).
fn reveal_main(app: &AppHandle) {
    eprintln!("[st-ia] splash: handover, showing main window");
    if let Some(main) = app.get_webview_window(MAIN_LABEL) {
        let _ = main.show();
        let _ = main.set_focus();
    } else {
        eprintln!("[st-ia] splash: main window missing at handover");
    }
    if let Some(splash) = app.get_webview_window(SPLASH_LABEL) {
        let _ = splash.close();
    }
}

/// Completes the splash phase once `floor` has elapsed since the window was
/// created. Non-blocking throughout: the wait is an async timer on Tauri's
/// runtime, never a `thread::sleep` — nothing about the splash is allowed to
/// block a thread that could be doing real startup work.
pub async fn hand_over(app: AppHandle, floor: Duration) {
    let remaining = {
        let state = app.state::<SplashState>();
        if state.already_handed_over() {
            return;
        }
        state.remaining_hold(floor)
    };

    if let Some(wait) = remaining {
        tokio::time::sleep(wait).await;
    }

    if !app.state::<SplashState>().claim() {
        return;
    }
    reveal_main(&app);
}

/// Bounded safety net, armed at startup. Costs one idle timer and settles
/// the app into a usable state if the readiness signal never arrives.
pub fn arm_watchdog(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(WATCHDOG).await;
        if app.state::<SplashState>().already_handed_over() {
            return;
        }
        eprintln!(
            "[st-ia] splash: no readiness signal after {}s, showing main window anyway",
            WATCHDOG.as_secs()
        );
        // Floor already long past; this hands over on the next poll.
        hand_over(app, Duration::ZERO).await;
    });
}

/// The single handshake, callable only from the main window.
///
/// `reduced_motion` is the value the main window already resolved for itself
/// (stored preference folded against the OS setting). Passing it here is
/// what lets the splash's minimum display time honour reduced motion without
/// giving the splash window any way to read settings — Rust cannot observe
/// the OS's `prefers-reduced-motion` on its own, and the frontend can.
#[tauri::command]
pub async fn notify_ui_ready(app: AppHandle, reduced_motion: bool) {
    let floor = if reduced_motion {
        MIN_VISIBLE_REDUCED
    } else {
        MIN_VISIBLE
    };
    hand_over(app, floor).await;
}

/// Recovers if the splash window disappears before the handover — a forced
/// close, or a webview that died. Without this the main window would stay
/// hidden and the app would look like it failed to start.
pub fn on_splash_destroyed(app: &AppHandle) {
    let state = app.state::<SplashState>();
    if state.already_handed_over() {
        return;
    }
    eprintln!("[st-ia] splash: window destroyed before handover, showing main window");
    if state.claim() {
        reveal_main(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handover_is_claimed_exactly_once() {
        // The watchdog, the readiness command and the destroyed-window
        // recovery can all fire; only one may show the main window.
        let state = SplashState::default();
        assert!(state.claim());
        assert!(!state.claim());
        assert!(!state.claim());
    }

    #[test]
    fn a_fresh_splash_has_not_handed_over() {
        assert!(!SplashState::default().already_handed_over());
    }

    #[test]
    fn hold_is_remaining_time_below_the_floor() {
        let state = SplashState::default();
        let remaining = state
            .remaining_hold(Duration::from_secs(30))
            .expect("a 30s floor cannot already have elapsed");
        assert!(remaining <= Duration::from_secs(30));
        assert!(!remaining.is_zero());
    }

    #[test]
    fn hold_is_none_once_the_floor_has_elapsed() {
        // The zero floor is the watchdog's path: hand over on the next poll
        // rather than waiting again.
        let state = SplashState::default();
        assert_eq!(state.remaining_hold(Duration::ZERO), None);
    }

    #[test]
    fn reduced_motion_floor_is_much_shorter_than_the_animated_one() {
        // Reduced motion must not be served a decorative delay.
        assert!(MIN_VISIBLE_REDUCED < MIN_VISIBLE);
        assert!(MIN_VISIBLE_REDUCED <= Duration::from_millis(200));
    }

    #[test]
    fn animated_floor_stays_inside_the_targeted_ux_window() {
        // 600–900ms: long enough to read as intentional, short enough not to
        // be felt as a wait.
        assert!(MIN_VISIBLE >= Duration::from_millis(600));
        assert!(MIN_VISIBLE <= Duration::from_millis(900));
    }

    #[test]
    fn splash_url_puts_preferences_in_the_fragment_never_the_query() {
        // Regression guard for a real defect: with a query string, Tauri's
        // asset resolver found no `splash.html?theme=…` asset and the window
        // showed a blank page while still reporting a perfectly normal URL.
        use crate::domain::settings::{MotionPreference, ThemePreference};
        let url = splash_url(&Settings {
            theme: ThemePreference::Dark,
            motion: MotionPreference::On,
            language: crate::domain::settings::LanguagePreference::Fr,
        });

        assert_eq!(url, "splash.html#theme=dark&motion=on");
        assert!(!url.contains('?'), "a query string does not resolve: {url}");
        let (path, _) = url
            .split_once('#')
            .expect("preferences travel in the fragment");
        assert_eq!(path, "splash.html", "the asset path must stay bare");
    }

    #[test]
    fn splash_url_carries_the_default_preferences_too() {
        let url = splash_url(&Settings::default());
        assert_eq!(url, "splash.html#theme=system&motion=system");
    }

    #[test]
    fn watchdog_is_bounded_and_longer_than_any_floor() {
        assert!(WATCHDOG > MIN_VISIBLE);
        assert!(WATCHDOG <= Duration::from_secs(30));
    }
}
