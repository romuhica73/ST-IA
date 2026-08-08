import type { JobStatus } from "./types";

export type StageKey = "audio" | "model" | "transcribing" | "writing";
export type StageState = "done" | "active" | "pending";

const STAGE_ORDER: StageKey[] = ["audio", "model", "transcribing", "writing"];

function currentStageIndex(status: JobStatus): number {
  switch (status.status) {
    case "preparingAudio":
      return 0;
    case "transcribing":
      return status.phase === "loadingModel" ? 1 : 2;
    case "writingOutputs":
      return 3;
    default:
      return STAGE_ORDER.length;
  }
}

export function stageState(status: JobStatus, stage: StageKey): StageState {
  const current = currentStageIndex(status);
  const index = STAGE_ORDER.indexOf(stage);
  if (index < current) return "done";
  if (index === current) return "active";
  return "pending";
}

/** Real progress fraction (0-1) when available, or null for an indeterminate state. Never fabricated. */
export function realProgress(status: JobStatus): number | null {
  if (status.status === "transcribing" && status.phase === "processing") {
    return status.progress;
  }
  return null;
}
