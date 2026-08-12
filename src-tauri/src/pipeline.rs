use crate::domain::model::ModelKind;
use crate::domain::transcription::{
    build_ffmpeg_args, build_whisper_args, classify_ffmpeg_failure, output_file_name,
    parse_segment_end_seconds, required_free_bytes, resolve_output_dir, wav_duration_secs,
    JobStatus, OutputFile, OutputKind, OutputLanguage, OutputRequest, TranscribingPhase,
    TranscribingVariant, TranscriptionError,
};
use crate::model;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

pub const EVENT_NAME: &str = "transcription://event";

/// Everything the runtime needs to know about the one job ST-IA allows at a
/// time. Guarded by a single mutex that is only ever held for short,
/// non-awaiting operations (read/replace status, take the child handle, flip
/// a flag) — never across an `.await`, so a long transcription never holds a
/// lock and cannot deadlock the UI thread.
#[derive(Default)]
pub struct Job {
    pub status: JobStatus,
    /// True from the moment a job claims the slot until it releases it.
    /// Set by an atomic test-and-set in `try_claim`, so two rapid
    /// `start_transcription` calls cannot both pass the check.
    active: bool,
    cancel_requested: bool,
    /// Handle to whichever sidecar (ffmpeg or whisper-cli) is running right
    /// now. Never exposed to the frontend.
    child: Option<CommandChild>,
    /// Temp workspace of the running job, so shutdown can remove it even
    /// when the async task never gets to run its drop guard.
    temp_dir: Option<PathBuf>,
}

/// Single in-memory job slot — ST-IA supports exactly one transcription at a
/// time, per mission scope (no job queue, no database).
#[derive(Default)]
pub struct JobState(pub Mutex<Job>);

impl JobState {
    /// Atomically claims the single job slot. Returns false if a job is
    /// already active — this is the authoritative double-start guard, on the
    /// Rust side, independent of whatever the React button does.
    pub fn try_claim(&self) -> bool {
        match self.0.lock() {
            Ok(mut job) if !job.active => {
                job.active = true;
                job.cancel_requested = false;
                job.child = None;
                job.temp_dir = None;
                true
            }
            _ => false,
        }
    }

    fn release(&self) {
        if let Ok(mut job) = self.0.lock() {
            job.active = false;
            job.cancel_requested = false;
            job.child = None;
            job.temp_dir = None;
        }
    }

    pub fn is_active(&self) -> bool {
        self.0.lock().map(|job| job.active).unwrap_or(false)
    }

    pub fn status(&self) -> JobStatus {
        self.0
            .lock()
            .map(|job| job.status.clone())
            .unwrap_or(JobStatus::Idle)
    }

    fn cancel_requested(&self) -> bool {
        self.0
            .lock()
            .map(|job| job.cancel_requested)
            .unwrap_or(false)
    }

    /// Flags the job for cancellation and hands back the running child so
    /// the caller can kill it outside the lock. Returns None when there is
    /// nothing to cancel.
    pub fn request_cancel(&self) -> Option<Option<CommandChild>> {
        match self.0.lock() {
            Ok(mut job) if job.active => {
                job.cancel_requested = true;
                Some(job.child.take())
            }
            _ => None,
        }
    }

    fn set_child(&self, child: CommandChild) {
        if let Ok(mut job) = self.0.lock() {
            job.child = Some(child);
        }
    }

    fn take_child(&self) -> Option<CommandChild> {
        self.0.lock().ok().and_then(|mut job| job.child.take())
    }

    fn set_temp_dir(&self, dir: Option<PathBuf>) {
        if let Ok(mut job) = self.0.lock() {
            job.temp_dir = dir;
        }
    }

    fn temp_dir(&self) -> Option<PathBuf> {
        self.0.lock().ok().and_then(|job| job.temp_dir.clone())
    }
}

pub struct StartRequest {
    pub media_path: PathBuf,
    pub outputs: OutputRequest,
}

/// Removes the job's temporary directory when dropped, so cleanup happens
/// on every exit path (success, business error, cancellation or early
/// return) without duplicating the removal call at each return site.
struct TempJobDir(PathBuf);

impl Drop for TempJobDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Releases the job slot on every exit path of `run`, so a failure or an
/// early return can never leave the app permanently unable to start a new
/// job (this is what makes "retry after error" work without a restart).
struct JobSlotGuard<'a>(&'a JobState);

impl Drop for JobSlotGuard<'_> {
    fn drop(&mut self) {
        self.0.release();
    }
}

fn emit_status(app: &AppHandle, state: &JobState, status: JobStatus) {
    if let Ok(mut job) = state.0.lock() {
        job.status = status.clone();
    }
    let _ = app.emit(EVENT_NAME, &status);
}

fn failed(err: TranscriptionError) -> JobStatus {
    JobStatus::Failed {
        code: err.code,
        message: err.message,
    }
}

/// Free bytes on the volume holding `path`, via statvfs. Returns None when
/// the call fails — callers then skip the check rather than blocking a job
/// on an unknown value.
fn available_bytes(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `c_path` is a valid NUL-terminated string that outlives the
    // call, and `stat` is a properly sized zeroed statvfs the kernel fills.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        Some((stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64))
    }
}

/// Runs the one active job to completion, failure or cancellation.
///
/// The caller must already have claimed the slot via `JobState::try_claim`
/// — `run` only releases it (through `JobSlotGuard`), so the claim and the
/// spawn cannot race.
pub async fn run(app: AppHandle, request: StartRequest) {
    let state = app.state::<JobState>();
    let _slot = JobSlotGuard(&state);

    // Defense in depth: re-validate on the Rust side even though the
    // frontend already disables the launch button for an empty selection.
    // Both axes are required — at least one version, and at least one format.
    if request.outputs.languages.is_empty() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::no_language_selected()),
        );
        return;
    }
    if request.outputs.formats.is_empty() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::no_output_selected()),
        );
        return;
    }

    // Every model this job will need must be present *before* any work
    // starts. Discovering a missing translation model after the French pass
    // has already run would mean throwing that work away — and the atomic
    // contract means we would publish nothing for it.
    let languages = request.outputs.languages.selected();
    let mut model_paths: Vec<(OutputLanguage, PathBuf)> = Vec::new();
    for language in &languages {
        let kind = language.model_kind();
        let missing = || match kind {
            ModelKind::Transcription => TranscriptionError::model_missing(),
            ModelKind::Translation => TranscriptionError::translation_model_missing(),
        };
        if !model::model_is_installed(&app, kind).unwrap_or(false) {
            emit_status(&app, &state, failed(missing()));
            return;
        }
        match model::model_path(&app, kind) {
            Ok(path) => model_paths.push((*language, path)),
            Err(_) => {
                emit_status(&app, &state, failed(missing()));
                return;
            }
        }
    }

    let job_id = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    );
    let temp_root = crate::cleanup::temp_root().join(job_id);
    if std::fs::create_dir_all(&temp_root).is_err() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::audio_preparation_failed()),
        );
        return;
    }
    let _cleanup = TempJobDir(temp_root.clone());
    state.set_temp_dir(Some(temp_root.clone()));

    // Coarse guard against a manifestly full disk: refuse before spawning
    // ffmpeg rather than failing halfway through writing the WAV.
    if let Some(source_size) = std::fs::metadata(&request.media_path).ok().map(|m| m.len()) {
        if let Some(free) = available_bytes(&temp_root) {
            let required = required_free_bytes(source_size);
            if free < required {
                eprintln!("[st-ia] disk guard: free={free} required={required} -> refusing job");
                emit_status(
                    &app,
                    &state,
                    failed(TranscriptionError::insufficient_disk_space()),
                );
                return;
            }
        }
    }

    if cancelled_now(&app, &state) {
        return;
    }

    emit_status(&app, &state, JobStatus::PreparingAudio);

    // FFmpeg runs exactly once per job, whatever the language selection: both
    // passes read this same WAV, and it is never copied.
    let wav_path = temp_root.join("audio.wav");
    match run_ffmpeg(&app, &state, &request.media_path, &wav_path).await {
        Ok(Outcome::Cancelled) => {
            emit_cancelled(&app, &state);
            return;
        }
        Ok(Outcome::Finished) => {}
        Err(err) => {
            emit_status(&app, &state, failed(err));
            return;
        }
    }

    if cancelled_now(&app, &state) {
        return;
    }

    let total_duration_secs = std::fs::read(&wav_path)
        .ok()
        .and_then(|bytes| wav_duration_secs(&bytes));

    // The passes run strictly one after another, in this loop and nowhere
    // else. There is no spawn, no join, no second task: at any instant the
    // job owns at most one whisper-cli child, which is what keeps the M4
    // one-job/one-child guarantee true for a bilingual job as well.
    //
    // Each pass writes under its own prefix inside the workspace. Nothing
    // leaves the workspace here — publication happens once, after the last
    // pass, so cancelling or failing at any point publishes nothing at all.
    for (language, model_path) in &model_paths {
        let variant = TranscribingVariant::from(*language);
        emit_status(
            &app,
            &state,
            JobStatus::Transcribing {
                variant,
                phase: TranscribingPhase::LoadingModel,
                progress: None,
            },
        );

        let prefix = pass_prefix(&temp_root, *language);
        match run_whisper(
            &app,
            &state,
            Pass {
                model: model_path,
                wav: &wav_path,
                output_prefix: &prefix,
                language: *language,
                formats: request.outputs.formats,
                total_duration_secs,
            },
        )
        .await
        {
            Ok(Outcome::Cancelled) => {
                emit_cancelled(&app, &state);
                return;
            }
            Ok(Outcome::Finished) => {}
            Err(err) => {
                emit_status(&app, &state, failed(err));
                return;
            }
        }

        // Between passes: the previous child handle has already been taken
        // and dropped by `run_whisper`, so nothing is running right now.
        if cancelled_now(&app, &state) {
            return;
        }
    }

    emit_status(&app, &state, JobStatus::WritingOutputs);

    match write_outputs(&request.media_path, &temp_root, request.outputs) {
        Ok((output_dir, files)) => {
            // Prefer the original French text for "copy transcript" when both
            // versions exist — it is the source of truth, not the derived one.
            let transcript_text = files
                .iter()
                .find(|f| f.kind == OutputKind::Txt && f.language == OutputLanguage::French)
                .or_else(|| files.iter().find(|f| f.kind == OutputKind::Txt))
                .and_then(|f| std::fs::read_to_string(&f.path).ok());
            emit_status(
                &app,
                &state,
                JobStatus::Completed {
                    output_dir: output_dir.to_string_lossy().to_string(),
                    files,
                    transcript_text,
                },
            );
        }
        Err(_) => {
            emit_status(&app, &state, failed(TranscriptionError::write_failed()));
        }
    }
}

/// Where a pass writes inside the workspace. Distinct per language, so the
/// English pass cannot overwrite the French pass's files.
fn pass_prefix(temp_root: &Path, language: OutputLanguage) -> PathBuf {
    match language {
        OutputLanguage::French => temp_root.join("transcript-fr"),
        OutputLanguage::English => temp_root.join("transcript-en"),
    }
}

/// Whether a child process ended because we killed it, or on its own.
enum Outcome {
    Finished,
    Cancelled,
}

fn emit_cancelled(app: &AppHandle, state: &JobState) {
    emit_status(app, state, JobStatus::Cancelled);
}

/// Checkpoint between stages: if cancellation was requested, emit the
/// terminal `Cancelled` state and let the caller return (the temp dir drop
/// guard removes the workspace).
fn cancelled_now(app: &AppHandle, state: &JobState) -> bool {
    if state.cancel_requested() {
        emit_cancelled(app, state);
        true
    } else {
        false
    }
}

/// Kills the running child, if any, and marks the job cancelling. Called
/// from the `cancel_transcription` command; the pipeline task observes the
/// process exit and settles into `Cancelled`.
pub fn cancel(app: &AppHandle, state: &JobState) -> bool {
    let Some(child) = state.request_cancel() else {
        return false;
    };
    emit_status(app, state, JobStatus::Cancelling);
    if let Some(child) = child {
        let pid = child.pid();
        match child.kill() {
            Ok(()) => eprintln!("[st-ia] cancel: killed child pid {pid}"),
            Err(e) => eprintln!("[st-ia] cancel: failed to kill child pid {pid}: {e}"),
        }
    }
    true
}

/// Terminates any running child and removes the active job's temp workspace.
/// Idempotent, non-blocking — safe to call from both `ExitRequested` and
/// `Exit`, and never waits on the child (the OS reaps it as we go down).
pub fn shutdown(app: &AppHandle) {
    let state = app.state::<JobState>();
    if let Some(child) = state.take_child() {
        let pid = child.pid();
        let _ = child.kill();
        eprintln!("[st-ia] shutdown: killed child pid {pid}");
    }
    if let Some(dir) = state.temp_dir() {
        let _ = std::fs::remove_dir_all(&dir);
        eprintln!("[st-ia] shutdown: removed temp workspace {}", dir.display());
    }
    state.set_temp_dir(None);
}

// Sidecar names here are bare ("ffmpeg", not "binaries/ffmpeg"): tauri-build's
// copy_binaries step strips both the source subdirectory and the
// -aarch64-apple-darwin suffix, placing the binary flat next to the app
// executable (target/debug/ffmpeg in dev). `externalBin` in tauri.conf.json
// keeps the "binaries/" prefix — that one is a source-tree path, not the
// runtime lookup name.
// Spawned rather than run via `.output()`: `output()` drops the child
// handle internally, which would leave ffmpeg unkillable and able to outlive
// a cancelled job. We keep the handle in JobState and drain the event stream
// ourselves.
async fn run_ffmpeg(
    app: &AppHandle,
    state: &JobState,
    input: &Path,
    output_wav: &Path,
) -> Result<Outcome, TranscriptionError> {
    let args = build_ffmpeg_args(input, output_wav);
    eprintln!("[st-ia] ffmpeg args: {args:?}");
    let sidecar = app.shell().sidecar("ffmpeg").map_err(|e| {
        eprintln!("[st-ia] ffmpeg sidecar() resolution failed: {e}");
        TranscriptionError::audio_preparation_failed()
    })?;
    let (mut rx, child) = sidecar.args(args).spawn().map_err(|e| {
        eprintln!("[st-ia] ffmpeg spawn() failed: {e}");
        TranscriptionError::audio_preparation_failed()
    })?;
    eprintln!("[st-ia] ffmpeg pid {}", child.pid());
    register_child(state, child);

    let mut exit_code = None;
    let mut stderr_log = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stderr(bytes) => {
                stderr_log.push_str(&String::from_utf8_lossy(&bytes));
                stderr_log.push('\n');
            }
            CommandEvent::Terminated(payload) => exit_code = payload.code,
            _ => {}
        }
    }
    // The process is gone by the time the stream closes; drop our handle so
    // shutdown/cancel never target a stale pid.
    let _ = state.take_child();

    if state.cancel_requested() {
        eprintln!("[st-ia] ffmpeg stopped by cancellation");
        return Ok(Outcome::Cancelled);
    }

    eprintln!(
        "[st-ia] ffmpeg exit code: {exit_code:?}, output_wav exists: {}",
        output_wav.is_file()
    );

    if exit_code == Some(0) && output_wav.is_file() {
        return Ok(Outcome::Finished);
    }

    // Logged for developers only — the classified business error is what
    // reaches the user, never this text.
    eprintln!("[st-ia] ffmpeg failed:\n{stderr_log}");
    Err(classify_ffmpeg_failure(&stderr_log))
}

/// Publishes the child handle to the job slot, killing it immediately if a
/// cancellation landed in the window between spawn and registration.
fn register_child(state: &JobState, child: CommandChild) {
    if state.cancel_requested() {
        let _ = child.kill();
        return;
    }
    state.set_child(child);
    // Re-check: a cancel that ran entirely between the check above and the
    // store would otherwise have taken a None handle and left the process
    // running.
    if state.cancel_requested() {
        if let Some(child) = state.take_child() {
            let _ = child.kill();
        }
    }
}

/// Everything one Whisper pass needs. Grouped rather than passed as eight
/// positional arguments, so the two call sites cannot silently swap the
/// model and the WAV, or the two languages.
struct Pass<'a> {
    model: &'a Path,
    wav: &'a Path,
    output_prefix: &'a Path,
    language: OutputLanguage,
    formats: crate::domain::transcription::OutputFormats,
    total_duration_secs: Option<f32>,
}

async fn run_whisper(
    app: &AppHandle,
    state: &JobState,
    pass: Pass<'_>,
) -> Result<Outcome, TranscriptionError> {
    let Pass {
        model,
        wav,
        output_prefix,
        language,
        formats,
        total_duration_secs,
    } = pass;
    let variant = TranscribingVariant::from(language);
    // The two passes fail differently as far as the user is concerned, even
    // though the mechanism is identical.
    let failure = || match language {
        OutputLanguage::French => TranscriptionError::transcription_failed(),
        OutputLanguage::English => TranscriptionError::translation_failed(),
    };

    let args = build_whisper_args(model, wav, output_prefix, language, formats);
    eprintln!("[st-ia] whisper-cli args ({variant:?}): {args:?}");
    let sidecar = app.shell().sidecar("whisper-cli").map_err(|e| {
        eprintln!("[st-ia] whisper-cli sidecar() resolution failed: {e}");
        failure()
    })?;
    let (mut rx, child) = sidecar.args(args).spawn().map_err(|e| {
        eprintln!("[st-ia] whisper-cli spawn() failed: {e}");
        failure()
    })?;
    eprintln!("[st-ia] whisper-cli pid {} ({variant:?})", child.pid());
    register_child(state, child);

    let mut succeeded = false;
    let mut stderr_log = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                if let Some(end_secs) = parse_segment_end_seconds(&line) {
                    // Progress is per pass, and real: it is this pass's own
                    // position in the audio, never a synthetic blend of two
                    // passes into one global bar.
                    let progress = total_duration_secs
                        .filter(|total| *total > 0.0)
                        .map(|total| (end_secs / total).clamp(0.0, 1.0));
                    emit_status(
                        app,
                        state,
                        JobStatus::Transcribing {
                            variant,
                            phase: TranscribingPhase::Processing,
                            progress,
                        },
                    );
                }
            }
            CommandEvent::Stderr(bytes) => {
                stderr_log.push_str(&String::from_utf8_lossy(&bytes));
                stderr_log.push('\n');
            }
            CommandEvent::Terminated(payload) => {
                succeeded = payload.code == Some(0);
            }
            CommandEvent::Error(_) => {
                succeeded = false;
            }
            _ => {}
        }
    }
    // The process is gone by the time the stream closes; drop our handle so
    // the next pass starts from a clean slot and cancel/shutdown can never
    // target a stale pid.
    let _ = state.take_child();

    if state.cancel_requested() {
        eprintln!("[st-ia] whisper-cli stopped by cancellation ({variant:?})");
        return Ok(Outcome::Cancelled);
    }

    if succeeded {
        Ok(Outcome::Finished)
    } else {
        eprintln!("[st-ia] whisper-cli failed ({variant:?}):\n{stderr_log}");
        Err(failure())
    }
}

/// Publishes every requested file from the temp workspace to the sibling
/// output folder, all at once.
///
/// The folder is created here and nowhere else, so until this function
/// succeeds nothing user-visible exists — which is what makes a cancelled or
/// failed job publish nothing at all, including a bilingual job cancelled
/// during its second pass with a complete French transcript already sitting
/// in the workspace. If any copy fails, the freshly created folder is removed
/// again rather than left as a partial result that looks like a success.
fn write_outputs(
    source_media: &Path,
    temp_root: &Path,
    outputs: OutputRequest,
) -> std::io::Result<(PathBuf, Vec<OutputFile>)> {
    let output_dir = resolve_output_dir(source_media, |p| p.exists());
    std::fs::create_dir_all(&output_dir)?;

    match copy_all_outputs(source_media, temp_root, outputs, &output_dir) {
        Ok(files) => Ok((output_dir, files)),
        Err(err) => {
            // `resolve_output_dir` only ever returns a path that did not
            // exist, so removing it cannot delete pre-existing user data.
            let _ = std::fs::remove_dir_all(&output_dir);
            Err(err)
        }
    }
}

fn copy_all_outputs(
    source_media: &Path,
    temp_root: &Path,
    outputs: OutputRequest,
    output_dir: &Path,
) -> std::io::Result<Vec<OutputFile>> {
    let stem = source_media
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sous-titres");
    let bilingual = outputs.languages.is_bilingual();
    let mut files = Vec::new();

    for language in outputs.languages.selected() {
        let prefix = pass_prefix(temp_root, language);
        for kind in outputs.formats.selected() {
            files.push(copy_output(
                &prefix, output_dir, stem, language, bilingual, kind,
            )?);
        }
    }

    Ok(files)
}

fn copy_output(
    transcript_prefix: &Path,
    output_dir: &Path,
    stem: &str,
    language: OutputLanguage,
    bilingual: bool,
    kind: OutputKind,
) -> std::io::Result<OutputFile> {
    let src = transcript_prefix.with_extension(kind.extension());
    let file_name = output_file_name(stem, language, bilingual, kind);
    let dest = output_dir.join(&file_name);
    std::fs::copy(&src, &dest)?;
    let size_bytes = std::fs::metadata(&dest)?.len();
    Ok(OutputFile {
        kind,
        language,
        file_name,
        path: dest.to_string_lossy().to_string(),
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transcription::{OutputFormats, OutputLanguages};

    #[test]
    fn second_claim_is_refused_while_a_job_is_active() {
        // This is the double-start guard: the second rapid click must not
        // get a slot, independently of any frontend button state.
        let state = JobState::default();
        assert!(state.try_claim());
        assert!(!state.try_claim());
        assert!(state.is_active());
    }

    #[test]
    fn releasing_the_slot_allows_a_new_job() {
        // Retry after a failure/cancellation must work without restarting
        // the app, which is exactly "the slot was released".
        let state = JobState::default();
        assert!(state.try_claim());
        state.release();
        assert!(!state.is_active());
        assert!(state.try_claim(), "slot must be reusable after release");
    }

    #[test]
    fn slot_guard_releases_on_every_exit_path() {
        let state = JobState::default();
        assert!(state.try_claim());
        {
            let _guard = JobSlotGuard(&state);
        }
        assert!(!state.is_active());
        assert!(state.try_claim());
    }

    #[test]
    fn cancel_is_a_no_op_when_idle() {
        let state = JobState::default();
        assert!(state.request_cancel().is_none());
        assert!(!state.cancel_requested());
    }

    #[test]
    fn cancel_flags_the_active_job() {
        let state = JobState::default();
        assert!(state.try_claim());
        assert!(state.request_cancel().is_some());
        assert!(state.cancel_requested());
    }

    #[test]
    fn claiming_clears_a_previous_cancellation_flag() {
        // Otherwise the next job would immediately observe "cancelled".
        let state = JobState::default();
        assert!(state.try_claim());
        let _ = state.request_cancel();
        state.release();

        assert!(state.try_claim());
        assert!(
            !state.cancel_requested(),
            "a fresh job must not inherit the previous cancellation"
        );
    }

    #[test]
    fn temp_dir_is_tracked_for_shutdown_and_cleared_on_release() {
        let state = JobState::default();
        assert!(state.try_claim());
        state.set_temp_dir(Some(PathBuf::from("/tmp/ST-IA/1-2")));
        assert_eq!(state.temp_dir(), Some(PathBuf::from("/tmp/ST-IA/1-2")));
        state.release();
        assert_eq!(state.temp_dir(), None);
    }

    #[test]
    fn temp_dir_guard_removes_the_workspace_on_drop() {
        let root = tempfile::tempdir().unwrap();
        let job_dir = root.path().join("job");
        std::fs::create_dir_all(job_dir.join("nested")).unwrap();
        std::fs::write(job_dir.join("audio.wav"), b"x").unwrap();

        {
            let _guard = TempJobDir(job_dir.clone());
            assert!(job_dir.exists());
        }
        assert!(!job_dir.exists(), "temp workspace must be gone after drop");
    }

    fn request(french: bool, english: bool, srt: bool, txt: bool) -> OutputRequest {
        OutputRequest {
            languages: OutputLanguages { french, english },
            formats: OutputFormats { srt, txt },
        }
    }

    /// Writes what a finished whisper pass would have left in the workspace.
    fn write_pass(temp_root: &Path, language: OutputLanguage, body: &str) {
        let prefix = pass_prefix(temp_root, language);
        std::fs::write(prefix.with_extension("srt"), body).unwrap();
        std::fs::write(prefix.with_extension("txt"), body).unwrap();
    }

    #[test]
    fn failed_output_copy_leaves_no_partial_output_folder() {
        // A cancelled/failed publication must never leave a folder that
        // looks like a successful result.
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("Film.mov");
        std::fs::write(&media, b"x").unwrap();
        // No pass files exist in the workspace, so copying will fail.

        let result = write_outputs(&media, root.path(), request(true, false, true, false));

        assert!(result.is_err());
        assert!(
            !root.path().join("Film-sous-titres").exists(),
            "partial output folder must be removed"
        );
    }

    #[test]
    fn french_only_publishes_the_historical_names() {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("IMG_8484.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::French, "bonjour");

        let (dir, files) =
            write_outputs(&media, root.path(), request(true, false, true, true)).unwrap();

        assert_eq!(files.len(), 2);
        assert!(dir.join("IMG_8484.srt").is_file());
        assert!(dir.join("IMG_8484.txt").is_file());
        // No language qualifier at all when there is only one version.
        assert!(!dir.join("IMG_8484.fr.srt").exists());
        assert!(!dir.join("IMG_8484.en.srt").exists());
    }

    #[test]
    fn english_only_publishes_qualified_names() {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("IMG_8484.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::English, "hello");

        let (dir, files) =
            write_outputs(&media, root.path(), request(false, true, true, true)).unwrap();

        assert_eq!(files.len(), 2);
        assert!(dir.join("IMG_8484.en.srt").is_file());
        assert!(dir.join("IMG_8484.en.txt").is_file());
        assert!(!dir.join("IMG_8484.srt").exists());
    }

    #[test]
    fn bilingual_publishes_four_files_from_two_passes() {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("IMG_8484.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::French, "bonjour");
        write_pass(root.path(), OutputLanguage::English, "hello");

        let (dir, files) =
            write_outputs(&media, root.path(), request(true, true, true, true)).unwrap();

        assert_eq!(files.len(), 4);
        for name in [
            "IMG_8484.fr.srt",
            "IMG_8484.fr.txt",
            "IMG_8484.en.srt",
            "IMG_8484.en.txt",
        ] {
            assert!(dir.join(name).is_file(), "{name} missing");
        }
        assert!(!dir.join("IMG_8484.srt").exists());
    }

    #[test]
    fn the_two_versions_keep_their_own_content() {
        // Regression guard for the two passes sharing a workspace prefix,
        // which would silently publish the English text twice.
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("IMG_8484.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::French, "bonjour");
        write_pass(root.path(), OutputLanguage::English, "hello");

        let (dir, _) =
            write_outputs(&media, root.path(), request(true, true, false, true)).unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.join("IMG_8484.fr.txt")).unwrap(),
            "bonjour"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("IMG_8484.en.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn a_bilingual_job_publishes_nothing_when_the_second_pass_left_no_files() {
        // This is cancel-during-English and fail-during-English, at the
        // publication layer: the French pass completed and its files are in
        // the workspace, but the job as a whole did not succeed, so *no*
        // final output may appear — not even the French half.
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("IMG_8484.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::French, "bonjour");

        let result = write_outputs(&media, root.path(), request(true, true, true, true));

        assert!(result.is_err(), "a partial job must not publish");
        assert!(
            !root.path().join("IMG_8484-sous-titres").exists(),
            "no output folder may survive a partial job"
        );
    }

    #[test]
    fn only_the_requested_formats_are_published() {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("Film.mov");
        std::fs::write(&media, b"x").unwrap();
        write_pass(root.path(), OutputLanguage::French, "bonjour");

        let (dir, files) =
            write_outputs(&media, root.path(), request(true, false, true, false)).unwrap();

        assert_eq!(files.len(), 1);
        assert!(dir.join("Film.srt").is_file());
        assert!(!dir.join("Film.txt").exists());
    }

    #[test]
    fn each_pass_writes_to_its_own_workspace_prefix() {
        let root = Path::new("/tmp/job");
        assert_ne!(
            pass_prefix(root, OutputLanguage::French),
            pass_prefix(root, OutputLanguage::English)
        );
    }

    #[test]
    fn available_bytes_reports_a_real_value_for_an_existing_path() {
        let root = tempfile::tempdir().unwrap();
        let free = available_bytes(root.path());
        assert!(free.is_some(), "statvfs should succeed on a real directory");
        assert!(free.unwrap() > 0);
    }

    #[test]
    fn available_bytes_is_none_for_a_missing_path() {
        // Callers must skip the disk guard rather than block a job on an
        // unknown value.
        assert_eq!(available_bytes(Path::new("/nonexistent-st-ia-path")), None);
    }
}
