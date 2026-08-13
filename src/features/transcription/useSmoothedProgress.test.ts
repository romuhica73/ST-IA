import { describe, expect, it } from "vitest";
import { CATCH_UP_MS, easedProgress } from "./useSmoothedProgress";

describe("progress easing", () => {
  it("starts at the previous measured value", () => {
    expect(easedProgress(0.4, 0.47, 0)).toBeCloseTo(0.4, 5);
  });

  it("arrives exactly at the new measured value", () => {
    expect(easedProgress(0.4, 0.47, CATCH_UP_MS)).toBeCloseTo(0.47, 5);
  });

  it("never overshoots the measured value, at any point", () => {
    // The safety property: this interpolates *between* two real values and
    // must never extrapolate past the newer one.
    for (let elapsed = 0; elapsed <= CATCH_UP_MS * 3; elapsed += 7) {
      expect(easedProgress(0.4, 0.47, elapsed)).toBeLessThanOrEqual(0.47);
    }
  });

  it("stays at the target once the catch-up is over", () => {
    // A bar that kept creeping forward after the last real value would be
    // inventing progress. It stops and waits.
    expect(easedProgress(0.4, 0.47, CATCH_UP_MS * 5)).toBeCloseTo(0.47, 5);
    expect(easedProgress(0.4, 0.47, 60_000)).toBeCloseTo(0.47, 5);
  });

  it("moves monotonically forward", () => {
    let previous = -1;
    for (let elapsed = 0; elapsed <= CATCH_UP_MS; elapsed += 10) {
      const value = easedProgress(0.1, 0.9, elapsed);
      expect(value).toBeGreaterThanOrEqual(previous);
      previous = value;
    }
  });

  it("eases out: more of the distance is covered early", () => {
    // Quick to acknowledge a new value, gentle to settle — this is what
    // turns a burst of segments into one readable sweep.
    const half = easedProgress(0, 1, CATCH_UP_MS / 2);
    expect(half).toBeGreaterThan(0.5);
  });

  it("handles a negative elapsed time without going backwards", () => {
    expect(easedProgress(0.4, 0.47, -100)).toBeCloseTo(0.4, 5);
  });

  it("is stable when there is nothing to move to", () => {
    expect(easedProgress(0.47, 0.47, 0)).toBeCloseTo(0.47, 5);
    expect(easedProgress(0.47, 0.47, CATCH_UP_MS)).toBeCloseTo(0.47, 5);
  });
});
