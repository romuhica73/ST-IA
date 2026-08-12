import type { JobStatus } from "./types";

/** How long the percentage may sit still before the UI says so.
 *
 * Chosen from measurement, not taste. Whisper decodes in windows and emits a
 * burst of segments per window, so the percentage legitimately freezes for as
 * long as one window takes. On the qualified 3-minute sample the longest gap
 * between progress events was 23.2s for the translation model (median 0.00s —
 * events arrive in bursts). A threshold below that would cry wolf on every
 * ordinary run; well above it, the user has already decided the app is dead.
 *
 * 12s is comfortably past a normal French burst and early enough to reassure
 * before doubt sets in. The reassurance is additive: the phase, the audio
 * position and the liveness animation are all still shown. */
export const STALLED_AFTER_MS = 12_000;

/** Formats a number of seconds as m:ss, or h:mm:ss past an hour.
 *
 * Used for "08:32 analysed of 18:10". Rounds down, so the figure never
 * momentarily exceeds the total. */
export function formatDuration(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  const seconds = safe % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return hours > 0 ? `${hours}:${pad(minutes)}:${pad(seconds)}` : `${minutes}:${pad(seconds)}`;
}

/** The audio position to display, or null when it is not known yet.
 *
 * Both values must be real: before the first segment is decoded there is no
 * position, and showing "0:00 of 18:10" would claim a measurement that has
 * not been made. */
export function audioPosition(
  status: JobStatus,
): { processed: number; total: number } | null {
  if (status.status !== "transcribing") return null;
  const { processedAudioSeconds, totalAudioSeconds } = status;
  if (processedAudioSeconds === null || totalAudioSeconds === null) return null;
  if (totalAudioSeconds <= 0) return null;
  return { processed: processedAudioSeconds, total: totalAudioSeconds };
}

/** Whether the job is in a state where work is genuinely ongoing, and a
 * liveness signal is therefore truthful.
 *
 * `cancelling` is excluded: something is happening, but "still analysing" is
 * the wrong thing to say while tearing a job down. */
export function isWorking(status: JobStatus): boolean {
  return (
    status.status === "preparingAudio" ||
    status.status === "transcribing" ||
    status.status === "writingOutputs"
  );
}

/** Whether to add the "some sections take longer" reassurance.
 *
 * Deliberately a function of *time since the last update*, not of the
 * percentage: a job legitimately sitting at 47% for 20 seconds is working,
 * and the message says exactly that rather than implying a problem.
 *
 * Restricted to the `processing` phase. Loading the 3.1 GB translation model
 * also takes many seconds, but "some audio sections take longer to analyse"
 * would be a false explanation there — no audio is being analysed yet. That
 * phase already shows an indeterminate bar, which reads correctly on its
 * own. */
export function isStalled(status: JobStatus, msSinceLastUpdate: number): boolean {
  return (
    status.status === "transcribing" &&
    status.phase === "processing" &&
    msSinceLastUpdate >= STALLED_AFTER_MS
  );
}
