import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { JobStatus, StartTranscriptionInput, TranscriptionError } from "./types";

const EVENT_NAME = "transcription://event";

export function useTranscription() {
  const [status, setStatus] = useState<JobStatus>({ status: "idle" });

  useEffect(() => {
    const unlistenPromise = listen<JobStatus>(EVENT_NAME, (event) => {
      setStatus(event.payload);
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const start = useCallback(async (input: StartTranscriptionInput) => {
    try {
      // Commands validated synchronously in Rust (already running, no
      // output selected) reject the promise; everything after that point
      // arrives as a "transcription://event" and updates `status` above.
      await invoke("start_transcription", { input });
    } catch (err) {
      const error = err as TranscriptionError;
      setStatus({
        status: "failed",
        code: error?.code ?? "transcriptionFailed",
        message: error?.message ?? "Une erreur inattendue est survenue.",
      });
    }
  }, []);

  const reset = useCallback(() => {
    setStatus({ status: "idle" });
  }, []);

  return { status, start, reset };
}
