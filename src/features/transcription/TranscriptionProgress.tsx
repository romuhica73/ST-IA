import { AudioWaveIcon, LockIcon } from "../media-selection/icons";
import { realProgress, stageState, type StageKey } from "./stages";
import type { JobStatus } from "./types";

interface TranscriptionProgressProps {
  fileName: string;
  status: JobStatus;
  onCancel: () => void;
}

const STEPS: { key: StageKey; label: string }[] = [
  { key: "audio", label: "Préparation de l'audio" },
  { key: "model", label: "Chargement du modèle" },
  { key: "transcribing", label: "Transcription" },
  { key: "writing", label: "Génération des fichiers" },
];

const STATE_LABEL: Record<ReturnType<typeof stageState>, string> = {
  done: "Terminée",
  active: "En cours…",
  pending: "En attente",
};

export function TranscriptionProgress({
  fileName,
  status,
  onCancel,
}: TranscriptionProgressProps) {
  const progress = realProgress(status);
  const cancelling = status.status === "cancelling";

  return (
    <div className="job">
      <div className="file-card">
        <div className="file-card__icon-audio" aria-hidden="true">
          <AudioWaveIcon />
        </div>
        <div className="file-card__info">
          <p className="file-card__name">{fileName}</p>
        </div>
      </div>

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

      {cancelling ? (
        // The per-stage list is meaningless once we are tearing the job
        // down, and showing every step as "done" would read as a success.
        <p className="model-gate__subtitle fade-in">Arrêt du traitement en cours…</p>
      ) : (
        <ul className="steps">
          {STEPS.map((step) => {
            const state = stageState(status, step.key);
            return (
              <li key={step.key} className={`steps__item steps__item--${state}`}>
                <span className={`steps__marker steps__marker--${state}`} aria-hidden="true">
                  {state === "done" ? "✓" : ""}
                </span>
                <span className="steps__label">{step.label}</span>
                <span className="steps__state">{STATE_LABEL[state]}</span>
              </li>
            );
          })}
        </ul>
      )}

      <div className="actions actions--single">
        <button
          type="button"
          className="button button--secondary"
          onClick={onCancel}
          disabled={cancelling}
        >
          {cancelling ? "Annulation…" : "Annuler"}
        </button>
      </div>

      <p className="home__footer">
        <LockIcon />
        Traitement 100&nbsp;% local
      </p>
    </div>
  );
}
