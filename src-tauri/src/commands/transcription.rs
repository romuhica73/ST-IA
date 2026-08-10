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

/// Reveals a generated output file in Finder. `path` is expected to be one
/// of the `.srt`/`.txt` files just written (not the bare directory) — the
/// opener plugin reveals the containing folder with that file selected,
/// which is the officially recommended way to "open a folder" and avoids
/// depending on the plugin's separate open-path permission/scope.
#[tauri::command]
pub fn open_output_folder(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|e| e.to_string())
}
