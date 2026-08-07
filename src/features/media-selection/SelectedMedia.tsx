import { useId, useState } from "react";
import { formatBytes } from "./formatBytes";
import {
  BoltIcon,
  ChevronDownIcon,
  FileIcon,
  GlobeIcon,
  SizeIcon,
  TargetIcon,
  TypeIcon,
} from "./icons";
import type { MediaInfo, OutputSelection, TranscriptionMode } from "./types";

interface SelectedMediaProps {
  media: MediaInfo;
  onChangeFile: () => void;
}

export function SelectedMedia({ media, onChangeFile }: SelectedMediaProps) {
  const languageId = useId();
  const [mode, setMode] = useState<TranscriptionMode>("fast");
  const [outputs, setOutputs] = useState<OutputSelection>({
    srt: true,
    txt: true,
  });

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
              {formatBytes(media.sizeBytes)}
            </span>
          </div>
          <p className="file-card__path" title={media.path}>
            {media.path}
          </p>
        </div>
      </div>

      <section className="field">
        <label className="field__label" htmlFor={languageId}>
          Langue
        </label>
        <div className="select">
          <GlobeIcon />
          <select id={languageId} value="fr" onChange={() => {}}>
            <option value="fr">Français</option>
          </select>
          <ChevronDownIcon />
        </div>
      </section>

      <section className="field">
        <span className="field__label" id="mode-label">
          Mode
        </span>
        <div
          className="segmented"
          role="radiogroup"
          aria-labelledby="mode-label"
        >
          <button
            type="button"
            role="radio"
            aria-checked={mode === "fast"}
            className={`segmented__option ${mode === "fast" ? "segmented__option--active" : ""}`}
            onClick={() => setMode("fast")}
          >
            <BoltIcon />
            Rapide
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={mode === "precise"}
            className={`segmented__option ${mode === "precise" ? "segmented__option--active" : ""}`}
            onClick={() => setMode("precise")}
          >
            <TargetIcon />
            Précis
          </button>
        </div>
      </section>

      <section className="field">
        <span className="field__label">Sorties</span>
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
            SRT
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
            TXT
          </label>
        </div>
      </section>

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onChangeFile}>
          Changer de fichier
        </button>
        <button
          type="button"
          className="button button--primary"
          aria-disabled="true"
          title="Disponible à partir de la Mission 2"
          onClick={(event) => event.preventDefault()}
        >
          Générer les sous-titres
        </button>
      </div>
    </div>
  );
}
