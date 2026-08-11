import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { formatBytes } from "./formatBytes";
import { ChevronDownIcon, FileIcon, GlobeIcon, SizeIcon, TypeIcon } from "./icons";
import { asSupportedLanguage } from "../../i18n/locale";
import type { MediaInfo, OutputSelection } from "./types";

interface SelectedMediaProps {
  media: MediaInfo;
  modelReady: boolean;
  onChangeFile: () => void;
  onGenerate: (outputs: OutputSelection) => void;
}

export function SelectedMedia({
  media,
  modelReady,
  onChangeFile,
  onGenerate,
}: SelectedMediaProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);
  const languageId = useId();
  const [outputs, setOutputs] = useState<OutputSelection>({
    srt: true,
    txt: true,
  });
  const hasOutputSelected = outputs.srt || outputs.txt;

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

      <section className="field">
        <span className="field__label">{t("media.outputs")}</span>
        <div className="checkboxes">
          <label className="checkbox">
            <input
              type="checkbox"
              checked={outputs.srt}
              onChange={(e) =>
                setOutputs((current) => ({ ...current, srt: e.target.checked }))
              }
            />
            <span className="checkbox__box" aria-hidden="true" />
            {t("media.srt")}
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={outputs.txt}
              onChange={(e) =>
                setOutputs((current) => ({ ...current, txt: e.target.checked }))
              }
            />
            <span className="checkbox__box" aria-hidden="true" />
            {t("media.txt")}
          </label>
        </div>
        {!hasOutputSelected && (
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
          disabled={!hasOutputSelected || !modelReady}
          title={!modelReady ? t("media.generateDisabledTitle") : undefined}
          onClick={() => onGenerate(outputs)}
        >
          {t("media.generate")}
        </button>
      </div>
    </div>
  );
}
