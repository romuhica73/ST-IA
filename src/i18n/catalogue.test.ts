import { describe, expect, it } from "vitest";
import fr from "./locales/fr";
import en from "./locales/en";

/** Recursively collects every leaf key path ("error.codes.modelMissing.title")
 * in a nested translation object. */
function collectKeyPaths(node: unknown, prefix = ""): string[] {
  if (typeof node !== "object" || node === null) {
    return [prefix];
  }
  return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
    collectKeyPaths(value, prefix ? `${prefix}.${key}` : key),
  );
}

function collectEmptyValuePaths(node: unknown, prefix = ""): string[] {
  if (typeof node === "string") {
    return node.trim() === "" ? [prefix] : [];
  }
  if (typeof node !== "object" || node === null) {
    return [];
  }
  return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
    collectEmptyValuePaths(value, prefix ? `${prefix}.${key}` : key),
  );
}

describe("i18n catalogue parity (fr/en)", () => {
  it("has exactly the same key shape in both catalogues", () => {
    const frKeys = new Set(collectKeyPaths(fr));
    const enKeys = new Set(collectKeyPaths(en));

    const missingFromEn = [...frKeys].filter((k) => !enKeys.has(k));
    const missingFromFr = [...enKeys].filter((k) => !frKeys.has(k));

    expect(missingFromEn, `keys present in fr but missing from en: ${missingFromEn.join(", ")}`).toEqual([]);
    expect(missingFromFr, `keys present in en but missing from fr: ${missingFromFr.join(", ")}`).toEqual([]);
  });

  it("has no empty string values in fr", () => {
    expect(collectEmptyValuePaths(fr)).toEqual([]);
  });

  it("has no empty string values in en", () => {
    expect(collectEmptyValuePaths(en)).toEqual([]);
  });

  it("has a non-trivial number of keys (sanity check against an accidentally-empty catalogue)", () => {
    expect(collectKeyPaths(fr).length).toBeGreaterThan(50);
  });
});
