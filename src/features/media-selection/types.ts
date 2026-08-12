export type MediaKind = "video" | "audio";

export interface MediaInfo {
  path: string;
  fileName: string;
  extension: string;
  sizeBytes: number;
  kind: MediaKind;
}

export type MediaErrorCode = "notFound" | "unsupported" | "empty";

export interface MediaError {
  code: MediaErrorCode;
  message: string;
}

/** "multipleFiles" is a frontend-only condition (no backend round trip);
 * "unknown" covers a malformed/unexpected error payload (should not happen
 * in practice — Tauri IPC always round-trips a well-formed MediaError, but
 * the fallback must still resolve to a real translated message rather than
 * guessing at one of the real codes). Everything else mirrors MediaErrorCode
 * from the Rust side. */
export type MediaSelectionErrorCode = MediaErrorCode | "multipleFiles" | "unknown";

export type MediaSelectionState =
  | { status: "idle" }
  | { status: "dragging" }
  | { status: "selected"; media: MediaInfo }
  | { status: "error"; code: MediaSelectionErrorCode };

export type { OutputSelection } from "./outputs";
