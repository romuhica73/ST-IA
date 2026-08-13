import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { formatBytes } from "./formatBytes";
import { ChevronDownIcon, FileIcon, GlobeIcon, SizeIcon, TypeIcon } from "./icons";
import { OutputLanguageCards } from "./OutputLanguageCards";
import { DEFAULT_OUTPUTS, fileCount, hasFormat, hasLanguage, isLaunchable } from "./outputs";
import type { OutputSelection } from "./outputs";
import { asSupportedLanguage } from "../../i18n/locale";
import type { MediaInfo } from "./types";

interface SelectedMediaProps {
  media: MediaInfo;
  modelReady: boolean;
  /** Whether the English translation model is installed. English can still
   * be selected without it — the download is offered inline instead of the
   * checkbox being disabled with no explanation. */
  translationModelReady: boolean;
  translationModelSize: number | null;
  translationModelBusy: boolean;
  onDownloadTranslationModel: () => void;
  onChangeFile: () => void;
  onGenerate: (outputs: OutputSelection) => void;
}

export function SelectedMedia({
  media,
  modelReady,
  translationModelReady,
  translationModelSize,
  translationModelBusy,
  onDownloadTranslationModel,
  onChangeFile,
  onGenerate,
}: SelectedMediaProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);
  const languageId = useId();
  const [outputs, setOutputs] = useState<OutputSelection>(DEFAULT_OUTPUTS);

  const count = fileCount(outputs);
  const needsTranslationModel = outputs.languages.english && !translationModelReady;
  const canLaunch = isLaunchable(outputs) && modelReady && !needsTranslationModel;

  return (
    <div className="job">
      <div className="file-card">
        <FileIcon kind={media.kind} />
        <div className="file-card__info">
          <p className="file-card__name">{media.fileName}</p>
          <div className="file-card__meta">
            <span className="meta-item">
              <TypeIcon />
              {media.extension.toUpperCase()}
            </span>
            <span className="meta-item">
              <SizeIcon />
              {formatBytes(media.sizeBytes, language)}
            </span>
          </div>
        </div>
      </div>

      <section className="field">
        <label className="field__label" htmlFor={languageId}>
          {t("transcription.language")}
        </label>
        <div className="select">
          <GlobeIcon />
          <select id={languageId} value="fr" onChange={() => {}}>
            <option value="fr">{t("transcription.languageFrench")}</option>
          </select>
          <ChevronDownIcon />
        </div>
      </section>

      <section className={`field ${!hasLanguage(outputs) ? "field--invalid" : ""}`}>
        <span className="field__label">{t("outputs.versions")}</span>
        <OutputLanguageCards
          languages={outputs.languages}
          englishNotice={
            needsTranslationModel ? (
              <span className="language-card__notice">
                {translationModelSize !== null
                  ? t("outputs.translationModelRequired", {
                      size: formatBytes(translationModelSize, language),
                    })
                  : t("outputs.translationModelRequiredNoSize")}
                <button
                  type="button"
                  className="language-card__download"
                  disabled={translationModelBusy}
                  onClick={(e) => {
                    // The card is a <label>: without this the click would
                    // also toggle the checkbox it sits inside.
                    e.preventDefault();
                    e.stopPropagation();
                    onDownloadTranslationModel();
                  }}
                >
                  {translationModelBusy
                    ? t("outputs.translationModelDownloading")
                    : t("outputs.downloadTranslationModel")}
                </button>
              </span>
            ) : null
          }
          onChange={(languages) => setOutputs((current) => ({ ...current, languages }))}
        />
        {!hasLanguage(outputs) && (
          <p className="drop-zone__error" role="alert">
            {t("outputs.languagesError")}
          </p>
        )}
      </section>

      <section className={`field ${!hasFormat(outputs) ? "field--invalid" : ""}`}>
        <span className="field__label">{t("media.outputs")}</span>
        <div className="checkboxes">
          <label className="checkbox">
            <input
              type="checkbox"
              checked={outputs.formats.srt}
              onChange={(e) =>
                setOutputs((current) => ({
                  ...current,
                  formats: { ...current.formats, srt: e.target.checked },
                }))
              }
            />
            <span className="checkbox__box" aria-hidden="true" />
            {t("media.srt")}
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={outputs.formats.txt}
              onChange={(e) =>
                setOutputs((current) => ({
                  ...current,
                  formats: { ...current.formats, txt: e.target.checked },
                }))
              }
            />
            <span className="checkbox__box" aria-hidden="true" />
            {t("media.txt")}
          </label>
        </div>
        {!hasFormat(outputs) && (
          <p className="drop-zone__error" role="alert">
            {t("media.outputsError")}
          </p>
        )}
      </section>

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onChangeFile}>
          {t("media.changeFile")}
        </button>
        <button
          type="button"
          className="button button--primary"
          disabled={!canLaunch}
          title={!modelReady ? t("media.generateDisabledTitle") : undefined}
          onClick={() => onGenerate(outputs)}
        >
          {/* The count is real (languages × formats), computed by the same
              rule Rust uses — never a guess. */}
          {count > 0 ? t("media.generateCount", { count }) : t("media.generate")}
        </button>
      </div>
    </div>
  );
}
