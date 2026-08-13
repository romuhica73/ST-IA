import { useEffect, useRef, useState } from "react";

/** How long the displayed value takes to catch up to a newly measured one.
 *
 * Whisper delivers segments in bursts, so several real values can land within
 * a few milliseconds and the bar would otherwise snap through them. Easing
 * over ~700ms turns a burst into one continuous sweep, which is what makes
 * the movement legible rather than twitchy. */
export const CATCH_UP_MS = 700;

/** The eased value between two measured points, at `elapsed` ms into the
 * catch-up.
 *
 * Pure, and clamped to `target` at every step: the displayed value is never
 * ahead of what was actually measured, not even by a rounding error mid-frame.
 * That clamp is the whole safety property — this function interpolates
 * *between* two real values and can never extrapolate past the newer one. */
export function easedProgress(from: number, target: number, elapsed: number): number {
  const t = Math.min(1, Math.max(0, elapsed) / CATCH_UP_MS);
  // easeOutCubic: quick to acknowledge the new value, gentle to settle.
  const eased = 1 - Math.pow(1 - t, 3);
  return Math.min(target, from + (target - from) * eased);
}

/**
 * Eases the *displayed* progress toward the last measured value.
 *
 * This interpolates strictly **between two real measurements**. It never
 * extrapolates: the displayed value converges on the last value the backend
 * actually reported and then stops, however long the next one takes. A bar
 * that kept creeping forward on its own would be inventing progress, which is
 * exactly what this project refuses to do — the liveness pulse, not the bar,
 * is what says "still working" during a long gap.
 *
 * Returns null whenever the real value is null, so an indeterminate state
 * stays indeterminate.
 */
export function useSmoothedProgress(target: number | null, animate: boolean): number | null {
  const [displayed, setDisplayed] = useState<number | null>(target);
  const frame = useRef<number | null>(null);
  const from = useRef<number>(target ?? 0);
  const startedAt = useRef<number>(0);

  useEffect(() => {
    if (target === null) {
      setDisplayed(null);
      return;
    }
    // Reduced motion, or a first value with nothing to ease from: show it.
    if (!animate || displayed === null) {
      setDisplayed(target);
      return;
    }
    // Never animate backwards. A lower value can only mean a new pass has
    // started, and sliding the bar back would read as lost work.
    if (target < displayed) {
      setDisplayed(target);
      return;
    }

    from.current = displayed;
    startedAt.current = performance.now();

    const step = (now: number) => {
      const elapsed = now - startedAt.current;
      setDisplayed(easedProgress(from.current, target, elapsed));
      if (elapsed < CATCH_UP_MS) {
        frame.current = requestAnimationFrame(step);
      }
    };

    frame.current = requestAnimationFrame(step);
    return () => {
      if (frame.current !== null) cancelAnimationFrame(frame.current);
    };
    // `displayed` is deliberately not a dependency: including it would
    // restart the easing on every frame it sets.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [target, animate]);

  return displayed;
}
