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
