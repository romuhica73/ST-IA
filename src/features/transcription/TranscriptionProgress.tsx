import { useTranslation } from "react-i18next";
import { AudioWaveIcon, LockIcon } from "../media-selection/icons";
import { realProgress, stageState, type StageKey } from "./stages";
import type { JobStatus } from "./types";

interface TranscriptionProgressProps {
  fileName: string;
  status: JobStatus;
  onCancel: () => void;
}

const STEP_KEYS: { key: StageKey; labelKey: string }[] = [
  { key: "audio", labelKey: "progress.audioPreparation" },
  { key: "model", labelKey: "progress.modelLoading" },
  { key: "transcribing", labelKey: "progress.transcription" },
  { key: "writing", labelKey: "progress.outputGeneration" },
];

const STATE_LABEL_KEY: Record<ReturnType<typeof stageState>, string> = {
  done: "progress.stateDone",
  active: "progress.stateActive",
  pending: "progress.statePending",
};

export function TranscriptionProgress({
  fileName,
  status,
  onCancel,
}: TranscriptionProgressProps) {
  const { t } = useTranslation();
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

      {cancelling ? (
        // The per-stage list is meaningless once we are tearing the job
        // down, and showing every step as "done" would read as a success.
        <p className="model-gate__subtitle fade-in">{t("progress.cancellingText")}</p>
      ) : (
        <ul className="steps">
          {STEP_KEYS.map((step) => {
            const state = stageState(status, step.key);
            return (
              <li key={step.key} className={`steps__item steps__item--${state}`}>
                <span className={`steps__marker steps__marker--${state}`} aria-hidden="true">
                  {state === "done" ? "✓" : ""}
                </span>
                <span className="steps__label">{t(step.labelKey)}</span>
                <span className="steps__state">{t(STATE_LABEL_KEY[state])}</span>
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
          {cancelling ? t("transcription.cancelling") : t("transcription.cancel")}
        </button>
      </div>

      <p className="home__footer">
        <LockIcon />
        {t("common.localProcessing")}
      </p>
    </div>
  );
}
