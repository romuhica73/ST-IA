import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppVersion } from "./useAppVersion";
import { SettingChoice } from "./SettingChoice";
import { AiModels } from "./AiModels";
import { CloseIcon } from "./icons";
import {
  DEFAULT_SECTION,
  moveSelection,
  SECTION_LABEL_KEY,
  SETTINGS_SECTIONS,
  type SettingsSection,
} from "./sections";
import type { Settings } from "./types";
import type { ModelKind, ModelStatus } from "../model-manager/types";

interface SettingsPanelProps {
  settings: Settings;
  /** Statuses the app has already resolved. Passed in rather than fetched
   * here so that opening Settings never triggers a fresh 3.1 GB hash. */
  modelStatuses: Partial<Record<ModelKind, ModelStatus | null>>;
  onThemeChange: (theme: Settings["theme"]) => void;
  onMotionChange: (motion: Settings["motion"]) => void;
  onLanguageChange: (language: Settings["language"]) => void;
  onClose: () => void;
}

/**
 * Settings as a desktop panel: a compact navigation column on the left, one
 * section at a time on the right.
 *
 * The previous version stacked every section vertically, which only worked
 * because the window grew to fit it. In a fixed shell that stacking is what
 * forces a scrollbar, so the sections became real navigation instead.
 */
export function SettingsPanel({
  settings,
  modelStatuses,
  onThemeChange,
  onMotionChange,
  onLanguageChange,
  onClose,
}: SettingsPanelProps) {
  const { t } = useTranslation();
  const version = useAppVersion();
  const [section, setSection] = useState<SettingsSection>(DEFAULT_SECTION);

  /** Arrow keys move between sections, matching how a native source list
   * behaves. Home/End jump to the ends. */
  function handleNavKeyDown(event: React.KeyboardEvent<HTMLDivElement>) {
    const moves: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
    if (event.key in moves) {
      event.preventDefault();
      setSection(moveSelection(section, moves[event.key]));
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      setSection(SETTINGS_SECTIONS[0]);
    }
    if (event.key === "End") {
      event.preventDefault();
      setSection(SETTINGS_SECTIONS[SETTINGS_SECTIONS.length - 1]);
    }
  }

  return (
    <div
      className="settings-panel"
      role="dialog"
      aria-modal="true"
      aria-label={t("settings.title")}
    >
      <div className="settings-panel__header">
        <p className="settings-panel__title">{t("settings.title")}</p>
        <button
          type="button"
          className="settings-panel__close"
          onClick={onClose}
          aria-label={t("settings.close")}
        >
          <CloseIcon />
        </button>
      </div>

      <div className="settings-panel__layout">
        {/* A tablist rather than a list of buttons: the relationship between
            each control and the panel it reveals is the point, and it is what
            gives assistive technology "tab 2 of 4" instead of four unrelated
            buttons. Roving tabindex keeps it to one stop in the tab order. */}
        <div
          className="settings-nav"
          role="tablist"
          aria-orientation="vertical"
          aria-label={t("settings.title")}
          onKeyDown={handleNavKeyDown}
        >
          {SETTINGS_SECTIONS.map((key) => (
            <button
              key={key}
              type="button"
              role="tab"
              id={`settings-tab-${key}`}
              aria-controls={`settings-panel-${key}`}
              aria-selected={section === key}
              tabIndex={section === key ? 0 : -1}
              className={`settings-nav__item ${
                section === key ? "settings-nav__item--active" : ""
              }`}
              onClick={() => setSection(key)}
            >
              {t(SECTION_LABEL_KEY[key])}
            </button>
          ))}
        </div>

        {/* Keyed on the section so React remounts it, which replays the short
            reveal transition and resets the scroll position of a long panel
            rather than carrying the previous section's offset into it. */}
        <div
          className="settings-content"
          key={section}
          role="tabpanel"
          id={`settings-panel-${section}`}
          aria-labelledby={`settings-tab-${section}`}
          tabIndex={0}
        >
          {section === "general" && (
            <>
              <SettingChoice
                label={t("settings.languageLabel")}
                value={settings.language}
                onChange={onLanguageChange}
                options={[
                  { value: "system", label: t("settings.languageSystem") },
                  { value: "fr", label: t("settings.languageFrench") },
                  { value: "en", label: t("settings.languageEnglish") },
                ]}
              />
              <SettingChoice
                label={t("settings.themeLabel")}
                value={settings.theme}
                onChange={onThemeChange}
                options={[
                  { value: "system", label: t("settings.themeSystem") },
                  { value: "light", label: t("settings.themeLight") },
                  { value: "dark", label: t("settings.themeDark") },
                ]}
              />
            </>
          )}

          {section === "accessibility" && (
            <SettingChoice
              label={t("settings.motionLabel")}
              value={settings.motion}
              onChange={onMotionChange}
              options={[
                { value: "system", label: t("settings.motionSystem") },
                { value: "on", label: t("settings.motionOn") },
                { value: "off", label: t("settings.motionOff") },
              ]}
            />
          )}

          {section === "models" && <AiModels statuses={modelStatuses} />}

          {section === "about" && (
            <div className="about-group">
              <p className="settings-panel__app-name">ST-IA</p>
              {version && (
                <p className="model-gate__subtitle">{t("about.version", { version })}</p>
              )}
              <p className="settings-panel__tagline">{t("about.tagline")}</p>

              <div className="about-group about-group--divided">
                <p className="model-gate__subtitle">{t("about.localProcessing")}</p>
              </div>

              <div className="about-group about-group--divided">
                <p className="about-group__heading">{t("about.licenses")}</p>
                <div className="third-party-list">
                  <div className="third-party-list__row">
                    <span className="third-party-list__name">FFmpeg 9.0</span>
                    <span className="third-party-list__license">LGPL-2.1</span>
                  </div>
                  <div className="third-party-list__row">
                    <span className="third-party-list__name">whisper.cpp v1.9.2</span>
                    <span className="third-party-list__license">MIT</span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
