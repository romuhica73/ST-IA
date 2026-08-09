use serde::Serialize;

/// Canonical manifest for the single MVP model. No catalog, no selection —
/// see ADR-004.
pub const MODEL_ID: &str = "large-v3-turbo-q5_0";
pub const MODEL_FILE_NAME: &str = "ggml-large-v3-turbo-q5_0.bin";
pub const MODEL_EXPECTED_SIZE: u64 = 574_041_195;
pub const MODEL_SHA256: &str = "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2";
pub const MODEL_DOWNLOAD_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin";

/// Frontend-facing manifest info — no sha256 (irrelevant to the UI) and no
/// generic catalog, just enough to render "Modèle requis" (name, size).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManifest {
    pub id: String,
    pub file_name: String,
    pub expected_size: u64,
}

pub fn manifest() -> ModelManifest {
    ModelManifest {
        id: MODEL_ID.to_string(),
        file_name: MODEL_FILE_NAME.to_string(),
        expected_size: MODEL_EXPECTED_SIZE,
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

/// Name of the temporary file a download is written to before its integrity
/// is verified. Never the final canonical name while a download is in
/// flight, so a crash/interruption can never leave a corrupt file at the
/// name whisper.cpp will actually load.
pub fn temp_file_name() -> String {
    format!("{MODEL_FILE_NAME}.download")
}

/// A model file is only ever valid if both its size and its SHA-256 match
/// the pinned manifest exactly — matching the file name alone proves
/// nothing.
pub fn is_valid(size: u64, sha256_hex: &str) -> bool {
    size == MODEL_EXPECTED_SIZE && sha256_hex.eq_ignore_ascii_case(MODEL_SHA256)
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

    #[test]
    fn temp_file_name_never_equals_final_name() {
        let tmp = temp_file_name();
        assert_ne!(tmp, MODEL_FILE_NAME);
        assert!(tmp.starts_with(MODEL_FILE_NAME));
    }

    #[test]
    fn rejects_wrong_size_even_with_correct_hash() {
        assert!(!is_valid(1, MODEL_SHA256));
    }

    #[test]
    fn rejects_correct_size_with_wrong_hash() {
        assert!(!is_valid(
            MODEL_EXPECTED_SIZE,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn accepts_exact_match() {
        assert!(is_valid(MODEL_EXPECTED_SIZE, MODEL_SHA256));
    }

    #[test]
    fn hash_comparison_is_case_insensitive() {
        assert!(is_valid(MODEL_EXPECTED_SIZE, &MODEL_SHA256.to_uppercase()));
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
}
