/** ST-IA holds two models, never a catalog: the fast French transcription
 * model, and the larger one used only for English translation (which the
 * turbo model cannot do at all — see ADR-008). */
export type ModelKind = "transcription" | "translation";

export interface ModelManifest {
  kind: ModelKind;
  id: string;
  fileName: string;
  expectedSize: number;
}

export type ModelErrorCode = "networkError" | "writeError" | "integrityMismatch";

export interface ModelError {
  code: ModelErrorCode;
  message: string;
}

/** What arrives on `model://event`: a status, plus which model it is about.
 * Without the kind, a 3.1 GB translation download would drive the
 * transcription screen's progress bar. */
export type ModelStatusEvent = ModelStatus & { kind: ModelKind };

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
