import type { TranscriptionErrorCode } from "./types";

interface TranscriptionFailureProps {
  code: TranscriptionErrorCode;
  message: string;
  fileName: string;
  onChooseAnother: () => void;
  onRetry: () => void;
}

const ERROR_TITLES: Record<TranscriptionErrorCode, string> = {
  modelMissing: "Modèle non installé",
  noAudioTrack: "Impossible de traiter ce fichier",
  audioPreparationFailed: "Impossible de traiter ce fichier",
  transcriptionFailed: "La transcription a échoué",
  writeFailed: "Impossible d'enregistrer les fichiers",
  noOutputSelected: "Aucune sortie sélectionnée",
  alreadyRunning: "Transcription déjà en cours",
};

export function TranscriptionFailure({
  code,
  message,
  fileName,
  onChooseAnother,
  onRetry,
}: TranscriptionFailureProps) {
  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon result-header__icon--error" aria-hidden="true">
          !
        </div>
        <p className="result-header__title">{ERROR_TITLES[code]}</p>
        <p className="result-header__subtitle">{message}</p>
        {code === "modelMissing" && (
          <p className="result-header__subtitle">
            L'installation automatique sera ajoutée dans la prochaine étape.
          </p>
        )}
      </div>

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onChooseAnother}>
          Choisir un autre fichier
        </button>
        <button type="button" className="button button--primary" onClick={onRetry}>
          Réessayer
        </button>
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
