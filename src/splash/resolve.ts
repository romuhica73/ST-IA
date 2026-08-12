/** Preference resolution for the splash window.
 *
 * The splash has no IPC capability of any kind (see ADR-009 and
 * `src-tauri/capabilities/`), so it cannot read `settings.json` the way the
 * main window does. Rust resolves nothing on its behalf either: it simply
 * forwards the two stored *preferences* as query parameters when it builds
 * the window, and the same "explicit value wins, otherwise ask the OS" rule
 * used by `features/settings/resolve.ts` is applied here.
 *
 * Keeping these as pure functions is what makes the splash's reduced-motion
 * behaviour testable without launching a window. */

export type ResolvedTheme = "light" | "dark";
export type ResolvedMotion = "reduce" | "full";

/** Mirrors `ThemePreference` / `MotionPreference` on the Rust side. Anything
 * else — including a missing parameter or a hand-edited URL — is treated as
 * "system", never as an error: a splash screen must not be able to fail.
 *
 * Preferences arrive in the URL *fragment*, not the query string. That is not
 * cosmetic: Tauri's embedded asset resolver matches the request path verbatim,
 * so `splash.html?theme=light` resolves to no asset at all and the window
 * loads a blank page. A fragment is never part of the request, so the asset
 * always resolves. Both separators are accepted here anyway, since the cost is
 * one character and the failure mode is a silently blank splash. */
export function readPreference(fragment: string, name: string): string {
  const value = new URLSearchParams(fragment.replace(/^[#?]/, "")).get(name);
  return value ?? "system";
}

export function resolveSplashTheme(
  preference: string,
  systemPrefersDark: boolean,
): ResolvedTheme {
  if (preference === "light") return "light";
  if (preference === "dark") return "dark";
  return systemPrefersDark ? "dark" : "light";
}

export function resolveSplashMotion(
  preference: string,
  systemPrefersReduced: boolean,
): ResolvedMotion {
  if (preference === "on") return "reduce";
  if (preference === "off") return "full";
  return systemPrefersReduced ? "reduce" : "full";
}
