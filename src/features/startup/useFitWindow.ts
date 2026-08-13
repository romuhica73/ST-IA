import { useEffect, useRef } from "react";
import type { RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  isMeasurable,
  isSignificantChange,
  NATURAL_WIDTH,
  SETTLE_MS,
  type Size,
} from "./windowFit";

/**
 * Keeps the native window sized to `ref`'s real content, growing or
 * shrinking it as the app's content does (see ADR-011).
 *
 * The element measured must NOT itself be height-constrained by the window
 * (no `100vh`, no `height: 100%`) — its `offsetWidth`/`offsetHeight` need to
 * reflect what the content actually needs, not what the window currently
 * happens to be. `.app` in App.tsx is that element; `.app-viewport` around it
 * is the fixed-to-100vh scroll fallback for the rare case content exceeds
 * the screen's maximum allowed size.
 *
 * Every call goes through `fit_window`, which does the actual clamping
 * against the monitor's usable area — this hook only decides *when* a
 * measurement is worth sending, via a small settle window and a
 * change-magnitude threshold, so a disclosure animating open or a stage
 * transition doesn't fire a resize per frame.
 */
export function useFitWindow(ref: RefObject<HTMLElement | null>) {
  const lastSent = useRef<Size | null>(null);
  const settleTimer = useRef<number | null>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const observer = new ResizeObserver(() => {
      if (settleTimer.current !== null) {
        window.clearTimeout(settleTimer.current);
      }
      settleTimer.current = window.setTimeout(() => {
        settleTimer.current = null;
        // Height is measured live (offsetHeight, not the ResizeObserver
        // entry's own contentRect — `.app` includes its own padding and
        // border, and the window needs the full box). Width is fixed at
        // NATURAL_WIDTH rather than measured — see the comment on that
        // constant for why a live width measurement here is a feedback loop
        // with the scroll fallback's own scrollbar.
        const size: Size = { width: NATURAL_WIDTH, height: el.offsetHeight };
        if (!isMeasurable(size)) return;
        if (!isSignificantChange(lastSent.current, size)) return;
        lastSent.current = size;
        void invoke("fit_window", { width: size.width, height: size.height }).catch((error) => {
          // Not recoverable and not worth surfacing: the window simply
          // keeps whatever size it already had.
          console.error("Failed to fit window to content:", error);
        });
      }, SETTLE_MS);
    });

    observer.observe(el);
    return () => {
      observer.disconnect();
      if (settleTimer.current !== null) window.clearTimeout(settleTimer.current);
    };
  }, [ref]);
}
