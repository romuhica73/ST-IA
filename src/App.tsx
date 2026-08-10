import { useState } from "react";
import { MediaDropZone } from "./features/media-selection/MediaDropZone";
import { SelectedMedia } from "./features/media-selection/SelectedMedia";
import { useMediaSelection } from "./features/media-selection/useMediaSelection";
import type { MediaInfo, OutputSelection } from "./features/media-selection/types";
import { TranscriptionProgress } from "./features/transcription/TranscriptionProgress";
import { TranscriptionSuccess } from "./features/transcription/TranscriptionSuccess";
import { TranscriptionFailure } from "./features/transcription/TranscriptionFailure";
import { useTranscription } from "./features/transcription/useTranscription";
import { ModelRequired } from "./features/model-manager/ModelRequired";
import { ModelDownloading } from "./features/model-manager/ModelDownloading";
import { ModelVerifying } from "./features/model-manager/ModelVerifying";
import { ModelCorrupted } from "./features/model-manager/ModelCorrupted";
import { useModelManager } from "./features/model-manager/useModelManager";
import "./styles/App.css";

function App() {
  const { status: modelStatus, manifest, install } = useModelManager();
  const [bypassModelGate, setBypassModelGate] = useState(false);

  const { state: mediaState, selectViaDialog, reset: resetMedia } = useMediaSelection();
  const { status: jobStatus, start, cancel: cancelJob, reset: resetJob } = useTranscription();
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

  function handleInstallModelFromFailure() {
    resetJob();
    setBypassModelGate(false);
    void install();
  }

  const isJobActive =
    jobStatus.status === "preparingAudio" ||
    jobStatus.status === "transcribing" ||
    jobStatus.status === "writingOutputs" ||
    jobStatus.status === "cancelling";

  const displayFileName =
    jobMedia?.fileName ?? (mediaState.status === "selected" ? mediaState.media.fileName : "");

  // Brief startup check — avoid flashing the wrong screen before the first
  // real answer arrives.
  if (modelStatus === null) {
    return <main className="app" />;
  }

  const modelReady = modelStatus.status === "ready";

  if (!modelReady && !bypassModelGate && !isJobActive) {
    if (modelStatus.status === "downloading") {
      return (
        <main className="app">
          <ModelDownloading
            downloadedBytes={modelStatus.downloadedBytes}
            totalBytes={modelStatus.totalBytes}
            progress={modelStatus.progress}
          />
        </main>
      );
    }
    if (modelStatus.status === "verifying") {
      return (
        <main className="app">
          <ModelVerifying />
        </main>
      );
    }
    if (modelStatus.status === "corrupted") {
      return (
        <main className="app">
          <ModelCorrupted onReinstall={install} />
        </main>
      );
    }
    // missing or failed
    return (
      <main className="app">
        <ModelRequired
          manifest={manifest}
          errorMessage={modelStatus.status === "failed" ? modelStatus.message : null}
          onDownload={install}
          onLater={() => setBypassModelGate(true)}
        />
      </main>
    );
  }

  return (
    <main className="app">
      {isJobActive ? (
        <TranscriptionProgress
          fileName={displayFileName}
          status={jobStatus}
          onCancel={() => void cancelJob()}
        />
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
          onInstallModel={handleInstallModelFromFailure}
        />
      ) : mediaState.status === "selected" ? (
        <SelectedMedia
          media={mediaState.media}
          modelReady={modelReady}
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
