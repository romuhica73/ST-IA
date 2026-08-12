import { invoke } from "@tauri-apps/api/core";
import { readPreference, resolveSplashTheme, resolveSplashMotion } from "./resolve";
import "./splash.css";

/** Splash entry point.
 *
 * Deliberately the smallest program in the codebase: it resolves two
 * presentation attributes, then reports once when the animation has played
 * out. It imports nothing from the main app and registers no other listener.
 *
 * The window holds a capability for exactly one command — the one called
 * below — enforced by the app ACL manifest in `src-tauri/build.rs`. Anything
 * else invoked from here is rejected by Tauri, by construction.
 *
 * Attributes are set before first paint (this module is the only script and
 * runs against an already-parsed body), so there is no flash of the wrong
 * theme or of motion that should have been suppressed. */

// Rust puts the preferences in the fragment; see readPreference for why that
// is load-bearing rather than a style choice.
const params = window.location.hash || window.location.search;

document.documentElement.dataset.theme = resolveSplashTheme(
  readPreference(params, "theme"),
  window.matchMedia("(prefers-color-scheme: dark)").matches,
);

document.documentElement.dataset.motion = resolveSplashMotion(
  readPreference(params, "motion"),
  window.matchMedia("(prefers-reduced-motion: reduce)").matches,
);

/** Reports the end of the fade-out, which is what triggers the cut to the
 * main window.
 *
 * Driven by the animation's real end rather than by a timer started
 * alongside it: a timer would drift against the compositor and either clip
 * the fade or leave a gap of empty screen after it.
 *
 * Guarded and idempotent — `animationend` bubbles from the decorative child
 * animations too, so the container's own animation is matched by name, and a
 * repeat could otherwise report twice. Rust ignores a second call anyway;
 * this just keeps the contract honest on both sides. */
const CYCLE_ANIMATION = "splash-cycle";
/** Mirrors `--cycle-duration` in splash.css and `total_duration()` in Rust. */
const CYCLE_MS = 6000;
/** Slack before the fallback fires: enough that it never beats a healthy
 * `animationend`, far short of the Rust watchdog. */
const FALLBACK_SLACK_MS = 1500;
let reported = false;

function reportFinished() {
  if (reported) return;
  reported = true;
  void invoke("notify_splash_finished").catch((error) => {
    // Not recoverable from here, and not worth surfacing on a splash screen:
    // the backend watchdog is the safety net.
    console.error("Failed to signal splash completion:", error);
  });
}

const splash = document.querySelector(".splash");

if (splash) {
  splash.addEventListener("animationend", (event) => {
    if ((event as AnimationEvent).animationName === CYCLE_ANIMATION) {
      reportFinished();
    }
  });

  // Release the paused animations on the first frame the window actually
  // renders. `requestAnimationFrame` does not fire while the window is off
  // screen, so this is what ties the cycle's start to the moment it becomes
  // visible — see the note in splash.css.
  requestAnimationFrame(() => {
    document.documentElement.dataset.splash = "running";
    // Belt and braces: if `animationend` is ever missed (a suspended
    // compositor, a repaint the page never gets), report anyway rather than
    // leave the app waiting on the backend watchdog.
    window.setTimeout(reportFinished, CYCLE_MS + FALLBACK_SLACK_MS);
  });
} else {
  // The element should always exist; if the document ever changes shape,
  // failing to a signal is far better than failing to a stuck splash.
  reportFinished();
}
