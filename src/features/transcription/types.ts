export type TranscribingPhase = "loadingModel" | "processing";

/** Which pass is running. A French-only job only ever reports `original`,
 * so its progress UI is identical to the pre-bilingual behaviour. */
export type TranscribingVariant = "original" | "englishTranslation";

export type OutputKind = "srt" | "txt";

/** A version of the transcript — not the interface language, and not the
 * spoken language of the media. See ADR-010. */
export type OutputLanguage = "french" | "english";

export interface OutputFile {
  kind: OutputKind;
  language: OutputLanguage;
  fileName: string;
  path: string;
  sizeBytes: number;
}

export type TranscriptionErrorCode =
  | "alreadyRunning"
  | "modelMissing"
  | "translationModelMissing"
  | "noOutputSelected"
  | "noLanguageSelected"
  | "audioPreparationFailed"
  | "noAudioTrack"
  | "transcriptionFailed"
  | "translationFailed"
  | "writeFailed"
  | "insufficientDiskSpace";

export interface TranscriptionError {
  code: TranscriptionErrorCode;
  message: string;
}

export type JobStatus =
  | { status: "idle" }
  | { status: "preparingAudio" }
  | {
      status: "transcribing";
      variant: TranscribingVariant;
      phase: TranscribingPhase;
      progress: number | null;
      /** How far into the audio this pass has got, and how long the audio is.
       * Both measured — the end timestamp Whisper printed for the segment it
       * just finished, and the WAV header. `null` until the first segment is
       * decoded: "not known yet" is not the same as "at 0:00". */
      processedAudioSeconds: number | null;
      totalAudioSeconds: number | null;
    }
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
  outputFrench: boolean;
  outputEnglish: boolean;
  outputSrt: boolean;
  outputTxt: boolean;
}
