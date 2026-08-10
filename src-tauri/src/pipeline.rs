use crate::domain::transcription::{
    build_ffmpeg_args, build_whisper_args, classify_ffmpeg_failure, parse_segment_end_seconds,
    required_free_bytes, resolve_output_dir, wav_duration_secs, JobStatus, OutputFile, OutputKind,
    OutputSelection, TranscribingPhase, TranscriptionError,
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
    pub outputs: OutputSelection,
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
    if request.outputs.is_empty() {
        emit_status(
            &app,
            &state,
            failed(TranscriptionError::no_output_selected()),
        );
        return;
    }

    if !model::model_is_installed(&app).unwrap_or(false) {
        emit_status(&app, &state, failed(TranscriptionError::model_missing()));
        return;
    }
    let model_path = match model::model_path(&app) {
        Ok(path) => path,
        Err(_) => {
            emit_status(&app, &state, failed(TranscriptionError::model_missing()));
            return;
        }
    };

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

    emit_status(
        &app,
        &state,
        JobStatus::Transcribing {
            phase: TranscribingPhase::LoadingModel,
            progress: None,
        },
    );

    let transcript_prefix = temp_root.join("transcript");
    match run_whisper(
        &app,
        &state,
        &model_path,
        &wav_path,
        &transcript_prefix,
        request.outputs,
        total_duration_secs,
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

    // Last checkpoint before anything becomes visible to the user: whisper
    // wrote its files inside the temp workspace only, so cancelling here
    // still publishes nothing.
    if cancelled_now(&app, &state) {
        return;
    }

    emit_status(&app, &state, JobStatus::WritingOutputs);

    match write_outputs(&request.media_path, &transcript_prefix, request.outputs) {
        Ok((output_dir, files)) => {
            let transcript_text = files
                .iter()
                .find(|f| f.kind == OutputKind::Txt)
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

async fn run_whisper(
    app: &AppHandle,
    state: &JobState,
    model: &Path,
    wav: &Path,
    output_prefix: &Path,
    outputs: OutputSelection,
    total_duration_secs: Option<f32>,
) -> Result<Outcome, TranscriptionError> {
    let args = build_whisper_args(model, wav, output_prefix, outputs);
    eprintln!("[st-ia] whisper-cli args: {args:?}");
    let sidecar = app.shell().sidecar("whisper-cli").map_err(|e| {
        eprintln!("[st-ia] whisper-cli sidecar() resolution failed: {e}");
        TranscriptionError::transcription_failed()
    })?;
    let (mut rx, child) = sidecar.args(args).spawn().map_err(|e| {
        eprintln!("[st-ia] whisper-cli spawn() failed: {e}");
        TranscriptionError::transcription_failed()
    })?;
    eprintln!("[st-ia] whisper-cli pid {}", child.pid());
    register_child(state, child);

    let mut succeeded = false;
    let mut stderr_log = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                if let Some(end_secs) = parse_segment_end_seconds(&line) {
                    let progress = total_duration_secs
                        .filter(|total| *total > 0.0)
                        .map(|total| (end_secs / total).clamp(0.0, 1.0));
                    emit_status(
                        app,
                        state,
                        JobStatus::Transcribing {
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
    let _ = state.take_child();

    if state.cancel_requested() {
        eprintln!("[st-ia] whisper-cli stopped by cancellation");
        return Ok(Outcome::Cancelled);
    }

    if succeeded {
        Ok(Outcome::Finished)
    } else {
        eprintln!("[st-ia] whisper-cli failed:\n{stderr_log}");
        Err(TranscriptionError::transcription_failed())
    }
}

/// Publishes the transcript files from the temp workspace to the sibling
/// output folder. The folder is created here and nowhere else, so until this
/// function succeeds nothing user-visible exists; if any copy fails the
/// freshly created folder is removed again rather than left as a partial
/// result that looks like a success.
fn write_outputs(
    source_media: &Path,
    transcript_prefix: &Path,
    outputs: OutputSelection,
) -> std::io::Result<(PathBuf, Vec<OutputFile>)> {
    let output_dir = resolve_output_dir(source_media, |p| p.exists());
    std::fs::create_dir_all(&output_dir)?;

    match copy_all_outputs(source_media, transcript_prefix, outputs, &output_dir) {
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
    transcript_prefix: &Path,
    outputs: OutputSelection,
    output_dir: &Path,
) -> std::io::Result<Vec<OutputFile>> {
    let stem = source_media
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sous-titres");
    let mut files = Vec::new();

    if outputs.srt {
        files.push(copy_output(
            transcript_prefix,
            "srt",
            output_dir,
            stem,
            OutputKind::Srt,
        )?);
    }
    if outputs.txt {
        files.push(copy_output(
            transcript_prefix,
            "txt",
            output_dir,
            stem,
            OutputKind::Txt,
        )?);
    }

    Ok(files)
}

fn copy_output(
    transcript_prefix: &Path,
    extension: &str,
    output_dir: &Path,
    stem: &str,
    kind: OutputKind,
) -> std::io::Result<OutputFile> {
    let src = transcript_prefix.with_extension(extension);
    let file_name = format!("{stem}.{extension}");
    let dest = output_dir.join(&file_name);
    std::fs::copy(&src, &dest)?;
    let size_bytes = std::fs::metadata(&dest)?.len();
    Ok(OutputFile {
        kind,
        file_name,
        path: dest.to_string_lossy().to_string(),
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transcription::OutputSelection;

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

    #[test]
    fn failed_output_copy_leaves_no_partial_output_folder() {
        // A cancelled/failed publication must never leave a folder that
        // looks like a successful result.
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("Film.mov");
        std::fs::write(&media, b"x").unwrap();
        // No transcript files exist at this prefix, so copying will fail.
        let prefix = root.path().join("transcript");

        let result = write_outputs(
            &media,
            &prefix,
            OutputSelection {
                srt: true,
                txt: false,
            },
        );

        assert!(result.is_err());
        assert!(
            !root.path().join("Film-sous-titres").exists(),
            "partial output folder must be removed"
        );
    }

    #[test]
    fn successful_copy_publishes_requested_outputs_only() {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("Film.mov");
        std::fs::write(&media, b"x").unwrap();
        let prefix = root.path().join("transcript");
        std::fs::write(prefix.with_extension("srt"), b"1\n").unwrap();
        std::fs::write(prefix.with_extension("txt"), b"bonjour").unwrap();

        let (dir, files) = write_outputs(
            &media,
            &prefix,
            OutputSelection {
                srt: true,
                txt: false,
            },
        )
        .unwrap();

        assert_eq!(files.len(), 1);
        assert!(dir.join("Film.srt").is_file());
        assert!(!dir.join("Film.txt").exists());
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
