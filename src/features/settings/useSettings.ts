import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_SETTINGS, type Settings } from "./types";

/**
 * Loads settings from the Rust-owned settings.json on mount, and persists
 * every change back to it. `settings` starts as `DEFAULT_SETTINGS` (all
 * "system") rather than `null` — that default is also the correct first-
 * launch state, so there is no separate "loading" screen to design for.
 */
export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);

  useEffect(() => {
    void invoke<Settings>("get_settings")
      .then(setSettings)
      .catch((error) => {
        console.error("Failed to load settings, using defaults:", error);
      });
  }, []);

  const persist = useCallback((next: Settings) => {
    setSettings(next);
    void invoke("save_settings", { settings: next }).catch((error) => {
      console.error("Failed to save settings:", error);
    });
  }, []);

  const setTheme = useCallback(
    (theme: Settings["theme"]) => persist({ ...settings, theme }),
    [settings, persist],
  );
  const setMotion = useCallback(
    (motion: Settings["motion"]) => persist({ ...settings, motion }),
    [settings, persist],
  );
  const setLanguage = useCallback(
    (language: Settings["language"]) => persist({ ...settings, language }),
    [settings, persist],
  );

  return { settings, setTheme, setMotion, setLanguage };
}
