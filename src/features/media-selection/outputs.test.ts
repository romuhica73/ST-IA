import { describe, expect, it } from "vitest";
import {
  DEFAULT_OUTPUTS,
  fileCount,
  hasFormat,
  hasLanguage,
  isLaunchable,
  needsTranslationModel,
  selectedLanguages,
} from "./outputs";
import type { OutputSelection } from "./outputs";

const selection = (
  french: boolean,
  english: boolean,
  srt: boolean,
  txt: boolean,
): OutputSelection => ({
  languages: { french, english },
  formats: { srt, txt },
});

describe("default output selection", () => {
  it("keeps the historical behaviour: French original, no translation", () => {
    // A user updating from a previous version must get exactly the files
    // they got before, without touching anything.
    expect(DEFAULT_OUTPUTS.languages).toEqual({ french: true, english: false });
    expect(DEFAULT_OUTPUTS.formats).toEqual({ srt: true, txt: true });
  });

  it("does not require the translation model by default", () => {
    // Otherwise every existing user would be met with a 3.1 GB download.
    expect(needsTranslationModel(DEFAULT_OUTPUTS)).toBe(false);
  });
});

describe("file count", () => {
  it("is languages times formats", () => {
    expect(fileCount(selection(true, false, true, true))).toBe(2);
    expect(fileCount(selection(false, true, true, true))).toBe(2);
    expect(fileCount(selection(true, true, true, true))).toBe(4);
    expect(fileCount(selection(true, true, true, false))).toBe(2);
    expect(fileCount(selection(true, false, true, false))).toBe(1);
  });

  it("is zero when either axis is empty", () => {
    expect(fileCount(selection(false, false, true, true))).toBe(0);
    expect(fileCount(selection(true, true, false, false))).toBe(0);
  });
});

describe("launch validation", () => {
  it("requires at least one version", () => {
    expect(hasLanguage(selection(false, false, true, true))).toBe(false);
    expect(isLaunchable(selection(false, false, true, true))).toBe(false);
  });

  it("requires at least one format", () => {
    expect(hasFormat(selection(true, false, false, false))).toBe(false);
    expect(isLaunchable(selection(true, false, false, false))).toBe(false);
  });

  it("accepts every valid combination", () => {
    for (const [french, english] of [
      [true, false],
      [false, true],
      [true, true],
    ] as const) {
      expect(isLaunchable(selection(french, english, true, false))).toBe(true);
      expect(isLaunchable(selection(french, english, false, true))).toBe(true);
      expect(isLaunchable(selection(french, english, true, true))).toBe(true);
    }
  });
});

describe("language ordering", () => {
  it("puts French before English, matching the pipeline's pass order", () => {
    expect(selectedLanguages({ french: true, english: true })).toEqual(["french", "english"]);
  });

  it("omits what was not selected", () => {
    expect(selectedLanguages({ french: false, english: true })).toEqual(["english"]);
    expect(selectedLanguages({ french: false, english: false })).toEqual([]);
  });
});

describe("translation model requirement", () => {
  it("is needed exactly when English is selected", () => {
    expect(needsTranslationModel(selection(true, true, true, true))).toBe(true);
    expect(needsTranslationModel(selection(false, true, true, true))).toBe(true);
    expect(needsTranslationModel(selection(true, false, true, true))).toBe(false);
  });
});
