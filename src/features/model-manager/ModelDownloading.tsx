import { useTranslation } from "react-i18next";
import { formatBytes } from "../media-selection/formatBytes";
import { DownloadCircleIcon } from "./icons";
import { asSupportedLanguage } from "../../i18n/locale";

interface ModelDownloadingProps {
  downloadedBytes: number;
  totalBytes: number | null;
  progress: number | null;
}

export function ModelDownloading({
  downloadedBytes,
  totalBytes,
  progress,
}: ModelDownloadingProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);

  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">{t("model.downloading")}</p>
      <p className="model-gate__subtitle">
        {totalBytes !== null
          ? t("model.downloadedOf", {
              downloaded: formatBytes(downloadedBytes, language),
              total: formatBytes(totalBytes, language),
            })
          : formatBytes(downloadedBytes, language)}
      </p>

      <section className="progress-field">
        <div className="progress-field__header">
          <span className="field__label">{t("progress.label")}</span>
          {progress !== null && (
            <span className="progress-field__percent">{Math.round(progress * 100)} %</span>
          )}
        </div>
        <div
          className={`progress-bar ${progress === null ? "progress-bar--indeterminate" : ""}`}
          role="progressbar"
          aria-valuenow={progress !== null ? Math.round(progress * 100) : undefined}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className="progress-bar__fill"
            style={progress !== null ? { width: `${progress * 100}%` } : undefined}
          />
        </div>
      </section>
    </div>
  );
}
