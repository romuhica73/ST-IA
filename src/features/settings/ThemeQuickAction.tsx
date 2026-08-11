import { useTranslation } from "react-i18next";
import { nextThemePreference } from "./resolve";
import { MoonIcon, SunIcon, SystemDisplayIcon } from "./icons";
import type { ThemePreference } from "./types";

interface ThemeQuickActionProps {
  theme: ThemePreference;
  onChange: (theme: ThemePreference) => void;
}

const ICON: Record<ThemePreference, typeof SunIcon> = {
  system: SystemDisplayIcon,
  light: SunIcon,
  dark: MoonIcon,
};

const LABEL_KEY: Record<ThemePreference, string> = {
  system: "settings.themeQuickSystem",
  light: "settings.themeQuickLight",
  dark: "settings.themeQuickDark",
};

/** Cycles the same `theme` preference Settings → Appearance reads and
 * writes (system → light → dark → system) — a shortcut to it, not a
 * second source of truth. The icon reflects the *preference*, not macOS's
 * resolved appearance, so "System" always shows the same glyph regardless
 * of whether the Mac currently happens to be in light or dark mode. */
export function ThemeQuickAction({ theme, onChange }: ThemeQuickActionProps) {
  const { t } = useTranslation();
  const Icon = ICON[theme];

  return (
    <button
      type="button"
      className="app-header__button"
      onClick={() => onChange(nextThemePreference(theme))}
      aria-label={t(LABEL_KEY[theme])}
      title={t(LABEL_KEY[theme])}
    >
      <Icon />
    </button>
  );
}
