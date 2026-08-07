import { formatBytes } from "./formatBytes";
import type { MediaInfo } from "./types";

interface SelectedMediaProps {
  media: MediaInfo;
  onChangeFile: () => void;
  onRemove: () => void;
}

export function SelectedMedia({
  media,
  onChangeFile,
  onRemove,
}: SelectedMediaProps) {
  return (
    <div className="selected-media">
      <div className="selected-media__icon" aria-hidden="true">
        {media.kind === "video" ? "🎬" : "🎧"}
      </div>
      <div className="selected-media__details">
        <p className="selected-media__name">{media.fileName}</p>
        <p className="selected-media__meta">
          {media.extension.toUpperCase()} · {formatBytes(media.sizeBytes)}
        </p>
        <p className="selected-media__path">{media.path}</p>
      </div>
      <div className="selected-media__actions">
        <button type="button" className="button" onClick={onChangeFile}>
          Changer de fichier
        </button>
        <button type="button" className="button button--ghost" onClick={onRemove}>
          Retirer
        </button>
      </div>
      <button
        type="button"
        className="button button--primary button--disabled"
        disabled
        title="Disponible à partir de la Mission 2"
      >
        Générer les sous-titres
      </button>
    </div>
  );
}
