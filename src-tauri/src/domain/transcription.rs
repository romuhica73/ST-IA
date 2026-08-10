use serde::Serialize;
use std::path::{Path, PathBuf};

pub const LANGUAGE: &str = "fr";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputKind {
    Srt,
    Txt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputSelection {
    pub srt: bool,
    pub txt: bool,
}

impl OutputSelection {
    pub fn is_empty(&self) -> bool {
        !self.srt && !self.txt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TranscribingPhase {
    LoadingModel,
    Processing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
    pub kind: OutputKind,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TranscriptionErrorCode {
    AlreadyRunning,
    ModelMissing,
    NoOutputSelected,
    AudioPreparationFailed,
    NoAudioTrack,
    TranscriptionFailed,
    WriteFailed,
    InsufficientDiskSpace,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionError {
    pub code: TranscriptionErrorCode,
    pub message: String,
}

impl TranscriptionError {
    pub fn new(code: TranscriptionErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn already_running() -> Self {
        Self::new(
            TranscriptionErrorCode::AlreadyRunning,
            "Une transcription est déjà en cours.",
        )
    }

    pub fn model_missing() -> Self {
        Self::new(
            TranscriptionErrorCode::ModelMissing,
            "Le modèle de transcription n'est pas encore installé.",
        )
    }

    pub fn no_output_selected() -> Self {
        Self::new(
            TranscriptionErrorCode::NoOutputSelected,
            "Sélectionnez au moins un format de sortie (SRT ou TXT).",
        )
    }

    pub fn no_audio_track() -> Self {
        Self::new(
            TranscriptionErrorCode::NoAudioTrack,
            "Aucune piste audio détectée. Vérifiez que la vidéo contient de l'audio ou choisissez un autre fichier.",
        )
    }

    pub fn audio_preparation_failed() -> Self {
        Self::new(
            TranscriptionErrorCode::AudioPreparationFailed,
            "L'audio n'a pas pu être préparé à partir de ce fichier.",
        )
    }

    pub fn transcription_failed() -> Self {
        Self::new(
            TranscriptionErrorCode::TranscriptionFailed,
            "La transcription a échoué.",
        )
    }

    pub fn write_failed() -> Self {
        Self::new(
            TranscriptionErrorCode::WriteFailed,
            "Les fichiers de sous-titres n'ont pas pu être enregistrés.",
        )
    }

    pub fn insufficient_disk_space() -> Self {
        Self::new(
            TranscriptionErrorCode::InsufficientDiskSpace,
            "L'espace disque disponible est insuffisant pour traiter ce fichier.",
        )
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum JobStatus {
    #[default]
    Idle,
    PreparingAudio,
    Transcribing {
        phase: TranscribingPhase,
        progress: Option<f32>,
    },
    WritingOutputs,
    #[serde(rename_all = "camelCase")]
    Completed {
        output_dir: String,
        files: Vec<OutputFile>,
        /// Text content of the generated .txt output, if it was requested —
        /// carried here so the frontend can offer "copy transcription"
        /// without a generic read-any-file command.
        transcript_text: Option<String>,
    },
    Failed {
        code: TranscriptionErrorCode,
        message: String,
    },
    /// Cancellation requested, child process being stopped. Terminal state
    /// `Cancelled` is only emitted once the process has actually exited and
    /// the temporary workspace has been removed.
    Cancelling,
    Cancelled,
}

/// Maps a failed ffmpeg run to a business error.
///
/// The user never sees ffmpeg's stderr; it is only inspected here to tell
/// "this file has no audio to transcribe" apart from "this file could not be
/// decoded at all". The sentinels are the strings the pinned sidecar build
/// actually produces (verified against real fixtures — see ADR-005).
pub fn classify_ffmpeg_failure(stderr: &str) -> TranscriptionError {
    let stderr = stderr.to_lowercase();
    if stderr.contains("does not contain any stream") || stderr.contains("matches no streams") {
        TranscriptionError::no_audio_track()
    } else {
        TranscriptionError::audio_preparation_failed()
    }
}

/// Safety margin on top of the source size when estimating how much free
/// space a job needs — covers the SRT/TXT outputs, filesystem overhead and
/// whisper.cpp's own scratch usage.
pub const DISK_SAFETY_MARGIN_BYTES: u64 = 256 * 1024 * 1024;

/// Conservative estimate of the free space a transcription needs.
///
/// The intermediate WAV is 16 kHz mono s16 = 32 kB/s = 256 kbps. Any real
/// audio/video file is encoded at a higher bitrate than that, so the source
/// file's own size is an upper bound for the WAV we are about to extract.
/// We require that upper bound plus a fixed margin — deliberately a coarse
/// guard against a manifestly full disk, not a precise storage manager.
pub fn required_free_bytes(source_size: u64) -> u64 {
    source_size.saturating_add(DISK_SAFETY_MARGIN_BYTES)
}

/// Whether a temp entry is one ST-IA created and may therefore delete.
///
/// Guard rail for startup cleanup: the caller passes entries found directly
/// inside ST-IA's *own* temp root, and this refuses anything that does not
/// look like a job directory this app produced (`<pid>-<nanos>`). Nothing
/// outside that root is ever considered, and no user file or model is ever
/// matched by this predicate.
pub fn is_removable_job_dir_name(name: &str) -> bool {
    match name.split_once('-') {
        Some((pid, nanos)) => {
            !pid.is_empty()
                && !nanos.is_empty()
                && pid.chars().all(|c| c.is_ascii_digit())
                && nanos.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Resolves the sibling output directory for a source media path, avoiding
/// silent overwrite of a pre-existing folder by appending -2, -3, ... .
/// `exists` is injected so the collision-avoidance logic is unit-testable
/// without touching the real filesystem.
pub fn resolve_output_dir(source: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sous-titres");

    let base = parent.join(format!("{stem}-sous-titres"));
    if !exists(&base) {
        return base;
    }

    let mut n = 2;
    loop {
        let candidate = parent.join(format!("{stem}-sous-titres-{n}"));
        if !exists(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Arguments for the ffmpeg sidecar: demux + decode the audio track of
/// `input`, downmix/resample it to 16kHz mono PCM16, write it to `output_wav`.
/// No noise reduction, no enhancement, no normalization.
pub fn build_ffmpeg_args(input: &Path, output_wav: &Path) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-i".to_string(),
        input.to_string_lossy().to_string(),
        "-vn".to_string(),
        "-ac".to_string(),
        "1".to_string(),
        "-ar".to_string(),
        "16000".to_string(),
        "-c:a".to_string(),
        "pcm_s16le".to_string(),
        output_wav.to_string_lossy().to_string(),
    ]
}

/// Arguments for the whisper-cli sidecar. Language is always forced to
/// French; output formats follow the user's SRT/TXT selection.
/// `output_prefix` is a path without extension (whisper-cli appends
/// `.srt`/`.txt` itself).
pub fn build_whisper_args(
    model: &Path,
    wav: &Path,
    output_prefix: &Path,
    outputs: OutputSelection,
) -> Vec<String> {
    let mut args = vec![
        "-m".to_string(),
        model.to_string_lossy().to_string(),
        "-f".to_string(),
        wav.to_string_lossy().to_string(),
        "-l".to_string(),
        LANGUAGE.to_string(),
        "-of".to_string(),
        output_prefix.to_string_lossy().to_string(),
    ];
    if outputs.srt {
        args.push("-osrt".to_string());
    }
    if outputs.txt {
        args.push("-otxt".to_string());
    }
    args
}

/// Parses the end timestamp (in seconds) from a whisper-cli stdout segment
/// line, e.g. "[00:00:11.000 --> 00:00:22.480]  some text" -> Some(22.48).
/// Returns None for any line that isn't a recognizable segment line — this
/// is the only source of transcription progress; if it can't be parsed, the
/// caller reports an indeterminate state rather than a fabricated number.
pub fn parse_segment_end_seconds(line: &str) -> Option<f32> {
    let end = line.split("-->").nth(1)?;
    let end = end.trim().split(']').next()?.trim();
    parse_timestamp_seconds(end)
}

fn parse_timestamp_seconds(ts: &str) -> Option<f32> {
    // HH:MM:SS.mmm
    let mut parts = ts.split(':');
    let hours: f32 = parts.next()?.parse().ok()?;
    let minutes: f32 = parts.next()?.parse().ok()?;
    let seconds: f32 = parts.next()?.parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

/// Computes the duration in seconds of a canonical PCM WAV file by walking
/// its RIFF chunks (rather than assuming a fixed 44-byte header).
pub fn wav_duration_secs(bytes: &[u8]) -> Option<f32> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }

    let mut pos = 12;
    let mut sample_rate: Option<u32> = None;
    let mut channels: Option<u16> = None;
    let mut bits_per_sample: Option<u16> = None;
    let mut data_len: Option<u32> = None;

    while pos + 8 <= bytes.len() {
        let chunk_id = &bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]);
        let body_start = pos + 8;

        if chunk_id == b"fmt " && body_start + 16 <= bytes.len() {
            channels = Some(u16::from_le_bytes([
                bytes[body_start + 2],
                bytes[body_start + 3],
            ]));
            sample_rate = Some(u32::from_le_bytes([
                bytes[body_start + 4],
                bytes[body_start + 5],
                bytes[body_start + 6],
                bytes[body_start + 7],
            ]));
            bits_per_sample = Some(u16::from_le_bytes([
                bytes[body_start + 14],
                bytes[body_start + 15],
            ]));
        } else if chunk_id == b"data" {
            data_len = Some(chunk_size);
        }

        // Chunks are word-aligned: a chunk with an odd size is padded by one byte.
        let advance = chunk_size as usize + (chunk_size as usize % 2);
        pos = body_start + advance;
    }

    let sample_rate = sample_rate? as f32;
    let channels = channels? as f32;
    let bits_per_sample = bits_per_sample? as f32;
    let data_len = data_len? as f32;

    if sample_rate == 0.0 || channels == 0.0 || bits_per_sample == 0.0 {
        return None;
    }

    let bytes_per_second = sample_rate * channels * (bits_per_sample / 8.0);
    Some(data_len / bytes_per_second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_output_dir_uses_base_name_when_free() {
        let dir = resolve_output_dir(Path::new("/videos/Emission-IA.mov"), |_| false);
        assert_eq!(dir, PathBuf::from("/videos/Emission-IA-sous-titres"));
    }

    #[test]
    fn resolve_output_dir_avoids_collision() {
        let taken = [
            PathBuf::from("/videos/Emission-IA-sous-titres"),
            PathBuf::from("/videos/Emission-IA-sous-titres-2"),
        ];
        let dir = resolve_output_dir(Path::new("/videos/Emission-IA.mov"), |p| {
            taken.contains(&p.to_path_buf())
        });
        assert_eq!(dir, PathBuf::from("/videos/Emission-IA-sous-titres-3"));
    }

    #[test]
    fn ffmpeg_args_force_mono_16k_pcm16_no_video() {
        let args = build_ffmpeg_args(Path::new("/in/media.mov"), Path::new("/tmp/audio.wav"));
        assert_eq!(
            args,
            vec![
                "-y",
                "-i",
                "/in/media.mov",
                "-vn",
                "-ac",
                "1",
                "-ar",
                "16000",
                "-c:a",
                "pcm_s16le",
                "/tmp/audio.wav",
            ]
        );
    }

    #[test]
    fn whisper_args_force_french_and_respect_output_selection() {
        let args = build_whisper_args(
            Path::new("/models/ggml-large-v3-turbo-q5_0.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/transcript"),
            OutputSelection {
                srt: true,
                txt: false,
            },
        );
        assert_eq!(
            args,
            vec![
                "-m",
                "/models/ggml-large-v3-turbo-q5_0.bin",
                "-f",
                "/tmp/audio.wav",
                "-l",
                "fr",
                "-of",
                "/tmp/transcript",
                "-osrt",
            ]
        );
    }

    #[test]
    fn whisper_args_include_both_outputs_when_both_selected() {
        let args = build_whisper_args(
            Path::new("/models/m.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/transcript"),
            OutputSelection {
                srt: true,
                txt: true,
            },
        );
        assert!(args.contains(&"-osrt".to_string()));
        assert!(args.contains(&"-otxt".to_string()));
    }

    #[test]
    fn output_selection_empty_when_neither_checked() {
        assert!(OutputSelection {
            srt: false,
            txt: false
        }
        .is_empty());
        assert!(!OutputSelection {
            srt: true,
            txt: false
        }
        .is_empty());
    }

    #[test]
    fn parses_segment_end_timestamp() {
        let line = "[00:00:11.000 --> 00:00:22.480]   some transcribed text";
        assert_eq!(parse_segment_end_seconds(line), Some(22.48));
    }

    #[test]
    fn parses_segment_end_timestamp_with_hours() {
        let line = "[01:02:03.000 --> 01:02:10.500]   text";
        assert_eq!(parse_segment_end_seconds(line), Some(3730.5));
    }

    #[test]
    fn ignores_non_segment_lines() {
        assert_eq!(
            parse_segment_end_seconds("whisper_print_timings: load time = 100ms"),
            None
        );
        assert_eq!(parse_segment_end_seconds(""), None);
    }

    #[test]
    fn computes_wav_duration_from_canonical_header() {
        // 1 second of 16kHz mono 16-bit PCM: 32000 bytes of data.
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36u32 + 32000).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&16000u32.to_le_bytes()); // sample rate
        wav.extend_from_slice(&32000u32.to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&32000u32.to_le_bytes());
        wav.extend(std::iter::repeat_n(0u8, 32000));

        assert_eq!(wav_duration_secs(&wav), Some(1.0));
    }

    #[test]
    fn wav_duration_rejects_non_riff_bytes() {
        assert_eq!(wav_duration_secs(b"not a wav file"), None);
    }

    // The two stderr excerpts below are verbatim output from the pinned
    // ffmpeg sidecar run with the production args, captured against real
    // fixtures (a video-only .mov and a .mov of random bytes).
    const REAL_NO_AUDIO_STDERR: &str = "\
[out#0/wav @ 0xb3ec44180] Output file does not contain any stream
Error opening output file /tmp/fixtures/no-audio.wav.
Error opening output files: Invalid argument";

    const REAL_CORRUPTED_STDERR: &str = "\
[in#0 @ 0xb3b014000] moov atom not found
[in#0 @ 0x90ac14000] Error opening input: Invalid data found when processing input
Error opening input file /tmp/fixtures/corrupted.mov.
Error opening input files: Invalid data found when processing input";

    #[test]
    fn video_without_audio_maps_to_no_audio_track() {
        let err = classify_ffmpeg_failure(REAL_NO_AUDIO_STDERR);
        assert_eq!(err.code, TranscriptionErrorCode::NoAudioTrack);
    }

    #[test]
    fn undecodable_media_maps_to_audio_preparation_failed() {
        // Must NOT be reported as "no audio track": the file is unreadable,
        // which is a different thing to tell the user.
        let err = classify_ffmpeg_failure(REAL_CORRUPTED_STDERR);
        assert_eq!(err.code, TranscriptionErrorCode::AudioPreparationFailed);
    }

    #[test]
    fn ffmpeg_stderr_never_leaks_into_the_user_message() {
        for stderr in [REAL_NO_AUDIO_STDERR, REAL_CORRUPTED_STDERR] {
            let message = classify_ffmpeg_failure(stderr).message;
            assert!(!message.contains("0x"), "raw pointer leaked: {message}");
            assert!(!message.contains("/tmp/"), "local path leaked: {message}");
            assert!(
                !message.contains("#0"),
                "ffmpeg internals leaked: {message}"
            );
        }
    }

    #[test]
    fn required_free_space_exceeds_source_size() {
        // The extracted WAV is bounded by the source size (256 kbps), so the
        // requirement must always be strictly above it.
        assert!(required_free_bytes(1_000_000) > 1_000_000);
        assert_eq!(
            required_free_bytes(1_000_000),
            1_000_000 + DISK_SAFETY_MARGIN_BYTES
        );
    }

    #[test]
    fn required_free_space_saturates_instead_of_overflowing() {
        assert_eq!(required_free_bytes(u64::MAX), u64::MAX);
    }

    #[test]
    fn job_dir_names_are_recognized() {
        assert!(is_removable_job_dir_name("1234-56789"));
        assert!(is_removable_job_dir_name("1-1"));
    }

    #[test]
    fn non_job_dir_names_are_never_removable() {
        // Guard rail: anything that is not exactly <pid>-<nanos> must be
        // left alone, so an unrelated directory that happens to sit in the
        // temp root is never deleted.
        for name in [
            "",
            "models",
            "ST-IA",
            "-",
            "1234-",
            "-56789",
            "abcd-56789",
            "1234-abc",
            "1234_56789",
            "1234-56789-extra",
            "../escape",
            ".",
        ] {
            assert!(
                !is_removable_job_dir_name(name),
                "{name} must not be removable"
            );
        }
    }

    #[test]
    fn cancelled_states_serialize_as_plain_tags() {
        assert_eq!(
            serde_json::to_value(JobStatus::Cancelling).unwrap()["status"],
            "cancelling"
        );
        assert_eq!(
            serde_json::to_value(JobStatus::Cancelled).unwrap()["status"],
            "cancelled"
        );
    }

    #[test]
    fn insufficient_disk_space_is_a_business_error() {
        let err = TranscriptionError::insufficient_disk_space();
        assert_eq!(err.code, TranscriptionErrorCode::InsufficientDiskSpace);
        // User-facing text, never a raw errno or stderr dump.
        assert!(err.message.contains("espace disque"));
    }

    #[test]
    fn completed_status_serializes_fields_as_camel_case() {
        // Regression: an internally-tagged enum's `rename_all` only renames
        // variant names, not struct-variant field names — those need their
        // own `rename_all` on the variant itself, or fields silently stay
        // snake_case and the frontend reads `undefined` (this broke both the
        // progress display and "Ouvrir le dossier").
        let status = JobStatus::Completed {
            output_dir: "/videos/Emission-IA-sous-titres".to_string(),
            files: vec![],
            transcript_text: Some("bonjour".to_string()),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["outputDir"], "/videos/Emission-IA-sous-titres");
        assert_eq!(json["transcriptText"], "bonjour");
        assert!(json.get("output_dir").is_none());
        assert!(json.get("transcript_text").is_none());
    }
}
