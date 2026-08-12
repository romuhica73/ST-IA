import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Tells Rust the main window has decided what to render, which is one half
 * of the splash handover (see ADR-009). The other half is the splash's own
 * animation reaching its end; whichever arrives last triggers the cut.
 *
 * "Ready" is deliberately defined as *the first screen the user will actually
 * see has been decided* — that is, the model status query has come back.
 * Before that, App renders a blank `<main>` placeholder, and revealing the
 * window then is exactly the startup flash the splash exists to remove.
 *
 * Fires once per launch. If the signal never gets sent — an exception here,
 * a webview that failed to boot — Rust's watchdog shows the window anyway,
 * so this hook can never leave the app unusable.
 */
export function useSplashHandoff(ready: boolean) {
  const sent = useRef(false);

  useEffect(() => {
    if (!ready || sent.current) return;
    sent.current = true;
    void invoke("notify_ui_ready").catch((error) => {
      // Not recoverable from here and not worth surfacing to the user: the
      // backend watchdog is the safety net.
      console.error("Failed to signal UI readiness:", error);
    });
  }, [ready]);
}
