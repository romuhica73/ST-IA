import { formatBytes } from "../media-selection/formatBytes";
import { DownloadCircleIcon, InfoIcon } from "./icons";
import type { ModelManifest } from "./types";

interface ModelRequiredProps {
  manifest: ModelManifest | null;
  errorMessage?: string | null;
  onDownload: () => void;
  onLater: () => void;
}

export function ModelRequired({
  manifest,
  errorMessage,
  onDownload,
  onLater,
}: ModelRequiredProps) {
  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">Le modèle local n'est pas encore installé</p>
      <p className="model-gate__subtitle">
        Télécharger le modèle pour utiliser l'application hors ligne.
      </p>

      <div className="model-card">
        <p className="model-card__name">Whisper — large-v3-turbo</p>
        {manifest && (
          <p className="model-card__size">~{formatBytes(manifest.expectedSize)}</p>
        )}
      </div>

      <p className="model-gate__info">
        <InfoIcon />
        Le modèle sera stocké sur votre ordinateur.
      </p>

      {errorMessage && (
        <p className="drop-zone__error" role="alert">
          {errorMessage}
        </p>
      )}

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onLater}>
          Plus tard
        </button>
        <button type="button" className="button button--primary" onClick={onDownload}>
          Télécharger
        </button>
      </div>
    </div>
  );
}
