import { useState } from "react";
import { MediaDropZone } from "./features/media-selection/MediaDropZone";
import { SelectedMedia } from "./features/media-selection/SelectedMedia";
import { useMediaSelection } from "./features/media-selection/useMediaSelection";
import type { MediaInfo, OutputSelection } from "./features/media-selection/types";
import { TranscriptionProgress } from "./features/transcription/TranscriptionProgress";
import { TranscriptionSuccess } from "./features/transcription/TranscriptionSuccess";
import { TranscriptionFailure } from "./features/transcription/TranscriptionFailure";
import { useTranscription } from "./features/transcription/useTranscription";
import "./styles/App.css";

function App() {
  const { state: mediaState, selectViaDialog, reset: resetMedia } = useMediaSelection();
  const { status: jobStatus, start, reset: resetJob } = useTranscription();
  const [jobMedia, setJobMedia] = useState<MediaInfo | null>(null);
  const [lastOutputs, setLastOutputs] = useState<OutputSelection | null>(null);

  function handleGenerate(outputs: OutputSelection) {
    if (mediaState.status !== "selected") return;
    setJobMedia(mediaState.media);
    setLastOutputs(outputs);
    void start({
      mediaPath: mediaState.media.path,
      outputSrt: outputs.srt,
      outputTxt: outputs.txt,
    });
  }

  function handleRetry() {
    if (!jobMedia || !lastOutputs) return;
    void start({
      mediaPath: jobMedia.path,
      outputSrt: lastOutputs.srt,
      outputTxt: lastOutputs.txt,
    });
  }

  function handleNewFile() {
    resetJob();
    resetMedia();
    setJobMedia(null);
  }

  function handleChooseAnother() {
    resetJob();
    resetMedia();
    setJobMedia(null);
    void selectViaDialog();
  }

  const isJobActive =
    jobStatus.status === "preparingAudio" ||
    jobStatus.status === "transcribing" ||
    jobStatus.status === "writingOutputs";

  const displayFileName =
    jobMedia?.fileName ?? (mediaState.status === "selected" ? mediaState.media.fileName : "");

  return (
    <main className="app">
      {isJobActive ? (
        <TranscriptionProgress fileName={displayFileName} status={jobStatus} />
      ) : jobStatus.status === "completed" ? (
        <TranscriptionSuccess
          outputDir={jobStatus.outputDir}
          files={jobStatus.files}
          transcriptText={jobStatus.transcriptText}
          onNewFile={handleNewFile}
        />
      ) : jobStatus.status === "failed" ? (
        <TranscriptionFailure
          code={jobStatus.code}
          message={jobStatus.message}
          fileName={displayFileName}
          onChooseAnother={handleChooseAnother}
          onRetry={handleRetry}
        />
      ) : mediaState.status === "selected" ? (
        <SelectedMedia
          media={mediaState.media}
          onChangeFile={selectViaDialog}
          onGenerate={handleGenerate}
        />
      ) : (
        <MediaDropZone
          isDragging={mediaState.status === "dragging"}
          errorMessage={mediaState.status === "error" ? mediaState.message : null}
          onSelectClick={selectViaDialog}
        />
      )}
    </main>
  );
}

export default App;
