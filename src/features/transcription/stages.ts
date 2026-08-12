import type { JobStatus } from "./types";

export type StageKey = "audio" | "transcribing" | "translating" | "writing";
export type StageState = "done" | "active" | "pending";

/** The stages actually shown, which depend on what the job was asked to do.
 *
 * A French-only job shows exactly the steps it did before English output
 * existed. A bilingual job inserts the translation step between the
 * transcription and the file writing — the two passes are real, sequential
 * stages, so they are shown as such rather than blended into one bar. */
export function stagesFor(bilingual: boolean): StageKey[] {
  return bilingual
    ? ["audio", "transcribing", "translating", "writing"]
    : ["audio", "transcribing", "writing"];
}

function currentStageKey(status: JobStatus): StageKey | "done" {
  switch (status.status) {
    case "preparingAudio":
      return "audio";
    case "transcribing":
      return status.variant === "englishTranslation" ? "translating" : "transcribing";
    case "writingOutputs":
      return "writing";
    default:
      return "done";
  }
}

export function stageState(status: JobStatus, stage: StageKey, stages: StageKey[]): StageState {
  const current = currentStageKey(status);
  const currentIndex = current === "done" ? stages.length : stages.indexOf(current);
  const index = stages.indexOf(stage);
  if (index < currentIndex) return "done";
  if (index === currentIndex) return "active";
  return "pending";
}

/** Real progress fraction (0-1) when available, or null for an indeterminate state. Never fabricated. */
export function realProgress(status: JobStatus): number | null {
  if (status.status === "transcribing" && status.phase === "processing") {
    // The active pass's own position in the audio. Deliberately not blended
    // into a global percentage across both passes: the two run at very
    // different speeds, so a combined bar would be a fabricated number.
    return status.progress;
  }
  return null;
}
