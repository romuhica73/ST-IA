import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { formatBytes } from "../media-selection/formatBytes";
import { CheckCircleIcon, SparkleIcon } from "../media-selection/icons";
import { asSupportedLanguage } from "../../i18n/locale";
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
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);
  const [copied, setCopied] = useState(false);
  const [openFolderError, setOpenFolderError] = useState(false);

  async function handleOpenFolder() {
    // Reveal one of the generated files rather than the bare directory, so
    // Finder opens directly on the output folder with its content visible.
    const target = files[0]?.path ?? outputDir;
    setOpenFolderError(false);
    try {
      await invoke("open_output_folder", { path: target });
    } catch (error) {
      console.error("Failed to reveal output folder in Finder:", error);
      setOpenFolderError(true);
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
        <div className="result-header__icon result-header__icon--success" aria-hidden="true">
          <CheckCircleIcon />
        </div>
        <p className="result-header__title">{t("success.title")}</p>
        <p className="result-header__subtitle">
          {t("success.subtitle", { count: files.length })}
        </p>
      </div>

      <div className="output-list">
        {files.map((file) => (
          <div className="output-list__row" key={file.path}>
            <span className={`output-badge output-badge--${file.kind}`}>
              {file.kind.toUpperCase()}
            </span>
            <span className="output-list__name">{file.fileName}</span>
            <span className="output-list__size">{formatBytes(file.sizeBytes, language)}</span>
          </div>
        ))}
      </div>

      <div className="actions actions--stack">
        <button type="button" className="button button--primary" onClick={handleOpenFolder}>
          {t("success.openFolder")}
        </button>
        {openFolderError && (
          <p className="drop-zone__error" role="alert">
            {t("success.openFolderError")}
          </p>
        )}
        <button
          type="button"
          className="button button--secondary"
          onClick={handleCopy}
          disabled={!transcriptText}
        >
          {copied ? t("success.copied") : t("success.copyTranscript")}
        </button>
        <button type="button" className="drop-zone__link" onClick={onNewFile}>
          {t("success.newFile")}
        </button>
      </div>

      <p className="home__footer home__footer--sparkle">
        <SparkleIcon />
        {t("success.readyForDavinci")}
      </p>
    </div>
  );
}
