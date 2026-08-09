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
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum JobStatus {
    Idle,
    PreparingAudio,
    Transcribing {
        phase: TranscribingPhase,
        progress: Option<f32>,
    },
    WritingOutputs,
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
        wav.extend(std::iter::repeat(0u8).take(32000));

        assert_eq!(wav_duration_secs(&wav), Some(1.0));
    }

    #[test]
    fn wav_duration_rejects_non_riff_bytes() {
        assert_eq!(wav_duration_secs(b"not a wav file"), None);
    }
}
