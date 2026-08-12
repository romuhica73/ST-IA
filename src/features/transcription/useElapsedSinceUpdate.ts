import { useEffect, useRef, useState } from "react";

/**
 * Milliseconds since `value` last changed.
 *
 * Used to tell "the percentage is holding still because Whisper is mid-window"
 * apart from "the percentage is holding still and the user is about to give
 * up". The clock is local and only measures the gap between backend updates —
 * it never advances any displayed progress figure.
 *
 * Ticks once a second, which is enough to cross a 12-second threshold
 * promptly without re-rendering the screen continuously.
 */
export function useElapsedSinceUpdate(value: unknown, active: boolean): number {
  const [elapsed, setElapsed] = useState(0);
  const changedAt = useRef(Date.now());

  // Reset the moment the observed value changes — this is what makes the
  // measurement "since the last update" rather than "since the job started".
  useEffect(() => {
    changedAt.current = Date.now();
    setElapsed(0);
  }, [value]);

  useEffect(() => {
    if (!active) {
      setElapsed(0);
      return;
    }
    const id = window.setInterval(() => {
      setElapsed(Date.now() - changedAt.current);
    }, 1000);
    return () => window.clearInterval(id);
  }, [active]);

  return elapsed;
}
