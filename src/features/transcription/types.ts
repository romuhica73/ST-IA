export type TranscribingPhase = "loadingModel" | "processing";

export type OutputKind = "srt" | "txt";

export interface OutputFile {
  kind: OutputKind;
  fileName: string;
  path: string;
  sizeBytes: number;
}

export type TranscriptionErrorCode =
  | "alreadyRunning"
  | "modelMissing"
  | "noOutputSelected"
  | "audioPreparationFailed"
  | "noAudioTrack"
  | "transcriptionFailed"
  | "writeFailed"
  | "insufficientDiskSpace";

export interface TranscriptionError {
  code: TranscriptionErrorCode;
  message: string;
}

export type JobStatus =
  | { status: "idle" }
  | { status: "preparingAudio" }
  | { status: "transcribing"; phase: TranscribingPhase; progress: number | null }
  | { status: "writingOutputs" }
  | {
      status: "completed";
      outputDir: string;
      files: OutputFile[];
      transcriptText: string | null;
    }
  | { status: "failed"; code: TranscriptionErrorCode; message: string }
  | { status: "cancelling" }
  | { status: "cancelled" };

export interface StartTranscriptionInput {
  mediaPath: string;
  outputSrt: boolean;
  outputTxt: boolean;
}
