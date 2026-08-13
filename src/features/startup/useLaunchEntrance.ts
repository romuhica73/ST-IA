import { useEffect, useState } from "react";

/** Module-level, deliberately: this must be true for the rest of the process
 * lifetime once the entrance has played, so that returning Home, cancelling
 * a job or closing Settings never replays it. React state alone would reset
 * with the component. */
let entrancePlayed = false;

/** For tests only — the flag is process-wide by design. */
export function resetLaunchEntranceForTests() {
  entrancePlayed = false;
}

/**
 * Whether the launch entrance animation should play right now.
 *
 * Returns `true` only on the first render of the first screen after the app
 * starts, and `false` forever after. The distinction the product needs is
 * between *the application appearing* — which happens once, straight after
 * the splash cut, and deserves a short staggered entrance — and *navigating
 * between states*, which happens constantly and must not re-animate the
 * whole screen every time.
 *
 * The flag flips on the first effect rather than on the first render, so the
 * initial paint still carries the entrance class and the animation is not
 * cancelled halfway by a re-render.
 */
export function useLaunchEntrance(): boolean {
  const [isLaunch] = useState(() => !entrancePlayed);

  useEffect(() => {
    entrancePlayed = true;
  }, []);

  return isLaunch;
}
