import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** The real running version (`app.package_info().version` on the Rust side
 * — the same source tauri.conf.json's `version` field feeds into
 * CFBundleShortVersionString), never a hardcoded string in React. */
export function useAppVersion() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    void invoke<string>("get_app_version")
      .then(setVersion)
      .catch((error) => {
        console.error("Failed to read app version:", error);
      });
  }, []);

  return version;
}
