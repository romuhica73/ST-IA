import { WarningIcon } from "../media-selection/icons";
import type { TranscriptionErrorCode } from "./types";

interface TranscriptionFailureProps {
  code: TranscriptionErrorCode;
  message: string;
  fileName: string;
  onChooseAnother: () => void;
  onRetry: () => void;
  onInstallModel: () => void;
}

const ERROR_TITLES: Record<TranscriptionErrorCode, string> = {
  modelMissing: "Modèle non installé",
  noAudioTrack: "Impossible de traiter ce fichier",
  audioPreparationFailed: "Impossible de traiter ce fichier",
  transcriptionFailed: "La transcription a échoué",
  writeFailed: "Impossible d'enregistrer les fichiers",
  noOutputSelected: "Aucune sortie sélectionnée",
  alreadyRunning: "Transcription déjà en cours",
  insufficientDiskSpace: "Espace disque insuffisant",
};

export function TranscriptionFailure({
  code,
  message,
  fileName,
  onChooseAnother,
  onRetry,
  onInstallModel,
}: TranscriptionFailureProps) {
  const isModelMissing = code === "modelMissing";

  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon result-header__icon--error" aria-hidden="true">
          <WarningIcon />
        </div>
        <p className="result-header__title">{ERROR_TITLES[code]}</p>
        <p className="result-header__subtitle">{message}</p>
      </div>

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onChooseAnother}>
          Choisir un autre fichier
        </button>
        {isModelMissing ? (
          <button type="button" className="button button--primary" onClick={onInstallModel}>
            Installer le modèle
          </button>
        ) : (
          <button type="button" className="button button--primary" onClick={onRetry}>
            Réessayer
          </button>
        )}
      </div>

      <details className="tech-details">
        <summary>Détails techniques</summary>
        <dl className="tech-details__grid">
          <dt>Erreur</dt>
          <dd>{ERROR_TITLES[code]}</dd>
          <dt>Code</dt>
          <dd>{code}</dd>
          <dt>Fichier</dt>
          <dd>{fileName}</dd>
        </dl>
      </details>
    </div>
  );
}
