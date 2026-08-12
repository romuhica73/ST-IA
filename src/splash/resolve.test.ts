import { describe, expect, it } from "vitest";
import { readPreference, resolveSplashMotion, resolveSplashTheme } from "./resolve";

describe("splash preference reading", () => {
  it("reads a preference from the fragment Rust builds the window with", () => {
    // Regression guard: this was a query string, which made the window load a
    // blank page — Tauri's asset resolver matches the request path verbatim
    // and `splash.html?theme=…` matches no embedded asset. A fragment never
    // reaches the resolver.
    expect(readPreference("#theme=dark&motion=on", "theme")).toBe("dark");
    expect(readPreference("#theme=dark&motion=on", "motion")).toBe("on");
  });

  it("still reads a query string, so neither separator can silently break", () => {
    expect(readPreference("?theme=dark&motion=on", "theme")).toBe("dark");
  });

  it("falls back to system when the parameter is absent", () => {
    expect(readPreference("", "theme")).toBe("system");
    expect(readPreference("#motion=on", "theme")).toBe("system");
  });

  it("never throws on a malformed fragment", () => {
    // A splash screen has no error state to fall back to, so every input
    // has to resolve to something displayable.
    for (const raw of ["#", "#=", "#theme", "##theme=dark", "&&&", "#theme=%", "?"]) {
      expect(() => readPreference(raw, "theme")).not.toThrow();
    }
  });
});

describe("splash theme resolution", () => {
  it("honours an explicit preference over the OS setting", () => {
    expect(resolveSplashTheme("light", true)).toBe("light");
    expect(resolveSplashTheme("dark", false)).toBe("dark");
  });

  it("follows the OS when the preference is system", () => {
    expect(resolveSplashTheme("system", true)).toBe("dark");
    expect(resolveSplashTheme("system", false)).toBe("light");
  });

  it("treats an unknown value as system rather than failing", () => {
    expect(resolveSplashTheme("purple", true)).toBe("dark");
    expect(resolveSplashTheme("", false)).toBe("light");
  });
});

describe("splash motion resolution", () => {
  it("forces reduced motion when the preference says so, whatever the OS says", () => {
    expect(resolveSplashMotion("on", false)).toBe("reduce");
  });

  it("forces full motion when the preference says so, whatever the OS says", () => {
    // The M7 setting can override the OS in *both* directions; a plain
    // prefers-reduced-motion media query could not express this case.
    expect(resolveSplashMotion("off", true)).toBe("full");
  });

  it("follows the OS when the preference is system", () => {
    expect(resolveSplashMotion("system", true)).toBe("reduce");
    expect(resolveSplashMotion("system", false)).toBe("full");
  });

  it("treats an unknown value as system", () => {
    expect(resolveSplashMotion("maybe", true)).toBe("reduce");
  });
});

describe("splash and main app resolve motion identically", () => {
  it("agrees with features/settings/resolve for every preference and OS combination", async () => {
    // The splash cannot import the app's resolver (different bundle, no
    // shared runtime), so the duplication is deliberate — this pins the two
    // implementations together so they cannot drift apart unnoticed.
    const { resolveMotion } = await import("../features/settings/resolve");
    for (const preference of ["system", "on", "off"] as const) {
      for (const systemReduced of [true, false]) {
        expect(resolveSplashMotion(preference, systemReduced)).toBe(
          resolveMotion(preference, systemReduced),
        );
      }
    }
  });
});
