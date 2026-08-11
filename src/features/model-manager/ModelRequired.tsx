import { useTranslation } from "react-i18next";
import { formatBytes } from "../media-selection/formatBytes";
import { DownloadCircleIcon, InfoIcon } from "./icons";
import { asSupportedLanguage } from "../../i18n/locale";
import type { ModelErrorCode, ModelManifest } from "./types";

interface ModelRequiredProps {
  manifest: ModelManifest | null;
  errorCode?: ModelErrorCode | null;
  onDownload: () => void;
  onLater: () => void;
}

export function ModelRequired({ manifest, errorCode, onDownload, onLater }: ModelRequiredProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);

  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">{t("model.requiredTitle")}</p>
      <p className="model-gate__subtitle">{t("model.requiredSubtitle")}</p>

      <div className="model-card">
        <p className="model-card__name">{t("model.name")}</p>
        {manifest && (
          <p className="model-card__size">
            {t("model.sizeApprox", { size: formatBytes(manifest.expectedSize, language) })}
          </p>
        )}
      </div>

      <p className="model-gate__info">
        <InfoIcon />
        {t("model.storageInfo")}
      </p>

      {errorCode && (
        <p className="drop-zone__error" role="alert">
          {t(`error.modelCodes.${errorCode}.message`)}
        </p>
      )}

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onLater}>
          {t("model.later")}
        </button>
        <button type="button" className="button button--primary" onClick={onDownload}>
          {t("model.download")}
        </button>
      </div>
    </div>
  );
}
