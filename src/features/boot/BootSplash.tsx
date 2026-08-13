import { useEffect, useRef } from "react";
import "./bootSplash.css";

/** The container animation's name, matched by `animationend` so the
 * decorative child animations do not end the boot phase early. */
const CYCLE_ANIMATION = "boot-splash-cycle";

/** Mirrors `--cycle-duration` in bootSplash.css. Only used by the fallback
 * timer below — the animation itself remains the source of truth. */
const CYCLE_MS = 6000;
const FALLBACK_SLACK_MS = 1500;

interface BootSplashProps {
  /** Called once when the cycle has played out. The layer is removed by the
   * parent in response; it does not unmount itself. */
  onFinished: () => void;
}

/**
 * The ST-IA intro, rendered inside the application window rather than in a
 * window of its own (see ADR-009).
 *
 * It is an absolutely positioned layer over the app's own content, which is
 * already mounted and laid out behind it. That is the entire point of the
 * design: when this layer goes, the interface underneath is finished — there
 * is no second window to swap to, no geometry to match, and nothing left to
 * lay out.
 *
 * Purely decorative and inert: `aria-hidden`, no focusable content, and it
 * covers the webview so nothing behind it can be clicked while it shows. The
 * native title bar sits outside the webview and stays visible throughout.
 */
export function BootSplash({ onFinished }: BootSplashProps) {
  const reported = useRef(false);

  function finish() {
    if (reported.current) return;
    reported.current = true;
    onFinished();
  }

  // Belt and braces behind `animationend`: if the compositor never delivers
  // it (a suspended window, a repaint the page never gets), the app must
  // still become usable. Cleared on unmount so it cannot fire afterwards.
  useEffect(() => {
    const id = window.setTimeout(finish, CYCLE_MS + FALLBACK_SLACK_MS);
    return () => window.clearTimeout(id);
    // `finish` is stable for the lifetime of this component (guarded by a
    // ref), so re-running this effect would only restart the fallback.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div
      className="boot-splash"
      aria-hidden="true"
      onAnimationEnd={(event) => {
        if (event.animationName === CYCLE_ANIMATION) finish();
      }}
    >
      <div className="boot-splash__mark">
        <div className="wave">
          {/* Nine bars forming a symmetric envelope; heights and stagger
              live in CSS, keyed by :nth-child. */}
          {Array.from({ length: 9 }, (_, i) => (
            <span className="wave__bar" key={i} />
          ))}
        </div>
        <div className="lines">
          <span className="lines__line" />
          <span className="lines__line" />
        </div>
      </div>
      <p className="boot-splash__wordmark">ST-IA</p>
    </div>
  );
}
