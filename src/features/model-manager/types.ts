export interface ModelManifest {
  id: string;
  fileName: string;
  expectedSize: number;
}

export type ModelErrorCode = "networkError" | "writeError" | "integrityMismatch";

export interface ModelError {
  code: ModelErrorCode;
  message: string;
}

export type ModelStatus =
  | { status: "missing" }
  | {
      status: "downloading";
      downloadedBytes: number;
      totalBytes: number | null;
      progress: number | null;
    }
  | { status: "verifying" }
  | { status: "ready" }
  | { status: "corrupted" }
  | { status: "failed"; code: ModelErrorCode; message: string };
