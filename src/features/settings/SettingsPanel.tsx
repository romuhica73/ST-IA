import { useTranslation } from "react-i18next";
import { useAppVersion } from "./useAppVersion";
import { SettingChoice } from "./SettingChoice";
import { AiModels } from "./AiModels";
import { CloseIcon } from "./icons";
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

  return (
    <div className="job settings-panel" role="dialog" aria-modal="true" aria-label={t("settings.title")}>
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

      <div className="settings-panel__body">
        <section className="settings-section">
          <h3 className="settings-section__title">{t("settings.general")}</h3>
          <div className="settings-panel__about-header">
            <p className="settings-panel__app-name">ST-IA</p>
            {version && <p className="model-gate__subtitle">{t("about.version", { version })}</p>}
          </div>
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">{t("settings.appearance")}</h3>
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
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">{t("settings.accessibility")}</h3>
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
        </section>

        <section className="settings-section">
          <h3 className="settings-section__title">{t("settings.language")}</h3>
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
        </section>

        <AiModels statuses={modelStatuses} />

        <section className="settings-section">
          <h3 className="settings-section__title">{t("settings.about")}</h3>

          <div className="about-group">
            <p className="settings-panel__app-name">ST-IA</p>
            {version && <p className="model-gate__subtitle">{t("about.version", { version })}</p>}
            <p className="settings-panel__tagline">{t("about.tagline")}</p>
          </div>

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
        </section>
      </div>
    </div>
  );
}
