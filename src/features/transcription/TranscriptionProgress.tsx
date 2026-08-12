import { useTranslation } from "react-i18next";
import { AudioWaveIcon, LockIcon } from "../media-selection/icons";
import { realProgress, stageState, stagesFor, type StageKey } from "./stages";
import { audioPosition, formatDuration, isStalled, isWorking } from "./liveness";
import { useElapsedSinceUpdate } from "./useElapsedSinceUpdate";
import type { JobStatus } from "./types";

interface TranscriptionProgressProps {
  fileName: string;
  status: JobStatus;
  /** Whether this job was asked for both versions — determines whether the
   * translation step is part of the sequence at all. */
  bilingual: boolean;
  onCancel: () => void;
}

const STEP_LABELS: Record<StageKey, string> = {
  audio: "progress.audioPreparation",
  transcribing: "progress.transcription",
  translating: "progress.translation",
  writing: "progress.outputGeneration",
};

const STATE_LABEL_KEY: Record<ReturnType<typeof stageState>, string> = {
  done: "progress.stateDone",
  active: "progress.stateActive",
  pending: "progress.statePending",
};

export function TranscriptionProgress({
  fileName,
  status,
  bilingual,
  onCancel,
}: TranscriptionProgressProps) {
  const { t } = useTranslation();
  const progress = realProgress(status);
  const cancelling = status.status === "cancelling";
  const stages = stagesFor(bilingual);
  const position = audioPosition(status);
  const working = isWorking(status) && !cancelling;

  // Measured against the audio position rather than the percentage: the
  // position moves in finer steps, so this reflects "Whisper produced
  // something" as directly as we can observe it.
  const elapsedSinceUpdate = useElapsedSinceUpdate(position?.processed ?? null, working);
  const stalled = !cancelling && isStalled(status, elapsedSinceUpdate);

  const phaseLabel =
    status.status === "transcribing"
      ? t(status.variant === "englishTranslation" ? "progress.translation" : "progress.transcription")
      : t("progress.label");

  const percent = progress !== null ? Math.round(progress * 100) : null;

  // Screen readers get the phase and the position in one utterance, because
  // a bare "47%" is exactly the signal that proved insufficient visually.
  const valueText = [
    phaseLabel,
    percent !== null ? `${percent} %` : null,
    position ? t("progress.analysedOf", {
      processed: formatDuration(position.processed),
      total: formatDuration(position.total),
    }) : null,
    working ? t("progress.stillWorking") : null,
  ]
    .filter(Boolean)
    .join(", ");

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
          <span className="field__label">{phaseLabel}</span>
          {percent !== null && <span className="progress-field__percent">{percent} %</span>}
        </div>

        {/* Real audio position — the honest answer to "is it still going?"
            when the percentage has not moved for a whole decoding window. */}
        {position && (
          <p className="progress-field__position">
            {t("progress.analysedOf", {
              processed: formatDuration(position.processed),
              total: formatDuration(position.total),
            })}
          </p>
        )}

        <div
          className={`progress-bar ${progress === null ? "progress-bar--indeterminate" : ""} ${
            working ? "progress-bar--live" : ""
          }`}
          role="progressbar"
          aria-valuenow={percent ?? undefined}
          aria-valuemin={progress !== null ? 0 : undefined}
          aria-valuemax={progress !== null ? 100 : undefined}
          aria-valuetext={valueText}
        >
          <div
            className="progress-bar__fill"
            style={progress !== null ? { width: `${progress * 100}%` } : undefined}
          />
        </div>

        {/* Liveness in words as well as motion, so the signal survives
            reduced motion and screen readers. */}
        {working && (
          <p className="progress-field__liveness">
            <span className="progress-field__pulse" aria-hidden="true" />
            {stalled ? t("progress.stalledHint") : t("progress.stillWorking")}
          </p>
        )}
      </section>

      {cancelling ? (
        // The per-stage list is meaningless once we are tearing the job
        // down, and showing every step as "done" would read as a success.
        <p className="model-gate__subtitle fade-in">{t("progress.cancellingText")}</p>
      ) : (
        <ul className="steps">
          {stages.map((stage) => {
            const state = stageState(status, stage, stages);
            return (
              <li key={stage} className={`steps__item steps__item--${state}`}>
                <span className={`steps__marker steps__marker--${state}`} aria-hidden="true">
                  {state === "done" ? "✓" : ""}
                </span>
                <span className="steps__label">{t(STEP_LABELS[stage])}</span>
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
