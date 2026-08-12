import { useState } from "react";
import { useTranslation } from "react-i18next";
import { MediaDropZone } from "./features/media-selection/MediaDropZone";
import { SelectedMedia } from "./features/media-selection/SelectedMedia";
import { useMediaSelection } from "./features/media-selection/useMediaSelection";
import type { MediaInfo } from "./features/media-selection/types";
import type { OutputSelection } from "./features/media-selection/outputs";
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
import { useSplashHandoff } from "./features/startup/useSplashHandoff";
import { SettingsPanel } from "./features/settings/SettingsPanel";
import { ThemeQuickAction } from "./features/settings/ThemeQuickAction";
import { GearIcon } from "./features/settings/icons";
import "./styles/App.css";

function App() {
  const { t } = useTranslation();
  const { settings, setTheme, setMotion, setLanguage } = useSettings();
  useApplySettings(settings);
  const [showSettings, setShowSettings] = useState(false);

  const { state: mediaState, selectViaDialog, reset: resetMedia } = useMediaSelection();
  // Verifying the 3.1 GB translation model costs seconds of I/O, so it is
  // never paid during startup. It is paid once a media file is in hand (the
  // answer is about to be needed) or once the AI models panel is open (the
  // user is explicitly asking what is installed) — neither is on the launch
  // path.
  const translationCheckEnabled = mediaState.status === "selected" || showSettings;

  const { status: modelStatus, manifest, install } = useModelManager("transcription");
  // The translation model is tracked independently: it is only needed if the
  // user asks for English output, and it must never gate an ordinary French
  // job behind a 3.1 GB download.
  const {
    status: translationStatus,
    manifest: translationManifest,
    install: installTranslation,
  } = useModelManager("translation", translationCheckEnabled);
  const [bypassModelGate, setBypassModelGate] = useState(false);

  // One half of the splash handover — see useSplashHandoff for why model
  // status is the readiness signal. The splash's own animation end is the
  // other half; the cut happens when both have arrived.
  useSplashHandoff(modelStatus !== null);

  const { status: jobStatus, start, cancel: cancelJob, reset: resetJob } = useTranscription();
  const [jobMedia, setJobMedia] = useState<MediaInfo | null>(null);
  const [lastOutputs, setLastOutputs] = useState<OutputSelection | null>(null);

  function handleGenerate(outputs: OutputSelection) {
    if (mediaState.status !== "selected") return;
    setJobMedia(mediaState.media);
    setLastOutputs(outputs);
    void start(toStartInput(mediaState.media.path, outputs));
  }

  function handleRetry() {
    // Retry re-runs the *same* request, including both versions — a
    // bilingual job that failed halfway published nothing, so there is
    // nothing partial to resume around.
    if (!jobMedia || !lastOutputs) return;
    void start(toStartInput(jobMedia.path, lastOutputs));
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

  function handleInstallTranslationModelFromFailure() {
    resetJob();
    void installTranslation();
  }

  function handleInstallModelFromFailure() {
    resetJob();
    setBypassModelGate(false);
    void install();
  }

  const bilingual = lastOutputs?.languages.french === true && lastOutputs?.languages.english === true;

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
          bilingual={bilingual === true}
          onCancel={() => void cancelJob()}
        />
      );
    }
    if (jobStatus.status === "completed") {
      return (
        <TranscriptionSuccess
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
          onInstallTranslationModel={handleInstallTranslationModelFromFailure}
        />
      );
    }
    if (mediaState.status === "selected") {
      return (
        <SelectedMedia
          media={mediaState.media}
          modelReady={modelReady}
          translationModelReady={translationStatus?.status === "ready"}
          translationModelSize={translationManifest?.expectedSize ?? null}
          translationModelBusy={
            translationStatus?.status === "downloading" ||
            translationStatus?.status === "verifying"
          }
          onDownloadTranslationModel={installTranslation}
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
        <ThemeQuickAction theme={settings.theme} onChange={setTheme} />
        <button
          type="button"
          className="app-header__button"
          onClick={() => setShowSettings(true)}
          aria-label={t("settings.open")}
          title={t("settings.open")}
        >
          <GearIcon />
        </button>
      </div>
      {showSettings ? (
        <SettingsPanel
          settings={settings}
          modelStatuses={{ transcription: modelStatus, translation: translationStatus }}
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

/** The wire shape Rust expects. Kept in one place so the initial launch and
 * the retry path can never disagree about what was requested. */
function toStartInput(mediaPath: string, outputs: OutputSelection) {
  return {
    mediaPath,
    outputFrench: outputs.languages.french,
    outputEnglish: outputs.languages.english,
    outputSrt: outputs.formats.srt,
    outputTxt: outputs.formats.txt,
  };
}

export default App;
