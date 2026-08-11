import { describe, expect, it } from "vitest";
import { resolveLanguage, resolveMotion, resolveTheme } from "./resolve";

describe("resolveTheme", () => {
  it("explicit light/dark always wins over the system state", () => {
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  it("system follows the OS query", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
  });
});

describe("resolveMotion", () => {
  it("On always reduces motion, even if the OS does not ask for it", () => {
    expect(resolveMotion("on", false)).toBe("reduce");
  });

  it("Off always keeps full motion, even if the OS asks for reduced motion", () => {
    expect(resolveMotion("off", true)).toBe("full");
  });

  it("system follows the OS query", () => {
    expect(resolveMotion("system", true)).toBe("reduce");
    expect(resolveMotion("system", false)).toBe("full");
  });
});

describe("resolveLanguage", () => {
  it("explicit fr/en always wins over the system locale", () => {
    expect(resolveLanguage("fr", "en-US")).toBe("fr");
    expect(resolveLanguage("en", "fr-FR")).toBe("en");
  });

  it("system falls through to system-locale resolution", () => {
    expect(resolveLanguage("system", "fr-FR")).toBe("fr");
    expect(resolveLanguage("system", "en-US")).toBe("en");
    expect(resolveLanguage("system", "de-DE")).toBe("en"); // documented fallback
  });
});
