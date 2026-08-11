import type { LanguagePreference, MotionPreference, ThemePreference } from "./types";
import { resolveSystemLanguage, type SupportedLanguage } from "../../i18n/locale";

/** Pure resolution functions — the same logic drives the synchronous
 * pre-paint default and every later re-resolution (settings loaded, system
 * preference changed live), so there is exactly one place that can get this
 * wrong. */

export function resolveTheme(
  preference: ThemePreference,
  systemPrefersDark: boolean,
): "light" | "dark" {
  if (preference === "light") return "light";
  if (preference === "dark") return "dark";
  return systemPrefersDark ? "dark" : "light";
}

export function resolveMotion(
  preference: MotionPreference,
  systemPrefersReduced: boolean,
): "reduce" | "full" {
  if (preference === "on") return "reduce";
  if (preference === "off") return "full";
  return systemPrefersReduced ? "reduce" : "full";
}

export function resolveLanguage(
  preference: LanguagePreference,
  systemLocale: string | undefined | null,
): SupportedLanguage {
  if (preference === "fr" || preference === "en") return preference;
  return resolveSystemLanguage(systemLocale);
}

const THEME_CYCLE: readonly ThemePreference[] = ["system", "light", "dark"];

/** One click of the header's quick-theme action: system → light → dark →
 * system. Cycles the *preference*, the same one Settings → Appearance
 * reads and writes — there is no second state to keep in sync. */
export function nextThemePreference(current: ThemePreference): ThemePreference {
  const index = THEME_CYCLE.indexOf(current);
  return THEME_CYCLE[(index + 1) % THEME_CYCLE.length];
}
