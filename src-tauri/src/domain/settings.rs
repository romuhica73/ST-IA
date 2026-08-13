//! Pure schema and validation for ST-IA's local preferences (see ADR-007).
//!
//! Three independent preferences, each with an explicit "follow the system"
//! option that is also the default — a first launch never requires the user
//! to make a choice before the app is usable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MotionPreference {
    System,
    /// Force reduced motion, regardless of the OS setting.
    On,
    /// Force full motion, regardless of the OS setting.
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguagePreference {
    System,
    Fr,
    En,
}

// Used for the native window's boot background, which has to be decided
// before any stylesheet exists (see `window::create`). Pinned to the serde
// representation by the test below rather than to a hand-written literal, so
// the two cannot drift.
impl ThemePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// The application's UI language and the language `whisper-cli` is told to
/// transcribe in are deliberately two separate settings (see ADR-007) — this
/// struct only ever represents the former. Nothing here reaches the
/// transcription pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: ThemePreference,
    pub motion: MotionPreference,
    pub language: LanguagePreference,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            motion: MotionPreference::System,
            language: LanguagePreference::System,
        }
    }
}

impl Settings {
    /// Parses a settings file's contents, never failing: a missing field, an
    /// unknown enum value, or plain garbage all fall back to `default()`
    /// wholesale rather than trying to salvage individual fields — simpler
    /// to reason about, and the file gets naturally healed on next save.
    pub fn parse(raw: &str) -> Self {
        serde_json::from_str(raw).unwrap_or_default()
    }

    pub fn to_json_pretty(self) -> String {
        // Infallible: every field is a plain enum, there is no way for this
        // struct to fail to serialize.
        serde_json::to_string_pretty(&self).expect("Settings is always serializable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_all_system() {
        let s = Settings::default();
        assert_eq!(s.theme, ThemePreference::System);
        assert_eq!(s.motion, MotionPreference::System);
        assert_eq!(s.language, LanguagePreference::System);
    }

    #[test]
    fn round_trips_through_json() {
        let s = Settings {
            theme: ThemePreference::Dark,
            motion: MotionPreference::On,
            language: LanguagePreference::En,
        };
        let parsed = Settings::parse(&s.to_json_pretty());
        assert_eq!(parsed, s);
    }

    #[test]
    fn empty_file_falls_back_to_defaults() {
        assert_eq!(Settings::parse(""), Settings::default());
    }

    #[test]
    fn garbage_falls_back_to_defaults() {
        assert_eq!(Settings::parse("not json at all {{{"), Settings::default());
    }

    #[test]
    fn unknown_enum_value_falls_back_to_defaults_wholesale() {
        // theme is garbage, but motion/language are otherwise valid — the
        // whole file is still rejected rather than salvaged field-by-field.
        let raw = r#"{"theme":"purple","motion":"on","language":"en"}"#;
        assert_eq!(Settings::parse(raw), Settings::default());
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        assert_eq!(Settings::parse(r#"{"theme":"dark"}"#), Settings::default());
    }

    /// The wire value serde actually produces, so the assertions below
    /// compare `as_str()` against serialization rather than against a second
    /// hand-written copy of the same literal.
    fn serialized(value: impl Serialize) -> String {
        serde_json::to_value(value)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn theme_as_str_matches_its_serialized_form() {
        for theme in [
            ThemePreference::System,
            ThemePreference::Light,
            ThemePreference::Dark,
        ] {
            assert_eq!(theme.as_str(), serialized(theme));
        }
    }

    #[test]
    fn extra_unknown_fields_are_tolerated() {
        // Forward compatibility: a settings file written by a newer ST-IA
        // version with an extra field must not be rejected by an older one.
        let raw = r#"{"theme":"dark","motion":"system","language":"system","futureField":123}"#;
        let parsed = Settings::parse(raw);
        assert_eq!(parsed.theme, ThemePreference::Dark);
    }
}
