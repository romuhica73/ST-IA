import { describe, expect, it } from "vitest";
import { isMeasurable, isSignificantChange, RESIZE_THRESHOLD_PX } from "./windowFit";

describe("significant change detection", () => {
  it("is always significant the first time (no previous measurement)", () => {
    expect(isSignificantChange(null, { width: 720, height: 480 })).toBe(true);
  });

  it("ignores sub-threshold noise on either axis", () => {
    const previous = { width: 720, height: 480 };
    expect(isSignificantChange(previous, { width: 722, height: 481 })).toBe(false);
  });

  it("reacts once a real layout change crosses the threshold", () => {
    const previous = { width: 720, height: 480 };
    expect(
      isSignificantChange(previous, { width: 720, height: 480 + RESIZE_THRESHOLD_PX }),
    ).toBe(true);
  });

  it("reacts to width alone, without requiring height to move too", () => {
    const previous = { width: 720, height: 480 };
    expect(
      isSignificantChange(previous, { width: 720 + RESIZE_THRESHOLD_PX, height: 480 }),
    ).toBe(true);
  });

  it("reacts to a shrink exactly like a growth", () => {
    const previous = { width: 720, height: 480 };
    expect(
      isSignificantChange(previous, { width: 720, height: 480 - RESIZE_THRESHOLD_PX }),
    ).toBe(true);
  });
});

describe("measurability guard", () => {
  it("accepts an ordinary measured size", () => {
    expect(isMeasurable({ width: 720, height: 480 })).toBe(true);
  });

  it("rejects a size not yet laid out", () => {
    expect(isMeasurable({ width: 0, height: 0 })).toBe(false);
  });

  it("rejects negative or non-finite values", () => {
    expect(isMeasurable({ width: -10, height: 480 })).toBe(false);
    expect(isMeasurable({ width: NaN, height: 480 })).toBe(false);
    expect(isMeasurable({ width: Infinity, height: 480 })).toBe(false);
  });
});
