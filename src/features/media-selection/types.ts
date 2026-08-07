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

export type MediaSelectionState =
  | { status: "idle" }
  | { status: "dragging" }
  | { status: "selected"; media: MediaInfo }
  | { status: "error"; message: string };

/**
 * Job configuration state — local UI preference only in this mission.
 * Not sent to Rust, not wired to a real transcription pipeline (M2+).
 */
export type TranscriptionMode = "fast" | "precise";

export interface OutputSelection {
  srt: boolean;
  txt: boolean;
}
