import { useState } from "react";

/** Where the application is in its startup sequence.
 *
 * `splash` — the intro layer is on screen. The interface is already mounted
 *            and laid out behind it, so nothing is waiting on this.
 * `ready`  — the layer is gone. On the transition into this phase, and only
 *            then, the launch entrance plays.
 *
 * Deliberately two states rather than a general-purpose machine: there is
 * one linear path with one event on it, and naming more phases than the app
 * actually has would be describing a design that does not exist. */
export type BootPhase = "splash" | "ready";

/** Module-scoped on purpose.
 *
 * The intro belongs to the process, not to a component. React StrictMode
 * mounts twice in development, and any remount of `App` would otherwise
 * replay a six-second splash. This latch makes "has the app already booted"
 * a fact about the run. */
let booted = false;

/** For tests only — the latch is process-wide by design. */
export function resetBootPhaseForTests() {
  booted = false;
}

export interface BootState {
  phase: BootPhase;
  /** True for the render in which the splash was just removed, so the launch
   * entrance can play exactly once. Never true again for the rest of the
   * session, including on returning Home or closing Settings. */
  isEntering: boolean;
  /** Called by the splash layer when its cycle has played out. */
  finishBoot: () => void;
}

export function useBootPhase(): BootState {
  // A second mount after boot skips the intro entirely rather than replaying
  // it — the app is already running, and the user has already seen it.
  const [phase, setPhase] = useState<BootPhase>(() => (booted ? "ready" : "splash"));
  const [isEntering, setIsEntering] = useState(false);

  function finishBoot() {
    if (booted) return;
    booted = true;
    setPhase("ready");
    setIsEntering(true);
  }

  return { phase, isEntering, finishBoot };
}
