export function CloudUploadIcon() {
  return (
    <svg
      className="icon icon--xl"
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M7 18a4.5 4.5 0 0 1-.4-8.98 5.5 5.5 0 0 1 10.6-1.98A4.5 4.5 0 0 1 17 18H7Z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M12 9.5v6.2m0 0-2.4-2.4M12 15.7l2.4-2.4"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function LockIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <rect
        x="5"
        y="11"
        width="14"
        height="9"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <path
        d="M8 11V8a4 4 0 0 1 8 0v3"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function GlobeIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="1.5" />
      <path
        d="M4 12h16M12 4c2.2 2.2 3.3 5 3.3 8s-1.1 5.8-3.3 8c-2.2-2.2-3.3-5-3.3-8S9.8 6.2 12 4Z"
        stroke="currentColor"
        strokeWidth="1.5"
      />
    </svg>
  );
}

export function ChevronDownIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M6 9l6 6 6-6"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function BoltIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M13 3 5 13h5l-1 8 8-10h-5l1-8Z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function TargetIcon() {
  return (
    <svg className="icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="12" cy="12" r="3.5" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

export function TypeIcon() {
  return (
    <svg className="icon icon--sm" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <rect
        x="3"
        y="6"
        width="18"
        height="12"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <path d="M7 10h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}

export function SizeIcon() {
  return (
    <svg className="icon icon--sm" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <rect
        x="3"
        y="5"
        width="18"
        height="14"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <circle cx="8" cy="15" r="1" fill="currentColor" />
      <path d="M3 11h18" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

export function FileIcon({ kind }: { kind: "video" | "audio" }) {
  return (
    <svg width="48" height="56" viewBox="0 0 48 56" fill="none" aria-hidden="true">
      <path
        d="M6 4h22l14 14v34a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z"
        fill="var(--file-icon-bg)"
        stroke="var(--file-icon-border)"
      />
      <path
        d="M28 4v12a2 2 0 0 0 2 2h12"
        fill="var(--file-icon-fold)"
        stroke="var(--file-icon-border)"
      />
      <rect x="10" y="30" width="24" height="20" rx="6" fill="var(--accent)" />
      {kind === "video" ? (
        <path d="M19 34.5v11l10-5.5-10-5.5Z" fill="white" />
      ) : (
        <g fill="white">
          <rect x="16.5" y="37" width="3" height="7" rx="1.5" />
          <rect x="22.5" y="33" width="3" height="15" rx="1.5" />
          <rect x="28.5" y="39" width="3" height="5" rx="1.5" />
        </g>
      )}
    </svg>
  );
}
