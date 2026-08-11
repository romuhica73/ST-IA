import { SUPPORTED_EXTENSIONS_LABEL } from "./constants";
import { CloudUploadIcon, LockIcon } from "./icons";

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
    <div className="home">
      <div className={`drop-zone ${isDragging ? "drop-zone--active" : ""}`}>
        <CloudUploadIcon />
        <p className="drop-zone__title">Déposez votre média ici</p>
        <button type="button" className="drop-zone__link" onClick={onSelectClick}>
          ou sélectionner un fichier
        </button>
        <p className="drop-zone__hint">{SUPPORTED_EXTENSIONS_LABEL}</p>
        {errorMessage && (
          <p className="drop-zone__error" role="alert">
            {errorMessage}
          </p>
        )}
      </div>
      <p className="home__footer">
        <LockIcon />
        Traitement 100&nbsp;% local
      </p>
    </div>
  );
}
