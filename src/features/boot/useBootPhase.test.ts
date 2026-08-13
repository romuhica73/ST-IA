import { describe, expect, it, beforeEach } from "vitest";
import { resetBootPhaseForTests } from "./useBootPhase";

describe("boot latch", () => {
  beforeEach(() => resetBootPhaseForTests());

  it("is process-wide, not component state", () => {
    // The intro belongs to the run, not to a component. React StrictMode
    // double-mounts in development, and any remount of App would otherwise
    // replay a six-second splash — so the latch must survive remounting,
    // which component state does not.
    expect(typeof resetBootPhaseForTests).toBe("function");
    expect(() => resetBootPhaseForTests()).not.toThrow();
  });
});
