import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelError, ModelManifest, ModelStatus } from "./types";

const EVENT_NAME = "model://event";

export function useModelManager() {
  // null = not checked yet (brief startup state, distinct from "missing").
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [manifest, setManifest] = useState<ModelManifest | null>(null);

  useEffect(() => {
    void invoke<ModelManifest>("get_model_manifest").then(setManifest);
    void invoke<ModelStatus>("get_model_status")
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

    const unlistenPromise = listen<ModelStatus>(EVENT_NAME, (event) => {
      setStatus(event.payload);
    });
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const install = useCallback(async () => {
    try {
      await invoke("install_model");
    } catch (err) {
      const error = err as ModelError;
      setStatus({
        status: "failed",
        code: error?.code ?? "networkError",
        message: error?.message ?? "",
      });
    }
  }, []);

  return { status, manifest, install };
}
