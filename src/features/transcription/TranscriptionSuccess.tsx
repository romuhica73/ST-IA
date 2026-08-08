import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { formatBytes } from "../media-selection/formatBytes";
import type { OutputFile } from "./types";

interface TranscriptionSuccessProps {
  outputDir: string;
  files: OutputFile[];
  transcriptText: string | null;
  onNewFile: () => void;
}

export function TranscriptionSuccess({
  outputDir,
  files,
  transcriptText,
  onNewFile,
}: TranscriptionSuccessProps) {
  const [copied, setCopied] = useState(false);

  async function handleOpenFolder() {
    try {
      await invoke("open_output_folder", { path: outputDir });
    } catch {
      // Non-critical: the folder path is still shown to the user below.
    }
  }

  async function handleCopy() {
    if (!transcriptText) return;
    try {
      await navigator.clipboard.writeText(transcriptText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard access can fail silently; no destructive effect either way.
    }
  }

  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon" aria-hidden="true">
          ✓
        </div>
        <p className="result-header__title">Sous-titres générés</p>
        <p className="result-header__subtitle">
          {files.length} fichier{files.length > 1 ? "s" : ""} créé{files.length > 1 ? "s" : ""} avec succès.
        </p>
      </div>

      <div className="output-list">
        {files.map((file) => (
          <div className="output-list__row" key={file.path}>
            <span className={`output-badge output-badge--${file.kind}`}>
              {file.kind.toUpperCase()}
            </span>
            <span className="output-list__name">{file.fileName}</span>
            <span className="output-list__size">{formatBytes(file.sizeBytes)}</span>
          </div>
        ))}
      </div>

      <div className="actions actions--stack">
        <button type="button" className="button button--primary" onClick={handleOpenFolder}>
          Ouvrir le dossier
        </button>
        <button
          type="button"
          className="button button--secondary"
          onClick={handleCopy}
          disabled={!transcriptText}
        >
          {copied ? "Copié !" : "Copier la transcription"}
        </button>
        <button type="button" className="drop-zone__link" onClick={onNewFile}>
          Nouveau fichier
        </button>
      </div>

      <p className="home__footer home__footer--sparkle">✨ Prêt pour DaVinci Resolve</p>
    </div>
  );
}
