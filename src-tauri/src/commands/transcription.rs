use crate::domain::media::validate_media_path;
use crate::domain::transcription::{JobStatus, OutputSelection, TranscriptionError};
use crate::pipeline::{self, JobState, StartRequest};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTranscriptionInput {
    pub media_path: String,
    pub output_srt: bool,
    pub output_txt: bool,
}

#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    state: State<'_, JobState>,
    input: StartTranscriptionInput,
) -> Result<(), TranscriptionError> {
    let outputs = OutputSelection {
        srt: input.output_srt,
        txt: input.output_txt,
    };
    if outputs.is_empty() {
        return Err(TranscriptionError::no_output_selected());
    }

    // `media_path` arrives from the WebView, so it is attacker-controlled the
    // moment any script runs there — `inspect_media` having validated it
    // earlier proves nothing, since nothing forces a caller to go through
    // that command first. Re-validating here is what actually bounds the
    // pipeline: it is the only place between the IPC boundary and
    // `build_ffmpeg_args`/`resolve_output_dir`, the latter deriving the
    // *output* directory from this same path — an unchecked value would let
    // a compromised frontend both transcribe any readable media on the disk
    // and create a `<path>-sous-titres/` folder anywhere the user can write.
    // `validate_media_path` bounds it to an existing, non-empty, readable,
    // regular file with one of the six supported extensions.
    validate_media_path(&input.media_path)
        .map_err(|_| TranscriptionError::audio_preparation_failed())?;

    // Atomic test-and-set: claiming the slot and spawning the task are not
    // separated by a window in which a second call could also see "idle".
    // Two rapid clicks therefore produce exactly one job, regardless of what
    // the frontend does with its button.
    if !state.try_claim() {
        return Err(TranscriptionError::already_running());
    }

    let request = StartRequest {
        media_path: input.media_path.into(),
        outputs,
    };

    tauri::async_runtime::spawn(pipeline::run(app, request));

    Ok(())
}

#[tauri::command]
pub fn get_transcription_status(state: State<'_, JobState>) -> Result<JobStatus, String> {
    Ok(state.status())
}

/// Requests cancellation of the running job: kills the active sidecar and
/// lets the pipeline settle into `Cancelled` once the process is actually
/// gone and the temp workspace has been removed. A no-op when idle.
#[tauri::command]
pub fn cancel_transcription(app: AppHandle, state: State<'_, JobState>) -> Result<(), String> {
    pipeline::cancel(&app, &state);
    Ok(())
}

/// Reveals the current job's generated output in Finder.
///
/// Takes no path: the target is derived entirely from the backend's own
/// `Completed` job state, so there is no frontend-supplied filesystem path to
/// validate in the first place. Previously the frontend passed the path it
/// wanted revealed, which meant any script running in the WebView could ask
/// Finder to open an arbitrary location and use the ok/err answer to probe
/// what exists on disk. Revealing a *file* (rather than opening the bare
/// directory) is still the mechanism — it opens the containing folder with
/// that file selected, and keeps us on `reveal-item-in-dir` alone, never the
/// opener plugin's broader open-path permission.
#[tauri::command]
pub fn open_output_folder(app: AppHandle, state: State<'_, JobState>) -> Result<(), String> {
    let JobStatus::Completed {
        output_dir, files, ..
    } = state.status()
    else {
        return Err("no completed transcription to reveal".to_string());
    };

    let target = files
        .first()
        .map(|file| file.path.clone())
        .unwrap_or(output_dir);

    app.opener()
        .reveal_item_in_dir(target)
        .map_err(|e| e.to_string())
}
