import { readPreference, resolveSplashTheme, resolveSplashMotion } from "./resolve";
import "./splash.css";

/** Splash entry point.
 *
 * Deliberately the smallest program in the codebase: it resolves two
 * presentation attributes and stops. It imports nothing from the main app,
 * calls no Tauri API, and registers no listener — the splash window is
 * granted no capability at all (see `src-tauri/capabilities/`), so an
 * `invoke` from here would simply be denied. Its lifecycle is driven
 * entirely from Rust, which shows the main window and closes this one.
 *
 * Attributes are set before first paint (this module is the only script and
 * runs synchronously against an already-parsed body), so there is no flash
 * of the wrong theme or of motion that should have been suppressed. */

const search = typeof window !== "undefined" ? window.location.search : "";

document.documentElement.dataset.theme = resolveSplashTheme(
  readPreference(search, "theme"),
  window.matchMedia("(prefers-color-scheme: dark)").matches,
);

document.documentElement.dataset.motion = resolveSplashMotion(
  readPreference(search, "motion"),
  window.matchMedia("(prefers-reduced-motion: reduce)").matches,
);
