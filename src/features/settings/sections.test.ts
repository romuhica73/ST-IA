import { describe, expect, it } from "vitest";
import {
  DEFAULT_SECTION,
  moveSelection,
  SECTION_LABEL_KEY,
  SETTINGS_SECTIONS,
} from "./sections";

describe("settings sections", () => {
  it("opens on General", () => {
    expect(DEFAULT_SECTION).toBe("general");
    expect(SETTINGS_SECTIONS[0]).toBe("general");
  });

  it("has a label key for every section, and no orphan keys", () => {
    // The sidebar and the panel body are both driven by this list, so a
    // section can never exist in one and not the other.
    expect(Object.keys(SECTION_LABEL_KEY).sort()).toEqual([...SETTINGS_SECTIONS].sort());
  });

  it("covers the four product areas", () => {
    expect(SETTINGS_SECTIONS).toEqual(["general", "accessibility", "models", "about"]);
  });
});

describe("keyboard navigation", () => {
  it("moves down and up through the list", () => {
    expect(moveSelection("general", 1)).toBe("accessibility");
    expect(moveSelection("accessibility", -1)).toBe("general");
  });

  it("stops at the ends instead of wrapping", () => {
    // Wrapping is menu behaviour; in a navigation list it makes it easy to
    // lose your place.
    expect(moveSelection("general", -1)).toBe("general");
    expect(moveSelection("about", 1)).toBe("about");
  });

  it("clamps an out-of-range jump to a real section", () => {
    expect(moveSelection("general", 99)).toBe("about");
    expect(moveSelection("about", -99)).toBe("general");
  });
});
