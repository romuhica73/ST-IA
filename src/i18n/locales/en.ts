/**
 * English catalogue. Must have exactly the same key shape as `fr.ts`
 * (checked by scripts/check-i18n-parity.mjs and by a Vitest test); values
 * are never empty.
 */
const en = {
  common: {
    localProcessing: "100% local processing",
  },
  drop: {
    title: "Drop your media here",
    selectFile: "or select a file",
  },
  media: {
    changeFile: "Change file",
    generate: "Generate subtitles",
    generateDisabledTitle: "The transcription model must be installed first.",
    outputs: "Outputs",
    outputsError: "Select at least one output format.",
    srt: "SRT",
    txt: "TXT",
  },
  transcription: {
    language: "Transcription language",
    languageFrench: "French",
    cancel: "Cancel",
    cancelling: "Cancelling…",
  },
  progress: {
    label: "Progress",
    audioPreparation: "Preparing audio",
    modelLoading: "Loading model",
    transcription: "Transcribing",
    outputGeneration: "Generating files",
    stateDone: "Done",
    stateActive: "In progress…",
    statePending: "Pending",
    cancellingText: "Stopping the current task…",
  },
  success: {
    title: "Subtitles generated",
    subtitle_one: "{{count}} file created successfully.",
    subtitle_other: "{{count}} files created successfully.",
    openFolder: "Open folder",
    openFolderError: "Could not open the folder.",
    copyTranscript: "Copy transcript",
    copied: "Copied!",
    newFile: "New file",
    readyForDavinci: "Ready for DaVinci Resolve",
  },
  error: {
    chooseAnother: "Choose another file",
    installModel: "Install model",
    retry: "Retry",
    technicalDetails: "Technical details",
    fieldError: "Error",
    fieldCode: "Code",
    fieldFile: "File",
    codes: {
      modelMissing: {
        title: "Model not installed",
        message: "The transcription model is not installed yet.",
      },
      noAudioTrack: {
        title: "This file could not be processed",
        message: "No audio track detected. Check that the video contains audio or choose another file.",
      },
      audioPreparationFailed: {
        title: "This file could not be processed",
        message: "The audio could not be prepared from this file.",
      },
      transcriptionFailed: {
        title: "Transcription failed",
        message: "Transcription failed.",
      },
      writeFailed: {
        title: "Could not save the files",
        message: "The subtitle files could not be saved.",
      },
      noOutputSelected: {
        title: "No output selected",
        message: "Select at least one output format (SRT or TXT).",
      },
      alreadyRunning: {
        title: "Transcription already running",
        message: "A transcription is already running.",
      },
      insufficientDiskSpace: {
        title: "Not enough disk space",
        message: "There is not enough free disk space to process this file.",
      },
    },
    mediaCodes: {
      notFound: {
        title: "File not found",
        message: "The selected file could not be found.",
      },
      unsupported: {
        title: "Unsupported format",
        message: "This file format is not supported.",
      },
      empty: {
        title: "Empty file",
        message: "The selected file is empty.",
      },
      multipleFiles: {
        title: "Multiple files",
        message: "Please select a single file.",
      },
      unknown: {
        title: "Error",
        message: "An unexpected error occurred.",
      },
    },
    modelCodes: {
      networkError: {
        title: "Network error",
        message: "A network error occurred while downloading.",
      },
      writeError: {
        title: "Write error",
        message: "The model could not be saved to disk.",
      },
      integrityMismatch: {
        title: "Invalid file",
        message: "The downloaded file is invalid.",
      },
    },
  },
  model: {
    requiredTitle: "The local model is not installed yet",
    requiredSubtitle: "Download the model to use the app offline.",
    name: "Whisper — large-v3-turbo",
    sizeApprox: "~{{size}}",
    storageInfo: "The model will be stored on your computer.",
    later: "Later",
    download: "Download",
    downloading: "Downloading model",
    downloadedOf: "{{downloaded}} / {{total}}",
    verifying: "Verifying model",
    corruptedTitle: "Model damaged",
    corruptedSubtitle: "The transcription model is damaged and must be reinstalled.",
    reinstall: "Reinstall model",
  },
  settings: {
    title: "Settings",
    open: "Settings",
    close: "Close",
    general: "General",
    appearance: "Appearance",
    accessibility: "Accessibility",
    language: "Language",
    about: "About",
    themeLabel: "Theme",
    themeSystem: "System",
    themeLight: "Light",
    themeDark: "Dark",
    motionLabel: "Reduce motion",
    motionSystem: "Follow system",
    motionOn: "On",
    motionOff: "Off",
    languageLabel: "App language",
    languageSystem: "System",
    languageFrench: "Français",
    languageEnglish: "English",
    themeQuickSystem: "Theme: System. Switch to light mode.",
    themeQuickLight: "Theme: Light. Switch to dark mode.",
    themeQuickDark: "Theme: Dark. Return to system theme.",
  },
  about: {
    title: "About",
    tagline: "Local media transcription into subtitles.",
    version: "Version {{version}}",
    localProcessing: "Local processing — your media never leaves your Mac.",
    licenses: "Third-party licenses",
  },
} as const;

export default en;
