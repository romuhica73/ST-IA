import { describe, expect, it } from "vitest";
import { formatBytes } from "./formatBytes";

describe("formatBytes", () => {
  it("uses French octet-based units", () => {
    expect(formatBytes(0, "fr")).toBe("0 o");
    expect(formatBytes(1024, "fr")).toBe("1.0 Ko");
    expect(formatBytes(574_041_195, "fr")).toBe("547.4 Mo");
  });

  it("uses English byte-based units for the same values", () => {
    expect(formatBytes(0, "en")).toBe("0 B");
    expect(formatBytes(1024, "en")).toBe("1.0 KB");
    expect(formatBytes(574_041_195, "en")).toBe("547.4 MB");
  });

  it("never fabricates a value for non-positive input", () => {
    expect(formatBytes(-5, "fr")).toBe("0 o");
    expect(formatBytes(-5, "en")).toBe("0 B");
  });
});
