use serde::{Deserialize, Serialize};

/// The two models ST-IA can hold on disk. Not a catalog and not a user
/// choice: each has exactly one qualified file, pinned by size and SHA-256
/// (see ADR-004 and ADR-010). The user never picks a model — they pick an
/// output language, and that determines which model is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelKind {
    /// French transcription. Downloaded on first use, required for any job.
    Transcription,
    /// French → English translation. Downloaded only if the user asks for
    /// English output, and never otherwise — it is five times the size.
    Translation,
}

impl ModelKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transcription => "transcription",
            Self::Translation => "translation",
        }
    }
}

pub struct ModelSpec {
    pub id: &'static str,
    pub file_name: &'static str,
    pub expected_size: u64,
    pub sha256: &'static str,
    pub download_url: &'static str,
}

// Both download URLs pin the exact commit of ggerganov/whisper.cpp on
// Hugging Face that was verified to serve the qualified model — not `main`,
// which is a moving branch pointer that could be repointed at a different
// upload. See ADR-004 for the original verification evidence and ADR-010 for
// the translation model's. The tests below assert the pin is still there.

/// `large-v3-turbo-q5_0` — fast French transcription. Unchanged since M0B;
/// M9 explicitly does *not* replace it.
pub const TRANSCRIPTION_MODEL: ModelSpec = ModelSpec {
    id: "large-v3-turbo-q5_0",
    file_name: "ggml-large-v3-turbo-q5_0.bin",
    expected_size: 574_041_195,
    sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
    download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo-q5_0.bin",
};

/// `large-v3` — full, non-turbo, non-quantised.
///
/// Deliberately *not* the turbo model: `large-v3-turbo` is a distilled
/// decoder trained on transcription data only and does not perform the
/// translation task at all — it returns French (measured, see ADR-008). And
/// deliberately not a quantised build either: the product decision for
/// translation is best available quality, size and speed second.
pub const TRANSLATION_MODEL: ModelSpec = ModelSpec {
    id: "large-v3",
    file_name: "ggml-large-v3.bin",
    expected_size: 3_095_033_483,
    sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
    download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3.bin",
};

pub fn spec(kind: ModelKind) -> &'static ModelSpec {
    match kind {
        ModelKind::Transcription => &TRANSCRIPTION_MODEL,
        ModelKind::Translation => &TRANSLATION_MODEL,
    }
}

// Kept as the transcription model's names so the pre-release data migration
// (ADR-006) keeps referring to exactly the artefact it was written for. The
// migration has never concerned the translation model, which did not exist
// when the legacy bundle identifier did.
pub const MODEL_FILE_NAME: &str = TRANSCRIPTION_MODEL.file_name;

/// Frontend-facing manifest info — no sha256 (irrelevant to the UI), just
/// enough to render the "model required" screens (which model, how big).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManifest {
    pub kind: ModelKind,
    pub id: String,
    pub file_name: String,
    pub expected_size: u64,
}

pub fn manifest(kind: ModelKind) -> ModelManifest {
    let spec = spec(kind);
    ModelManifest {
        kind,
        id: spec.id.to_string(),
        file_name: spec.file_name.to_string(),
        expected_size: spec.expected_size,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelErrorCode {
    NetworkError,
    WriteError,
    IntegrityMismatch,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelError {
    pub code: ModelErrorCode,
    pub message: String,
}

impl ModelError {
    pub fn network(message: impl Into<String>) -> Self {
        Self {
            code: ModelErrorCode::NetworkError,
            message: message.into(),
        }
    }

    pub fn write(message: impl Into<String>) -> Self {
        Self {
            code: ModelErrorCode::WriteError,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ModelStatus {
    Missing,
    #[serde(rename_all = "camelCase")]
    Downloading {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        progress: Option<f32>,
    },
    Verifying,
    Ready,
    Corrupted,
    Failed {
        code: ModelErrorCode,
        message: String,
    },
}

/// What goes over `model://event`. The `kind` is what lets the frontend tell
/// a translation-model download apart from a transcription-model one —
/// without it, a 3 GB download would drive the transcription screen's
/// progress bar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatusEvent {
    pub kind: ModelKind,
    #[serde(flatten)]
    pub status: ModelStatus,
}

/// Name of the temporary file a download is written to before its integrity
/// is verified. Never the final canonical name while a download is in
/// flight, so a crash/interruption can never leave a corrupt file at the
/// name whisper.cpp will actually load.
pub fn temp_file_name(kind: ModelKind) -> String {
    format!("{}.download", spec(kind).file_name)
}

/// A model file is only ever valid if both its size and its SHA-256 match
/// the pinned manifest exactly — matching the file name alone proves
/// nothing.
pub fn is_valid(kind: ModelKind, size: u64, sha256_hex: &str) -> bool {
    let spec = spec(kind);
    size == spec.expected_size && sha256_hex.eq_ignore_ascii_case(spec.sha256)
}

/// Real progress only — `None` when the remote content-length isn't known,
/// so the caller shows an indeterminate state instead of a fabricated
/// number.
pub fn compute_progress(downloaded: u64, total: Option<u64>) -> Option<f32> {
    total
        .filter(|t| *t > 0)
        .map(|t| (downloaded as f32 / t as f32).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTH: [ModelKind; 2] = [ModelKind::Transcription, ModelKind::Translation];

    /// The verified commit both models must be served from.
    const PINNED_COMMIT: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

    #[test]
    fn temp_file_name_never_equals_final_name() {
        for kind in BOTH {
            let tmp = temp_file_name(kind);
            assert_ne!(tmp, spec(kind).file_name);
            assert!(tmp.starts_with(spec(kind).file_name));
        }
    }

    #[test]
    fn the_two_models_are_distinct_files() {
        // They live in the same directory; identical names would mean one
        // download silently overwriting the other.
        assert_ne!(
            TRANSCRIPTION_MODEL.file_name, TRANSLATION_MODEL.file_name,
            "the two models must not share a file name"
        );
        assert_ne!(TRANSCRIPTION_MODEL.sha256, TRANSLATION_MODEL.sha256);
        assert_ne!(
            temp_file_name(ModelKind::Transcription),
            temp_file_name(ModelKind::Translation)
        );
    }

    #[test]
    fn the_translation_model_is_not_a_turbo_build() {
        // The whole reason this model exists: turbo cannot translate. A
        // well-meaning "let's save 2.5 GB" edit would silently reintroduce
        // French-in-the-English-file.
        assert!(
            !TRANSLATION_MODEL.id.contains("turbo"),
            "large-v3-turbo does not perform the translation task (ADR-008)"
        );
        assert!(!TRANSLATION_MODEL.file_name.contains("turbo"));
    }

    #[test]
    fn the_transcription_model_is_unchanged() {
        // M9 must not swap the qualified French transcription model.
        assert_eq!(TRANSCRIPTION_MODEL.id, "large-v3-turbo-q5_0");
        assert_eq!(TRANSCRIPTION_MODEL.expected_size, 574_041_195);
        assert_eq!(
            TRANSCRIPTION_MODEL.sha256,
            "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2"
        );
    }

    #[test]
    fn both_models_are_pinned_to_the_same_verified_commit() {
        // A moving `main` pointer could be repointed at a different upload.
        for kind in BOTH {
            let url = spec(kind).download_url;
            assert!(url.starts_with("https://"), "{url} must be HTTPS");
            assert!(
                url.contains(PINNED_COMMIT),
                "{url} must pin the verified commit, not a branch"
            );
            assert!(!url.contains("/main/"), "{url} must not track a branch");
        }
    }

    #[test]
    fn download_urls_end_with_the_file_they_claim_to_serve() {
        for kind in BOTH {
            let spec = spec(kind);
            assert!(
                spec.download_url.ends_with(spec.file_name),
                "{} does not serve {}",
                spec.download_url,
                spec.file_name
            );
        }
    }

    #[test]
    fn rejects_wrong_size_even_with_correct_hash() {
        assert!(!is_valid(
            ModelKind::Transcription,
            1,
            TRANSCRIPTION_MODEL.sha256
        ));
        assert!(!is_valid(
            ModelKind::Translation,
            1,
            TRANSLATION_MODEL.sha256
        ));
    }

    #[test]
    fn rejects_correct_size_with_wrong_hash() {
        assert!(!is_valid(
            ModelKind::Transcription,
            TRANSCRIPTION_MODEL.expected_size,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn rejects_the_other_models_hash() {
        // The two files sit side by side; validating one against the other's
        // manifest must fail on both counts.
        assert!(!is_valid(
            ModelKind::Transcription,
            TRANSLATION_MODEL.expected_size,
            TRANSLATION_MODEL.sha256
        ));
        assert!(!is_valid(
            ModelKind::Translation,
            TRANSCRIPTION_MODEL.expected_size,
            TRANSCRIPTION_MODEL.sha256
        ));
    }

    #[test]
    fn accepts_exact_match() {
        for kind in BOTH {
            assert!(is_valid(kind, spec(kind).expected_size, spec(kind).sha256));
        }
    }

    #[test]
    fn hash_comparison_is_case_insensitive() {
        for kind in BOTH {
            assert!(is_valid(
                kind,
                spec(kind).expected_size,
                &spec(kind).sha256.to_uppercase()
            ));
        }
    }

    #[test]
    fn progress_is_none_without_known_total() {
        assert_eq!(compute_progress(1000, None), None);
    }

    #[test]
    fn progress_is_none_with_zero_total() {
        assert_eq!(compute_progress(0, Some(0)), None);
    }

    #[test]
    fn progress_computes_real_fraction() {
        assert_eq!(compute_progress(50, Some(200)), Some(0.25));
    }

    #[test]
    fn progress_is_clamped_to_one() {
        // Defensive: a server that reports a total smaller than what we
        // actually received should never produce a >100% progress value.
        assert_eq!(compute_progress(300, Some(200)), Some(1.0));
    }

    #[test]
    fn downloading_status_serializes_fields_as_camel_case() {
        // Regression: an internally-tagged enum's `rename_all` only renames
        // variant names, not struct-variant field names — those need their
        // own `rename_all` on the variant itself, or fields silently stay
        // snake_case and the frontend reads `undefined`.
        let status = ModelStatus::Downloading {
            downloaded_bytes: 12_345_678,
            total_bytes: Some(574_041_195),
            progress: Some(0.5),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["downloadedBytes"], 12_345_678);
        assert_eq!(json["totalBytes"], 574_041_195);
        assert!(json.get("downloaded_bytes").is_none());
        assert!(json.get("total_bytes").is_none());
    }

    #[test]
    fn the_event_carries_which_model_it_is_about() {
        // Without this the frontend cannot tell a 3 GB translation download
        // from a 574 MB transcription one, and would drive the wrong screen.
        let event = ModelStatusEvent {
            kind: ModelKind::Translation,
            status: ModelStatus::Downloading {
                downloaded_bytes: 1,
                total_bytes: Some(2),
                progress: Some(0.5),
            },
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "translation");
        assert_eq!(json["status"], "downloading");
        assert_eq!(json["downloadedBytes"], 1);
    }
}
