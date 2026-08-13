//! Splash window lifecycle (see ADR-009).
//!
//! Two real windows, not an overlay inside the main one: `main` is created
//! hidden and `splashscreen` is created visible, and the handover — show
//! main, close splash — happens exactly once, in Rust.
//!
//! The splash is a presentational window. It holds a capability for exactly
//! one command (`notify_splash_finished`) and nothing else: no transcription,
//! no model manager, no settings, no Finder, no sidecar, no filesystem, no
//! network. That isolation is enforced by the app ACL manifest declared in
//! `build.rs` — without it, Tauri would allow every app command in every
//! window regardless of what the capability files say.
//!
//! ## Handover
//!
//! Two independent things must be true before the main window appears:
//!
//! * the splash animation has played out to its end (the *visual* end, not a
//!   timer racing it — the page itself reports it);
//! * the frontend has resolved which screen it will show first.
//!
//! Whichever arrives last triggers the transition, so the animation is never
//! cut off mid-fade and the main window never appears behind a visible
//! splash. A watchdog covers the case where either signal never arrives.

use crate::domain::settings::Settings;
use crate::domain::shell;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};

pub const SPLASH_LABEL: &str = "splashscreen";
pub const MAIN_LABEL: &str = "main";

/// The splash's own timeline, mirrored in `src/splash/splash.css`. Kept here
/// so the watchdog can be reasoned about against it, and so the tests can
/// assert the two stay consistent.
pub const FADE_IN: Duration = Duration::from_millis(1000);
pub const HOLD: Duration = Duration::from_millis(3000);
pub const FADE_OUT: Duration = Duration::from_millis(2000);

/// Total time from first frame to the cut. ~6s by design: long enough to read
/// as a product intro rather than a flash.
pub fn total_duration() -> Duration {
    FADE_IN + HOLD + FADE_OUT
}

/// Upper bound on the whole splash phase. If a signal never arrives — a
/// corrupt settings file, a webview that failed to boot, a bug we have not
/// thought of — the app must still become usable rather than sit behind a
/// splash forever. Sized well clear of the animation so it never races it.
pub const WATCHDOG: Duration = Duration::from_secs(15);

#[derive(Default)]
struct Inner {
    /// The splash page reported its fade-out finished.
    animation_finished: bool,
    /// The frontend resolved which screen it will show first.
    ui_ready: bool,
    /// Set by whichever path completes the handover. Makes it idempotent.
    handed_over: bool,
}

#[derive(Default)]
pub struct SplashState(Mutex<Inner>);

/// Which of the two signals arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    AnimationFinished,
    UiReady,
}

impl SplashState {
    /// Records a signal and reports whether the handover should now run.
    /// Returns true for exactly one caller ever: the one that both completes
    /// the pair and wins the test-and-set.
    fn record(&self, signal: Signal) -> bool {
        match self.0.lock() {
            Ok(mut inner) => {
                match signal {
                    Signal::AnimationFinished => inner.animation_finished = true,
                    Signal::UiReady => inner.ui_ready = true,
                }
                if inner.handed_over || !(inner.animation_finished && inner.ui_ready) {
                    return false;
                }
                inner.handed_over = true;
                true
            }
            Err(_) => false,
        }
    }

    /// Forces the handover regardless of the signals, for the watchdog and
    /// the destroyed-window recovery. Returns true only for the first caller.
    fn force(&self) -> bool {
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
/// read by the splash itself: that is what lets the window honour a forced
/// dark theme or a forced reduced-motion setting on its very first frame
/// without holding a settings capability. They are preferences, not user
/// data — no path, no filename, no transcript ever reaches this window.
pub fn create(app: &AppHandle, settings: &Settings, size: shell::Size) -> tauri::Result<()> {
    let url = splash_url(settings);

    WebviewWindowBuilder::new(app, SPLASH_LABEL, WebviewUrl::App(url.into()))
        // A window that opens is not a window that displays. This is the only
        // screenshot-free signal that the document actually resolved and
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
            // Shown only once the document has finished loading. Created
            // visible, the window would sit blank for the ~0.5s the webview
            // takes to boot, and the fade-in would start from an already-
            // visible rectangle instead of from nothing — the first frame
            // the user sees must be the first frame of the animation.
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) {
                let _ = webview.show();
            }
        })
        .title("ST-IA")
        // Exactly the session shell geometry, so the cut to the main window
        // is a pure swap: same size, same centred position, no perceptible
        // change of shape between the two.
        .inner_size(size.width, size.height)
        .resizable(false)
        .decorations(false)
        .center()
        .always_on_top(true)
        .shadow(true)
        .focused(true)
        .visible(false)
        .build()?;

    eprintln!(
        "[st-ia] splash: window created ({}x{}, theme={}, motion={}, timeline={}ms)",
        size.width,
        size.height,
        settings.theme.as_str(),
        settings.motion.as_str(),
        total_duration().as_millis()
    );
    Ok(())
}

/// Closes the splash and shows the main window — a hard cut, by design.
///
/// The main window gets no fade of its own: the splash has already faded to
/// nothing, so the app should simply *be there*. Order matters — the splash
/// is closed first here precisely because it is already invisible by this
/// point, and closing it first avoids showing `main` underneath a window that
/// still exists.
fn reveal_main(app: &AppHandle) {
    eprintln!("[st-ia] splash: handover, cutting to main window");
    if let Some(splash) = app.get_webview_window(SPLASH_LABEL) {
        let _ = splash.close();
    }
    match app.get_webview_window(MAIN_LABEL) {
        Some(main) => {
            // Applied while the window is still hidden, so the geometry is
            // already final at the moment it appears — no resize is ever
            // visible after the cut. In the ordinary case this matches what
            // the config already created; it only does real work when a
            // small monitor forced a reduced shell.
            let size = *app.state::<shell::Size>();
            let _ = main.set_size(LogicalSize::new(size.width, size.height));
            let _ = main.center();
            let _ = main.show();
            let _ = main.set_focus();
        }
        None => eprintln!("[st-ia] splash: main window missing at handover"),
    }
}

/// Records one of the two handover signals, revealing the main window once
/// both have arrived.
pub fn signal(app: &AppHandle, signal: Signal) {
    if app.state::<SplashState>().record(signal) {
        reveal_main(app);
    }
}

/// Bounded safety net, armed at startup.
pub fn arm_watchdog(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(WATCHDOG).await;
        if app.state::<SplashState>().already_handed_over() {
            return;
        }
        eprintln!(
            "[st-ia] splash: incomplete handover after {}s, showing main window anyway",
            WATCHDOG.as_secs()
        );
        if app.state::<SplashState>().force() {
            reveal_main(&app);
        }
    });
}

/// Reported by the *main* window once it has resolved which screen it will
/// show first. Half of the handover pair.
#[tauri::command]
pub fn notify_ui_ready(app: AppHandle) {
    eprintln!("[st-ia] splash: signal ui-ready");
    signal(&app, Signal::UiReady);
}

/// Reported by the *splash* window when its fade-out animation ends. The
/// other half of the pair, and the only command that window is allowed to
/// call.
///
/// Driving the cut from the animation's real end — rather than from a Rust
/// timer started in parallel with it — is what keeps the two from drifting:
/// a timer would either cut the fade short or leave a gap after it.
#[tauri::command]
pub fn notify_splash_finished(app: AppHandle) {
    eprintln!("[st-ia] splash: signal animation-finished");
    signal(&app, Signal::AnimationFinished);
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
    if state.force() {
        reveal_main(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neither_signal_alone_hands_over() {
        // The whole point of the pair: the animation must not be cut short
        // because the frontend was fast, and the main window must not appear
        // before it has decided what to render.
        let state = SplashState::default();
        assert!(!state.record(Signal::UiReady));
        assert!(!state.already_handed_over());

        let state = SplashState::default();
        assert!(!state.record(Signal::AnimationFinished));
        assert!(!state.already_handed_over());
    }

    #[test]
    fn the_second_signal_hands_over_whichever_order_they_arrive_in() {
        for order in [
            [Signal::UiReady, Signal::AnimationFinished],
            [Signal::AnimationFinished, Signal::UiReady],
        ] {
            let state = SplashState::default();
            assert!(!state.record(order[0]));
            assert!(state.record(order[1]), "{order:?} must complete the pair");
            assert!(state.already_handed_over());
        }
    }

    #[test]
    fn handover_happens_exactly_once_however_many_signals_repeat() {
        // A page that fires animationend twice, or a re-mounting frontend,
        // must not show the main window twice.
        let state = SplashState::default();
        state.record(Signal::UiReady);
        assert!(state.record(Signal::AnimationFinished));
        assert!(!state.record(Signal::AnimationFinished));
        assert!(!state.record(Signal::UiReady));
        assert!(!state.force());
    }

    #[test]
    fn the_watchdog_can_force_a_handover_with_no_signals_at_all() {
        let state = SplashState::default();
        assert!(state.force());
        assert!(state.already_handed_over());
        assert!(!state.force());
    }

    #[test]
    fn a_forced_handover_stops_the_signals_from_running_it_again() {
        let state = SplashState::default();
        assert!(state.force());
        assert!(!state.record(Signal::UiReady));
        assert!(!state.record(Signal::AnimationFinished));
    }

    #[test]
    fn timeline_matches_the_specified_cycle() {
        assert_eq!(FADE_IN, Duration::from_millis(1000));
        assert_eq!(HOLD, Duration::from_millis(3000));
        assert_eq!(FADE_OUT, Duration::from_millis(2000));
        assert_eq!(total_duration(), Duration::from_millis(6000));
    }

    #[test]
    fn the_watchdog_never_races_the_animation() {
        // It is a failure net, not a second clock: it must leave the full
        // cycle room to complete plus slack for a slow first paint.
        assert!(WATCHDOG > total_duration() + Duration::from_secs(5));
    }

    #[test]
    fn splash_url_puts_preferences_in_the_fragment_never_the_query() {
        // Regression guard for a real defect: with a query string, Tauri's
        // asset resolver found no `splash.html?theme=…` asset and the window
        // showed a blank page while still reporting a perfectly normal URL.
        use crate::domain::settings::{LanguagePreference, MotionPreference, ThemePreference};
        let url = splash_url(&Settings {
            theme: ThemePreference::Dark,
            motion: MotionPreference::On,
            language: LanguagePreference::Fr,
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
    fn the_css_timeline_agrees_with_the_rust_one() {
        // The stylesheet owns the actual animation; these constants only
        // describe it. A silent drift would make the watchdog's margin wrong.
        let css = std::fs::read_to_string("../src/splash/splash.css").expect("read splash.css");
        let expected = format!("--cycle-duration: {}ms;", total_duration().as_millis());
        assert!(
            css.contains(&expected),
            "splash.css must declare `{expected}`"
        );
    }
}
