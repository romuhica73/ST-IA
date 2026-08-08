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

fn is_busy(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::PreparingAudio | JobStatus::Transcribing { .. } | JobStatus::WritingOutputs
    )
}

#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    state: State<'_, JobState>,
    input: StartTranscriptionInput,
) -> Result<(), TranscriptionError> {
    {
        let current = state
            .0
            .lock()
            .map_err(|_| TranscriptionError::transcription_failed())?;
        if is_busy(&current) {
            return Err(TranscriptionError::already_running());
        }
    }

    let outputs = OutputSelection {
        srt: input.output_srt,
        txt: input.output_txt,
    };
    if outputs.is_empty() {
        return Err(TranscriptionError::no_output_selected());
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
    state
        .0
        .lock()
        .map(|guard| guard.clone())
        .map_err(|_| "transcription state lock poisoned".to_string())
}

#[tauri::command]
pub fn open_output_folder(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}
