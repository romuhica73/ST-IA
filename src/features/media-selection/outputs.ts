import type { OutputLanguage } from "../transcription/types";

/** What the user asked for, on two independent axes: which versions, and in
 * which formats. The number of files produced is the product of the two. */
export interface OutputLanguages {
  french: boolean;
  english: boolean;
}

export interface OutputFormats {
  srt: boolean;
  txt: boolean;
}

export interface OutputSelection {
  languages: OutputLanguages;
  formats: OutputFormats;
}

/** French original selected, English translation not — the historical
 * behaviour, preserved so an existing user's first run after updating
 * produces exactly the files it produced before. */
export const DEFAULT_OUTPUTS: OutputSelection = {
  languages: { french: true, english: false },
  formats: { srt: true, txt: true },
};

export function selectedLanguages(languages: OutputLanguages): OutputLanguage[] {
  const selected: OutputLanguage[] = [];
  if (languages.french) selected.push("french");
  if (languages.english) selected.push("english");
  return selected;
}

export function countFormats(formats: OutputFormats): number {
  return (formats.srt ? 1 : 0) + (formats.txt ? 1 : 0);
}

/** Mirrors `OutputRequest::file_count` in Rust: languages × formats. Drives
 * the launch button's label, so a mismatch would promise the wrong number of
 * files. */
export function fileCount(selection: OutputSelection): number {
  return selectedLanguages(selection.languages).length * countFormats(selection.formats);
}

export function hasLanguage(selection: OutputSelection): boolean {
  return selection.languages.french || selection.languages.english;
}

export function hasFormat(selection: OutputSelection): boolean {
  return countFormats(selection.formats) > 0;
}

/** Whether the job can be launched at all: at least one version *and* at
 * least one format. Rust re-checks both — this only decides the button. */
export function isLaunchable(selection: OutputSelection): boolean {
  return hasLanguage(selection) && hasFormat(selection);
}

/** Whether the English translation model is needed for this selection. */
export function needsTranslationModel(selection: OutputSelection): boolean {
  return selection.languages.english;
}
