import { SUPPORTED_EXTENSIONS_LABEL } from "./constants";

interface MediaDropZoneProps {
  isDragging: boolean;
  errorMessage: string | null;
  onSelectClick: () => void;
}

export function MediaDropZone({
  isDragging,
  errorMessage,
  onSelectClick,
}: MediaDropZoneProps) {
  return (
    <div className={`drop-zone ${isDragging ? "drop-zone--active" : ""}`}>
      <p className="drop-zone__title">
        Déposez votre vidéo ou votre audio ici
      </p>
      <p className="drop-zone__or">ou</p>
      <button type="button" className="button" onClick={onSelectClick}>
        Sélectionner un fichier
      </button>
      <p className="drop-zone__hint">{SUPPORTED_EXTENSIONS_LABEL}</p>
      {errorMessage && <p className="drop-zone__error">{errorMessage}</p>}
    </div>
  );
}
