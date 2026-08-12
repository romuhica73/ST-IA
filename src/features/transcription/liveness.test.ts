import { describe, expect, it } from "vitest";
import {
  audioPosition,
  formatDuration,
  isStalled,
  isWorking,
  STALLED_AFTER_MS,
} from "./liveness";
import type { JobStatus } from "./types";

const transcribing = (
  processedAudioSeconds: number | null,
  totalAudioSeconds: number | null = 1090,
): JobStatus => ({
  status: "transcribing",
  variant: "original",
  phase: processedAudioSeconds === null ? "loadingModel" : "processing",
  progress: null,
  processedAudioSeconds,
  totalAudioSeconds,
});

describe("duration formatting", () => {
  it("formats minutes and seconds", () => {
    expect(formatDuration(512)).toBe("8:32");
    expect(formatDuration(1090)).toBe("18:10");
    expect(formatDuration(5)).toBe("0:05");
    expect(formatDuration(0)).toBe("0:00");
  });

  it("adds hours only past an hour", () => {
    expect(formatDuration(3599)).toBe("59:59");
    expect(formatDuration(3600)).toBe("1:00:00");
    expect(formatDuration(3725)).toBe("1:02:05");
  });

  it("rounds down so the figure never exceeds the total", () => {
    // 8:32.9 must not render as 8:33 while the total renders as 8:32.
    expect(formatDuration(512.9)).toBe("8:32");
  });

  it("never renders a negative time", () => {
    expect(formatDuration(-5)).toBe("0:00");
  });
});

describe("audio position", () => {
  it("reports the measured position once a segment has been decoded", () => {
    expect(audioPosition(transcribing(512))).toEqual({ processed: 512, total: 1090 });
  });

  it("is unknown before the first segment, rather than zero", () => {
    // "0:00 of 18:10" would claim a measurement that has not been made.
    expect(audioPosition(transcribing(null))).toBe(null);
  });

  it("is unknown when the audio duration could not be read", () => {
    expect(audioPosition(transcribing(512, null))).toBe(null);
    expect(audioPosition(transcribing(512, 0))).toBe(null);
  });

  it("is absent outside the transcribing states", () => {
    expect(audioPosition({ status: "preparingAudio" })).toBe(null);
    expect(audioPosition({ status: "writingOutputs" })).toBe(null);
    expect(audioPosition({ status: "idle" })).toBe(null);
  });
});

describe("working state", () => {
  it("covers every state where work is genuinely happening", () => {
    expect(isWorking({ status: "preparingAudio" })).toBe(true);
    expect(isWorking(transcribing(10))).toBe(true);
    expect(isWorking({ status: "writingOutputs" })).toBe(true);
  });

  it("excludes cancelling and terminal states", () => {
    // Something is happening while cancelling, but "still analysing" is the
    // wrong thing to claim during teardown.
    expect(isWorking({ status: "cancelling" })).toBe(false);
    expect(isWorking({ status: "cancelled" })).toBe(false);
    expect(isWorking({ status: "idle" })).toBe(false);
  });
});

describe("stalled reassurance", () => {
  it("stays hidden during a normal burst of updates", () => {
    expect(isStalled(transcribing(512), 0)).toBe(false);
    expect(isStalled(transcribing(512), STALLED_AFTER_MS - 1)).toBe(false);
  });

  it("appears once the gap exceeds the measured window", () => {
    expect(isStalled(transcribing(512), STALLED_AFTER_MS)).toBe(true);
  });

  it("is a function of time, not of the percentage", () => {
    // A job legitimately holding at the same position IS working; the copy
    // says so rather than implying a fault.
    expect(isStalled(transcribing(512), 30_000)).toBe(true);
  });

  it("never appears outside transcription", () => {
    expect(isStalled({ status: "writingOutputs" }, 60_000)).toBe(false);
    expect(isStalled({ status: "preparingAudio" }, 60_000)).toBe(false);
  });

  it("never appears while a model is still loading", () => {
    // Loading the 3.1 GB translation model takes many seconds, but no audio
    // is being analysed yet — the message would be a false explanation.
    expect(isStalled(transcribing(null), 60_000)).toBe(false);
  });

  it("uses a threshold above the longest gap measured on real audio", () => {
    // 23.2s was the worst observed gap for the translation model, but the
    // message is reassurance rather than an alarm, so it fires earlier —
    // just not so early that it fires on every ordinary burst.
    expect(STALLED_AFTER_MS).toBeGreaterThanOrEqual(10_000);
    expect(STALLED_AFTER_MS).toBeLessThan(23_200);
  });
});
