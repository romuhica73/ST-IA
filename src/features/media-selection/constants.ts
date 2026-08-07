export const SUPPORTED_EXTENSIONS = [
  "mp4",
  "mov",
  "wav",
  "mp3",
  "m4a",
  "flac",
] as const;

export const SUPPORTED_EXTENSIONS_LABEL = SUPPORTED_EXTENSIONS.map(
  (ext) => `.${ext}`,
).join(", ");

export const MULTIPLE_FILES_ERROR_MESSAGE =
  "Veuillez sélectionner un seul fichier.";
