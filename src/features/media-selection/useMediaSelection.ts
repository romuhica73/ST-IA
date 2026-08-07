import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { SUPPORTED_EXTENSIONS, MULTIPLE_FILES_ERROR_MESSAGE } from "./constants";
import type { MediaError, MediaInfo, MediaSelectionState } from "./types";

export function useMediaSelection() {
  const [state, setState] = useState<MediaSelectionState>({ status: "idle" });
  // Ignore drag-drop events once a media is already selected, so a stray
  // drag over the window can't silently replace it.
  const stateRef = useRef(state);
  stateRef.current = state;

  const inspectPath = useCallback(async (path: string) => {
    try {
      const media = await invoke<MediaInfo>("inspect_media", { path });
      setState({ status: "selected", media });
    } catch (err) {
      const mediaError = err as MediaError;
      setState({
        status: "error",
        message: mediaError?.message ?? "Une erreur inattendue est survenue.",
      });
    }
  }, []);

  useEffect(() => {
    const unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
      if (stateRef.current.status === "selected") return;

      switch (event.payload.type) {
        case "enter":
        case "over":
          setState({ status: "dragging" });
          break;
        case "leave":
          setState((current) =>
            current.status === "dragging" ? { status: "idle" } : current,
          );
          break;
        case "drop": {
          const { paths } = event.payload;
          if (paths.length !== 1) {
            setState({ status: "error", message: MULTIPLE_FILES_ERROR_MESSAGE });
            break;
          }
          void inspectPath(paths[0]);
          break;
        }
      }
    });

    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [inspectPath]);

  const selectViaDialog = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Média", extensions: [...SUPPORTED_EXTENSIONS] }],
    });

    if (selected === null) return; // user cancelled: no state change
    await inspectPath(selected);
  }, [inspectPath]);

  const reset = useCallback(() => {
    setState({ status: "idle" });
  }, []);

  return { state, selectViaDialog, reset };
}
