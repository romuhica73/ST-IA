import { describe, expect, it } from "vitest";
import { FALLBACK_LANGUAGE, resolveSystemLanguage } from "./locale";

describe("resolveSystemLanguage", () => {
  it("resolves French locales to fr", () => {
    expect(resolveSystemLanguage("fr")).toBe("fr");
    expect(resolveSystemLanguage("fr-FR")).toBe("fr");
    expect(resolveSystemLanguage("fr-CA")).toBe("fr");
  });

  it("resolves English locales to en", () => {
    expect(resolveSystemLanguage("en")).toBe("en");
    expect(resolveSystemLanguage("en-US")).toBe("en");
    expect(resolveSystemLanguage("en-GB")).toBe("en");
  });

  it("falls back to the documented default for an unsupported locale", () => {
    // v0.1 ships exactly fr/en; a German, Spanish, etc. system must not be
    // silently claimed as supported.
    expect(resolveSystemLanguage("de")).toBe(FALLBACK_LANGUAGE);
    expect(resolveSystemLanguage("de-DE")).toBe(FALLBACK_LANGUAGE);
    expect(resolveSystemLanguage("es-ES")).toBe(FALLBACK_LANGUAGE);
    expect(resolveSystemLanguage("ja")).toBe(FALLBACK_LANGUAGE);
  });

  it("falls back for missing/empty input rather than throwing", () => {
    expect(resolveSystemLanguage(undefined)).toBe(FALLBACK_LANGUAGE);
    expect(resolveSystemLanguage(null)).toBe(FALLBACK_LANGUAGE);
    expect(resolveSystemLanguage("")).toBe(FALLBACK_LANGUAGE);
  });

  it("fallback language is English, the documented international default", () => {
    expect(FALLBACK_LANGUAGE).toBe("en");
  });
});
