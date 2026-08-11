// GearIcon: filled cog silhouette (solid ring + 8 overlapping teeth + a
// punched-out hole) rather than a stroked circle-plus-rays — the earlier
// ray version was visually indistinguishable from a sun/brightness icon at
// this size (confirmed both by a user report and by rendering both at
// actual icon size before choosing this one). Filled, not stroked, because
// thin gear teeth do not read cleanly at 18px; FileIcon already sets the
// precedent of mixing a filled glyph into this otherwise stroke-based
// family for exactly this legibility reason.
export function GearIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <g fill="currentColor">
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(45 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(90 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(135 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(180 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(225 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(270 12 12)" />
        <rect x="10.7" y="2.6" width="2.6" height="4.2" rx="0.5" transform="rotate(315 12 12)" />
        <circle cx="12" cy="12" r="6.2" />
      </g>
      <circle cx="12" cy="12" r="3" fill="var(--bg)" />
    </svg>
  );
}

export function CloseIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M6 6l12 12M18 6 6 18"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function SunIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="12" cy="12" r="4.2" fill="currentColor" />
      <path
        d="M12 2.5v2.4M12 19.1v2.4M21.5 12h-2.4M4.9 12H2.5M18.5 5.5l-1.7 1.7M7.2 16.8l-1.7 1.7M18.5 18.5l-1.7-1.7M7.2 7.2 5.5 5.5"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function MoonIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"
        fill="currentColor"
      />
    </svg>
  );
}

export function SystemDisplayIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <rect
        x="3"
        y="4.5"
        width="18"
        height="12"
        rx="1.6"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinejoin="round"
      />
      <path
        d="M9 20h6M12 16.5V20"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}
