import { useState } from "react";
import { useTranslation } from "react-i18next";
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
import { useSettings } from "./features/settings/useSettings";
import { useApplySettings } from "./features/settings/useApplySettings";
import { SettingsPanel } from "./features/settings/SettingsPanel";
import { GearIcon } from "./features/settings/icons";
import "./styles/App.css";

function App() {
  const { t } = useTranslation();
  const { settings, setTheme, setMotion, setLanguage } = useSettings();
  useApplySettings(settings);
  const [showSettings, setShowSettings] = useState(false);

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
  // real answer arrives. Settings aren't reachable yet either; this state
  // is expected to last a handful of milliseconds.
  if (modelStatus === null) {
    return <main className="app" />;
  }

  const modelReady = modelStatus.status === "ready";

  function renderScreen() {
    if (modelStatus === null) return null; // narrowed above; keeps TS happy below

    if (!modelReady && !bypassModelGate && !isJobActive) {
      if (modelStatus.status === "downloading") {
        return (
          <ModelDownloading
            downloadedBytes={modelStatus.downloadedBytes}
            totalBytes={modelStatus.totalBytes}
            progress={modelStatus.progress}
          />
        );
      }
      if (modelStatus.status === "verifying") {
        return <ModelVerifying />;
      }
      if (modelStatus.status === "corrupted") {
        return <ModelCorrupted onReinstall={install} />;
      }
      // missing or failed
      return (
        <ModelRequired
          manifest={manifest}
          errorCode={modelStatus.status === "failed" ? modelStatus.code : null}
          onDownload={install}
          onLater={() => setBypassModelGate(true)}
        />
      );
    }

    if (isJobActive) {
      return (
        <TranscriptionProgress
          fileName={displayFileName}
          status={jobStatus}
          onCancel={() => void cancelJob()}
        />
      );
    }
    if (jobStatus.status === "completed") {
      return (
        <TranscriptionSuccess
          outputDir={jobStatus.outputDir}
          files={jobStatus.files}
          transcriptText={jobStatus.transcriptText}
          onNewFile={handleNewFile}
        />
      );
    }
    if (jobStatus.status === "failed") {
      return (
        <TranscriptionFailure
          code={jobStatus.code}
          fileName={displayFileName}
          onChooseAnother={handleChooseAnother}
          onRetry={handleRetry}
          onInstallModel={handleInstallModelFromFailure}
        />
      );
    }
    if (mediaState.status === "selected") {
      return (
        <SelectedMedia
          media={mediaState.media}
          modelReady={modelReady}
          onChangeFile={selectViaDialog}
          onGenerate={handleGenerate}
        />
      );
    }
    return (
      <MediaDropZone
        isDragging={mediaState.status === "dragging"}
        errorCode={mediaState.status === "error" ? mediaState.code : null}
        onSelectClick={selectViaDialog}
      />
    );
  }

  return (
    <main className="app">
      <div className="app-header">
        <button
          type="button"
          className="app-header__gear"
          onClick={() => setShowSettings(true)}
          aria-label={t("settings.open")}
        >
          <GearIcon />
        </button>
      </div>
      {showSettings ? (
        <SettingsPanel
          settings={settings}
          onThemeChange={setTheme}
          onMotionChange={setMotion}
          onLanguageChange={setLanguage}
          onClose={() => setShowSettings(false)}
        />
      ) : (
        renderScreen()
      )}
    </main>
  );
}

export default App;
