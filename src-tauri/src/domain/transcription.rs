use crate::domain::model::ModelKind;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The language spoken in the media. One value for now: the qualified scope
/// is French audio (ADR-001). This is *not* the output language, and not the
/// interface language — see ADR-010 for why the three are kept apart.
pub const SOURCE_LANGUAGE: &str = "fr";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputKind {
    Srt,
    Txt,
}

impl OutputKind {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Srt => "srt",
            Self::Txt => "txt",
        }
    }
}

/// A version of the transcript the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputLanguage {
    /// The original French transcription.
    French,
    /// The English translation, produced locally by a second Whisper pass.
    English,
}

impl OutputLanguage {
    /// Which model this version needs. The two differ: the fast turbo model
    /// cannot translate at all (ADR-008).
    pub fn model_kind(self) -> ModelKind {
        match self {
            Self::French => ModelKind::Transcription,
            Self::English => ModelKind::Translation,
        }
    }

    /// Whisper's `--translate` flag applies to the English version only.
    pub fn translates(self) -> bool {
        matches!(self, Self::English)
    }
}

/// Which formats to write. Independent of the language selection: the number
/// of files produced is languages × formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputFormats {
    pub srt: bool,
    pub txt: bool,
}

impl OutputFormats {
    pub fn is_empty(&self) -> bool {
        !self.srt && !self.txt
    }

    pub fn selected(&self) -> Vec<OutputKind> {
        let mut kinds = Vec::new();
        if self.srt {
            kinds.push(OutputKind::Srt);
        }
        if self.txt {
            kinds.push(OutputKind::Txt);
        }
        kinds
    }
}

/// Which versions to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputLanguages {
    pub french: bool,
    pub english: bool,
}

impl OutputLanguages {
    pub fn is_empty(&self) -> bool {
        !self.french && !self.english
    }

    /// Both requested — the case that changes the French file names, because
    /// leaving one version unqualified would be ambiguous.
    pub fn is_bilingual(&self) -> bool {
        self.french && self.english
    }

    /// In pipeline order: French first. Not cosmetic — the cheap pass runs
    /// first so a cancellation during the expensive translation has already
    /// been given the maximum chance to happen earlier.
    pub fn selected(&self) -> Vec<OutputLanguage> {
        let mut langs = Vec::new();
        if self.french {
            langs.push(OutputLanguage::French);
        }
        if self.english {
            langs.push(OutputLanguage::English);
        }
        langs
    }
}

/// The complete output request for one job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputRequest {
    pub languages: OutputLanguages,
    pub formats: OutputFormats,
}

impl OutputRequest {
    /// Total number of files this request will publish — languages ×
    /// formats. The frontend shows this on its launch button; the rule lives
    /// here as the reference the TypeScript mirror is tested against.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn file_count(&self) -> usize {
        self.languages.selected().len() * self.formats.selected().len()
    }
}

/// Which pass is running. `Original` behaves exactly as the single pass did
/// before bilingual output existed, so a French-only job is indistinguishable
/// from the pre-M9 behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TranscribingVariant {
    Original,
    EnglishTranslation,
}

impl From<OutputLanguage> for TranscribingVariant {
    fn from(language: OutputLanguage) -> Self {
        match language {
            OutputLanguage::French => Self::Original,
            OutputLanguage::English => Self::EnglishTranslation,
        }
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
    pub language: OutputLanguage,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
}

/// The output naming contract (ADR-010).
///
/// Historical compatibility is the constraint: a French-only job must keep
/// producing exactly the names it produced before English output existed, so
/// existing user workflows and any scripts around them keep working.
///
/// * French only    → `IMG_8484.srt`
/// * English only   → `IMG_8484.en.srt`
/// * French+English → `IMG_8484.fr.srt` and `IMG_8484.en.srt`
///
/// The French version only becomes `.fr.` when there is an English one to
/// tell it apart from; qualifying it in isolation would be noise, and would
/// break the historical names for no benefit.
pub fn output_file_name(
    stem: &str,
    language: OutputLanguage,
    bilingual: bool,
    kind: OutputKind,
) -> String {
    let extension = kind.extension();
    match (language, bilingual) {
        (OutputLanguage::French, false) => format!("{stem}.{extension}"),
        (OutputLanguage::French, true) => format!("{stem}.fr.{extension}"),
        (OutputLanguage::English, _) => format!("{stem}.en.{extension}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TranscriptionErrorCode {
    AlreadyRunning,
    ModelMissing,
    /// English output was requested but its (separate, larger) model is not
    /// installed. Distinct from `ModelMissing` so the UI can offer the right
    /// download rather than the wrong one.
    TranslationModelMissing,
    NoOutputSelected,
    NoLanguageSelected,
    AudioPreparationFailed,
    NoAudioTrack,
    TranscriptionFailed,
    TranslationFailed,
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

    pub fn no_language_selected() -> Self {
        Self::new(
            TranscriptionErrorCode::NoLanguageSelected,
            "Sélectionnez au moins une version à générer.",
        )
    }

    pub fn translation_model_missing() -> Self {
        Self::new(
            TranscriptionErrorCode::TranslationModelMissing,
            "Le modèle de traduction anglaise n'est pas encore installé.",
        )
    }

    pub fn translation_failed() -> Self {
        Self::new(
            TranscriptionErrorCode::TranslationFailed,
            "La traduction anglaise a échoué.",
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
    #[serde(rename_all = "camelCase")]
    Transcribing {
        /// Which pass this is. A French-only job only ever reports
        /// `original`, so its progress UI is unchanged from before M9.
        variant: TranscribingVariant,
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

/// Arguments for the whisper-cli sidecar.
///
/// The source language is always French — that is the qualified scope, and
/// it is the *input* language in both passes. What differs between the two
/// passes is the task: the English version adds `-tr`, which whisper.cpp
/// documents as "translate from source language to english".
///
/// The model differs too, and must: `large-v3-turbo` ignores `-tr` and
/// returns French (ADR-008), so the English pass is given the full
/// `large-v3`. Passing the model in rather than resolving it here keeps this
/// function pure and testable.
///
/// `output_prefix` is a path without extension (whisper-cli appends
/// `.srt`/`.txt` itself).
pub fn build_whisper_args(
    model: &Path,
    wav: &Path,
    output_prefix: &Path,
    language: OutputLanguage,
    formats: OutputFormats,
) -> Vec<String> {
    let mut args = vec![
        "-m".to_string(),
        model.to_string_lossy().to_string(),
        "-f".to_string(),
        wav.to_string_lossy().to_string(),
        "-l".to_string(),
        SOURCE_LANGUAGE.to_string(),
        "-of".to_string(),
        output_prefix.to_string_lossy().to_string(),
    ];
    if language.translates() {
        args.push("-tr".to_string());
    }
    if formats.srt {
        args.push("-osrt".to_string());
    }
    if formats.txt {
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
    use crate::domain::model::ModelKind;

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
    fn french_pass_uses_the_transcription_model_and_no_translate_flag() {
        let args = build_whisper_args(
            Path::new("/models/ggml-large-v3-turbo-q5_0.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/fr"),
            OutputLanguage::French,
            OutputFormats {
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
                "/tmp/fr",
                "-osrt",
            ]
        );
        assert!(!args.contains(&"-tr".to_string()));
    }

    #[test]
    fn english_pass_adds_translate_and_keeps_french_as_the_source_language() {
        // `-l` is the *spoken* language in both passes; `-tr` is what makes
        // the output English. Setting `-l en` instead would tell Whisper the
        // audio is English, which it is not.
        let args = build_whisper_args(
            Path::new("/models/ggml-large-v3.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/en"),
            OutputLanguage::English,
            OutputFormats {
                srt: true,
                txt: true,
            },
        );
        assert_eq!(
            args,
            vec![
                "-m",
                "/models/ggml-large-v3.bin",
                "-f",
                "/tmp/audio.wav",
                "-l",
                "fr",
                "-of",
                "/tmp/en",
                "-tr",
                "-osrt",
                "-otxt",
            ]
        );
    }

    #[test]
    fn the_two_passes_read_the_same_wav() {
        // FFmpeg runs once per job; a second extraction would be pure waste
        // and could drift from the first.
        let wav = Path::new("/tmp/job/audio.wav");
        let fr = build_whisper_args(
            Path::new("/m/turbo.bin"),
            wav,
            Path::new("/tmp/fr"),
            OutputLanguage::French,
            OutputFormats {
                srt: true,
                txt: false,
            },
        );
        let en = build_whisper_args(
            Path::new("/m/large.bin"),
            wav,
            Path::new("/tmp/en"),
            OutputLanguage::English,
            OutputFormats {
                srt: true,
                txt: false,
            },
        );
        let input_of =
            |args: &[String]| args[args.iter().position(|a| a == "-f").unwrap() + 1].clone();
        assert_eq!(input_of(&fr), input_of(&en));
    }

    #[test]
    fn each_language_maps_to_its_own_model() {
        // The whole reason two models exist.
        assert_eq!(
            OutputLanguage::French.model_kind(),
            ModelKind::Transcription
        );
        assert_eq!(OutputLanguage::English.model_kind(), ModelKind::Translation);
        assert!(!OutputLanguage::French.translates());
        assert!(OutputLanguage::English.translates());
    }

    #[test]
    fn the_two_passes_write_to_different_prefixes() {
        // Same prefix would make the second pass overwrite the first's files
        // inside the workspace, silently losing the French version.
        let fr = build_whisper_args(
            Path::new("/m/turbo.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/job/fr"),
            OutputLanguage::French,
            OutputFormats {
                srt: true,
                txt: false,
            },
        );
        let en = build_whisper_args(
            Path::new("/m/large.bin"),
            Path::new("/tmp/audio.wav"),
            Path::new("/tmp/job/en"),
            OutputLanguage::English,
            OutputFormats {
                srt: true,
                txt: false,
            },
        );
        let prefix_of =
            |args: &[String]| args[args.iter().position(|a| a == "-of").unwrap() + 1].clone();
        assert_ne!(prefix_of(&fr), prefix_of(&en));
    }

    #[test]
    fn formats_are_empty_only_when_neither_is_checked() {
        assert!(OutputFormats {
            srt: false,
            txt: false
        }
        .is_empty());
        assert!(!OutputFormats {
            srt: true,
            txt: false
        }
        .is_empty());
        assert!(!OutputFormats {
            srt: false,
            txt: true
        }
        .is_empty());
    }

    #[test]
    fn languages_are_empty_only_when_neither_is_checked() {
        assert!(OutputLanguages {
            french: false,
            english: false
        }
        .is_empty());
        assert!(!OutputLanguages {
            french: true,
            english: false
        }
        .is_empty());
        assert!(!OutputLanguages {
            french: false,
            english: true
        }
        .is_empty());
    }

    #[test]
    fn french_runs_before_english() {
        // Cheap pass first: a job cancelled during the long translation has
        // already had its best chance to be cancelled earlier.
        let both = OutputLanguages {
            french: true,
            english: true,
        };
        assert_eq!(
            both.selected(),
            vec![OutputLanguage::French, OutputLanguage::English]
        );
        assert!(both.is_bilingual());
    }

    #[test]
    fn a_single_language_is_not_bilingual() {
        assert!(!OutputLanguages {
            french: true,
            english: false
        }
        .is_bilingual());
        assert!(!OutputLanguages {
            french: false,
            english: true
        }
        .is_bilingual());
    }

    #[test]
    fn file_count_is_languages_times_formats() {
        let request = |french, english, srt, txt| OutputRequest {
            languages: OutputLanguages { french, english },
            formats: OutputFormats { srt, txt },
        };
        assert_eq!(request(true, false, true, true).file_count(), 2);
        assert_eq!(request(false, true, true, true).file_count(), 2);
        assert_eq!(request(true, true, true, true).file_count(), 4);
        assert_eq!(request(true, true, true, false).file_count(), 2);
        assert_eq!(request(true, false, true, false).file_count(), 1);
    }

    #[test]
    fn french_only_keeps_the_historical_file_names() {
        // The compatibility guarantee: a French-only job must produce exactly
        // what it produced before English output existed.
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::French, false, OutputKind::Srt),
            "IMG_8484.srt"
        );
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::French, false, OutputKind::Txt),
            "IMG_8484.txt"
        );
    }

    #[test]
    fn english_only_is_always_qualified() {
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::English, false, OutputKind::Srt),
            "IMG_8484.en.srt"
        );
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::English, false, OutputKind::Txt),
            "IMG_8484.en.txt"
        );
    }

    #[test]
    fn bilingual_qualifies_both_versions() {
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::French, true, OutputKind::Srt),
            "IMG_8484.fr.srt"
        );
        assert_eq!(
            output_file_name("IMG_8484", OutputLanguage::English, true, OutputKind::Srt),
            "IMG_8484.en.srt"
        );
    }

    #[test]
    fn no_two_requested_files_ever_share_a_name() {
        // Any collision would mean one file silently overwriting another in
        // the published output folder.
        for (french, english) in [(true, false), (false, true), (true, true)] {
            let languages = OutputLanguages { french, english };
            let formats = OutputFormats {
                srt: true,
                txt: true,
            };
            let mut names: Vec<String> = Vec::new();
            for language in languages.selected() {
                for kind in formats.selected() {
                    names.push(output_file_name(
                        "IMG_8484",
                        language,
                        languages.is_bilingual(),
                        kind,
                    ));
                }
            }
            let unique: std::collections::HashSet<&String> = names.iter().collect();
            assert_eq!(unique.len(), names.len(), "name collision in {names:?}");
        }
    }

    #[test]
    fn transcribing_variant_follows_the_language() {
        assert_eq!(
            TranscribingVariant::from(OutputLanguage::French),
            TranscribingVariant::Original
        );
        assert_eq!(
            TranscribingVariant::from(OutputLanguage::English),
            TranscribingVariant::EnglishTranslation
        );
    }

    #[test]
    fn transcribing_status_serializes_variant_and_phase() {
        let status = JobStatus::Transcribing {
            variant: TranscribingVariant::EnglishTranslation,
            phase: TranscribingPhase::Processing,
            progress: Some(0.42),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["status"], "transcribing");
        assert_eq!(json["variant"], "englishTranslation");
        assert_eq!(json["phase"], "processing");
    }

    #[test]
    fn missing_translation_model_is_its_own_error() {
        // Distinct from ModelMissing so the UI offers the 3.1 GB translation
        // download rather than re-offering the transcription model.
        assert_eq!(
            TranscriptionError::translation_model_missing().code,
            TranscriptionErrorCode::TranslationModelMissing
        );
        assert_ne!(
            TranscriptionError::translation_model_missing().code,
            TranscriptionError::model_missing().code
        );
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
