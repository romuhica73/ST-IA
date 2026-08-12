import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { formatBytes } from "../media-selection/formatBytes";
import { CheckCircleIcon, SparkleIcon } from "../media-selection/icons";
import { asSupportedLanguage } from "../../i18n/locale";
import type { OutputFile, OutputLanguage } from "./types";

interface TranscriptionSuccessProps {
  files: OutputFile[];
  transcriptText: string | null;
  onNewFile: () => void;
}

export function TranscriptionSuccess({
  files,
  transcriptText,
  onNewFile,
}: TranscriptionSuccessProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);
  const [copied, setCopied] = useState(false);
  const [openFolderError, setOpenFolderError] = useState(false);

  // Preserves the backend's order (French first), without assuming both
  // versions are present.
  const languages = files.reduce<OutputLanguage[]>((acc, file) => {
    if (!acc.includes(file.language)) acc.push(file.language);
    return acc;
  }, []);

  function renderRow(file: OutputFile) {
    return (
      <div className="output-list__row" key={file.path}>
        <span className={`output-badge output-badge--${file.kind}`}>
          {file.kind.toUpperCase()}
        </span>
        <span className="output-list__name">{file.fileName}</span>
        <span className="output-list__size">{formatBytes(file.sizeBytes, language)}</span>
      </div>
    );
  }

  async function handleOpenFolder() {
    // No path is sent: the backend derives what to reveal from its own
    // completed-job state, so the frontend cannot ask Finder to open an
    // arbitrary location. Which file it picks is the same choice as before
    // (a generated file rather than the bare directory, so Finder opens on
    // the output folder with its content visible).
    setOpenFolderError(false);
    try {
      await invoke("open_output_folder");
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

      {/* Grouped by version only when there is more than one — a
          single-language result would gain a header that explains nothing. */}
      {languages.length > 1 ? (
        languages.map((outputLanguage) => (
          <div className="output-group" key={outputLanguage}>
            <p className="output-group__title">{t(`outputs.${outputLanguage}`)}</p>
            <div className="output-list">
              {files
                .filter((file) => file.language === outputLanguage)
                .map((file) => renderRow(file))}
            </div>
          </div>
        ))
      ) : (
        <div className="output-list">{files.map((file) => renderRow(file))}</div>
      )}

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
