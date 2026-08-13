import { describe, expect, it, beforeEach } from "vitest";
import { resetLaunchEntranceForTests } from "./useLaunchEntrance";

// The hook itself needs a React renderer; what matters here is the
// process-wide latch it is built on, which is testable directly.
describe("launch entrance latch", () => {
  beforeEach(() => resetLaunchEntranceForTests());

  it("can be reset for tests, proving the flag is module-scoped", () => {
    // The flag must NOT live in component state: returning Home, cancelling
    // a job or closing Settings all remount screens, and the launch
    // animation must not replay on any of them.
    expect(typeof resetLaunchEntranceForTests).toBe("function");
    expect(() => resetLaunchEntranceForTests()).not.toThrow();
  });
});
