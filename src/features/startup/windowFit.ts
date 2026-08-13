/** Pure decision logic for `useFitWindow` — kept separate from the
 * ResizeObserver/timer plumbing so it can be unit tested directly. */

export interface Size {
  width: number;
  height: number;
}

/** The width every screen is visually tuned for, and the only width this app
 * ever requests.
 *
 * Content is measured for height only — never for width. `.app` sits inside
 * `.app-viewport`, whose vertical scrollbar (the fallback for the rare case
 * content exceeds the screen) steals horizontal space from `.app` exactly
 * when it is showing. Trusting a live width measurement there is a feedback
 * loop: the scrollbar narrows the measured width, the window shrinks to
 * match, the narrower window still overflows vertically so the scrollbar
 * stays, and the next measurement is narrower still. Observed running the
 * packaged app: 720 → 703 → 686 → … in 17px steps down to the floor. Nothing
 * in this app's single-column layout legitimately needs more or less than
 * this width, so the fix is to stop asking the DOM and simply not vary it. */
export const NATURAL_WIDTH = 720;

/** Below this many logical pixels of change on either axis, a new
 * measurement is not worth a resize call.
 *
 * Font hinting, sub-pixel layout rounding and the liveness pulse's own
 * micro-reflows all produce noise in the 1-3px range on every render; a
 * resize request for each one would make the window visibly jitter. 6px is
 * comfortably above that noise floor and comfortably below anything a real
 * layout change (a stage appearing, a card expanding) produces. */
export const RESIZE_THRESHOLD_PX = 6;

/** How long a measurement must stay put before it is trusted.
 *
 * A disclosure opening, a stage transition or the success screen mounting
 * all pass through one or more transient layout states before settling —
 * without a settle window, each one would fire its own resize mid-animation.
 * 120ms is long enough to skip past a single reflow, short enough that the
 * resize still reads as an immediate response to the user's action. */
export const SETTLE_MS = 120;

/** Whether a newly measured size differs enough from the last one requested
 * to be worth asking the window to resize for. */
export function isSignificantChange(previous: Size | null, next: Size): boolean {
  if (previous === null) return true;
  return (
    Math.abs(next.width - previous.width) >= RESIZE_THRESHOLD_PX ||
    Math.abs(next.height - previous.height) >= RESIZE_THRESHOLD_PX
  );
}

/** Whether a measurement is usable at all — guards against the brief window
 * where an element hasn't been laid out yet (0×0) or a browser quirk hands
 * back a non-finite value. Never ask the backend to resize to garbage. */
export function isMeasurable(size: Size): boolean {
  return (
    Number.isFinite(size.width) &&
    Number.isFinite(size.height) &&
    size.width > 0 &&
    size.height > 0
  );
}
