/**
 * System locale detection (see ADR-007). v0.1 supports exactly two UI
 * languages; anything else falls back to English, chosen as the
 * international default over French so a non-French, non-English system
 * still gets an interface most users can read.
 */
export type SupportedLanguage = "fr" | "en";

export const SUPPORTED_LANGUAGES: readonly SupportedLanguage[] = ["fr", "en"];
export const FALLBACK_LANGUAGE: SupportedLanguage = "en";

/** `navigator.language`-shaped input (e.g. "fr-FR", "en-US", "de"). */
export function resolveSystemLanguage(rawLocale: string | undefined | null): SupportedLanguage {
  const primary = (rawLocale ?? "").slice(0, 2).toLowerCase();
  if (primary === "fr") return "fr";
  if (primary === "en") return "en";
  return FALLBACK_LANGUAGE;
}

/** Narrows i18next's `i18n.language` (typed as plain `string`) back to
 * `SupportedLanguage`. Should always already be "fr" or "en" in practice —
 * every path that sets it goes through resolveSystemLanguage/resolveLanguage
 * — this only exists so a caller never has to `as`-cast past the type
 * system on an invariant that lives elsewhere. */
export function asSupportedLanguage(language: string): SupportedLanguage {
  return resolveSystemLanguage(language);
}
