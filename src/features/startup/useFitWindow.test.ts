import { describe, expect, it } from "vitest";
import { NATURAL_WIDTH } from "./windowFit";

describe("window width policy", () => {
  it("is a fixed constant, never derived from a live DOM measurement", () => {
    // Regression guard for a real bug: measuring `.app`'s offsetWidth picked
    // up the width its own overflow-fallback scrollbar was stealing,
    // producing a feedback loop that walked the window down to the floor in
    // 17px steps (720 -> 703 -> 686 -> ... observed on the packaged app).
    // Width must stay a constant the DOM cannot perturb.
    expect(NATURAL_WIDTH).toBe(720);
    expect(Number.isFinite(NATURAL_WIDTH)).toBe(true);
  });
});
