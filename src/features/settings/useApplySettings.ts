import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { Settings } from "./types";
import { resolveLanguage, resolveMotion, resolveTheme } from "./resolve";

const DARK_QUERY = "(prefers-color-scheme: dark)";
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

/**
 * Applies the resolved theme/motion/language to the document and to i18n,
 * and keeps "system" preferences live — if the OS theme or reduced-motion
 * setting changes while ST-IA is open and the user has chosen "System", the
 * UI follows without a restart (see ADR-007 / mission §16).
 */
export function useApplySettings(settings: Settings) {
  const { i18n } = useTranslation();

  useEffect(() => {
    const media = window.matchMedia(DARK_QUERY);
    const apply = () => {
      document.documentElement.dataset.theme = resolveTheme(settings.theme, media.matches);
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [settings.theme]);

  useEffect(() => {
    const media = window.matchMedia(REDUCED_MOTION_QUERY);
    const apply = () => {
      document.documentElement.dataset.motion = resolveMotion(settings.motion, media.matches);
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [settings.motion]);

  useEffect(() => {
    const resolved = resolveLanguage(settings.language, navigator.language);
    if (i18n.language !== resolved) {
      void i18n.changeLanguage(resolved);
    }
  }, [settings.language, i18n]);
}
