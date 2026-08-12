import { describe, expect, it } from "vitest";
import { realProgress, stageState, stagesFor } from "./stages";
import type { JobStatus } from "./types";

const transcribing = (
  variant: "original" | "englishTranslation",
  progress: number | null = null,
): JobStatus => ({
  status: "transcribing",
  variant,
  phase: progress === null ? "loadingModel" : "processing",
  progress,
  processedAudioSeconds: progress === null ? null : progress * 1090,
  totalAudioSeconds: 1090,
});

describe("stage sequence", () => {
  it("omits the translation step for a French-only job", () => {
    // The pre-bilingual progress UI, unchanged.
    expect(stagesFor(false)).toEqual(["audio", "transcribing", "writing"]);
  });

  it("inserts the translation step between transcription and writing", () => {
    expect(stagesFor(true)).toEqual(["audio", "transcribing", "translating", "writing"]);
  });
});

describe("stage state", () => {
  const stages = stagesFor(true);

  it("marks the French pass active while it runs", () => {
    const status = transcribing("original", 0.4);
    expect(stageState(status, "audio", stages)).toBe("done");
    expect(stageState(status, "transcribing", stages)).toBe("active");
    expect(stageState(status, "translating", stages)).toBe("pending");
    expect(stageState(status, "writing", stages)).toBe("pending");
  });

  it("marks transcription done once the translation starts", () => {
    // The visible hand-off the user is meant to notice.
    const status = transcribing("englishTranslation", 0.1);
    expect(stageState(status, "transcribing", stages)).toBe("done");
    expect(stageState(status, "translating", stages)).toBe("active");
  });

  it("marks every pass done while writing outputs", () => {
    const status: JobStatus = { status: "writingOutputs" };
    expect(stageState(status, "transcribing", stages)).toBe("done");
    expect(stageState(status, "translating", stages)).toBe("done");
    expect(stageState(status, "writing", stages)).toBe("active");
  });

  it("never marks a French-only job as translating", () => {
    const monolingual = stagesFor(false);
    const status = transcribing("original", 0.5);
    expect(monolingual.includes("translating")).toBe(false);
    expect(stageState(status, "writing", monolingual)).toBe("pending");
  });
});

describe("progress", () => {
  it("reports the running pass's own real progress", () => {
    expect(realProgress(transcribing("englishTranslation", 0.42))).toBe(0.42);
    expect(realProgress(transcribing("original", 0.1))).toBe(0.1);
  });

  it("is indeterminate while a model is loading", () => {
    expect(realProgress(transcribing("original"))).toBe(null);
  });

  it("is never fabricated for non-transcribing states", () => {
    expect(realProgress({ status: "preparingAudio" })).toBe(null);
    expect(realProgress({ status: "writingOutputs" })).toBe(null);
  });
});
