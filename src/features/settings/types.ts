export type ThemePreference = "system" | "light" | "dark";
export type MotionPreference = "system" | "on" | "off";
export type LanguagePreference = "system" | "fr" | "en";

export interface Settings {
  theme: ThemePreference;
  motion: MotionPreference;
  language: LanguagePreference;
}

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  motion: "system",
  language: "system",
};
