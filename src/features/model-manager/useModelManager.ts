import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelError, ModelKind, ModelManifest, ModelStatus, ModelStatusEvent } from "./types";

const EVENT_NAME = "model://event";

/**
 * Tracks one model's presence and download.
 *
 * Instantiated once per model kind. The two are genuinely independent: the
 * transcription model gates every job, while the translation model is only
 * fetched if the user asks for English output — so a single shared status
 * would have to answer "ready?" for two different questions at once.
 *
 * `enabled` exists because detection is not free: it verifies the file's
 * SHA-256, which is ~9 seconds of I/O for the 3.1 GB translation model.
 * Doing that at every launch would make startup slower for the majority of
 * users, who never ask for English output. The translation model is
 * therefore only checked once a media file has been selected — by which
 * point the answer is needed, and the cost is hidden behind the user
 * choosing output languages.
 */
export function useModelManager(kind: ModelKind, enabled = true) {
  // null = not checked yet (brief startup state, distinct from "missing").
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [manifest, setManifest] = useState<ModelManifest | null>(null);

  useEffect(() => {
    if (!enabled) return;
    void invoke<ModelManifest>("get_model_manifest", { kind }).then(setManifest);
    void invoke<ModelStatus>("get_model_status", { kind })
      .then(setStatus)
      .catch((err) => {
        const error = err as ModelError;
        // `message` is not displayed (the UI translates from `code` — see
        // ModelRequired.tsx / ADR-007); kept only because ModelStatus's
        // wire shape requires it.
        setStatus({
          status: "failed",
          code: error?.code ?? "writeError",
          message: error?.message ?? "",
        });
      });

    // One event channel carries both models; ignore the other one's
    // progress rather than rendering it as this model's.
    const unlistenPromise = listen<ModelStatusEvent>(EVENT_NAME, (event) => {
      if (event.payload.kind !== kind) return;
      setStatus(event.payload);
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [kind, enabled]);

  const install = useCallback(async () => {
    try {
      await invoke("install_model", { kind });
    } catch (err) {
      const error = err as ModelError;
      setStatus({
        status: "failed",
        code: error?.code ?? "networkError",
        message: error?.message ?? "",
      });
    }
  }, [kind]);

  return { status, manifest, install };
}
