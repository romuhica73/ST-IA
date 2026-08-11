import { WarningIcon } from "../media-selection/icons";

interface ModelCorruptedProps {
  onReinstall: () => void;
}

export function ModelCorrupted({ onReinstall }: ModelCorruptedProps) {
  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon result-header__icon--error" aria-hidden="true">
          <WarningIcon />
        </div>
        <p className="result-header__title">Modèle endommagé</p>
        <p className="result-header__subtitle">
          Le modèle de transcription est endommagé et doit être réinstallé.
        </p>
      </div>

      <div className="actions actions--single">
        <button type="button" className="button button--primary" onClick={onReinstall}>
          Réinstaller le modèle
        </button>
      </div>
    </div>
  );
}
