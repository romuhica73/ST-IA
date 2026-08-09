import { formatBytes } from "../media-selection/formatBytes";
import { DownloadCircleIcon } from "./icons";

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
  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">Téléchargement du modèle</p>
      <p className="model-gate__subtitle">
        {formatBytes(downloadedBytes)}
        {totalBytes !== null ? ` / ${formatBytes(totalBytes)}` : ""}
      </p>

      <section className="progress-field">
        <div className="progress-field__header">
          <span className="field__label">Progression</span>
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
