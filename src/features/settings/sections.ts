/** The Settings sections, in navigation order.
 *
 * A single list drives both the sidebar and the panel body, so a section can
 * never exist in one and not the other. `labelKey` is resolved at render
 * time — the order is fixed, the wording is localised.
 */
export const SETTINGS_SECTIONS = ["general", "accessibility", "models", "about"] as const;

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

export const DEFAULT_SECTION: SettingsSection = "general";

export const SECTION_LABEL_KEY: Record<SettingsSection, string> = {
  general: "settings.general",
  accessibility: "settings.accessibility",
  models: "aiModels.title",
  about: "settings.about",
};

/** Moves the selection by `delta`, stopping at the ends rather than wrapping.
 *
 * Arrow keys in a vertical list are expected to stop at the boundaries —
 * wrapping from the last item back to the first is what a menu does, not a
 * navigation list, and it makes it easy to lose your place. */
export function moveSelection(current: SettingsSection, delta: number): SettingsSection {
  const index = SETTINGS_SECTIONS.indexOf(current);
  const next = Math.min(SETTINGS_SECTIONS.length - 1, Math.max(0, index + delta));
  return SETTINGS_SECTIONS[next];
}
