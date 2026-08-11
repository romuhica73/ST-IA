import { useTranslation } from "react-i18next";
import { WarningIcon } from "../media-selection/icons";

interface ModelCorruptedProps {
  onReinstall: () => void;
}

export function ModelCorrupted({ onReinstall }: ModelCorruptedProps) {
  const { t } = useTranslation();

  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon result-header__icon--error" aria-hidden="true">
          <WarningIcon />
        </div>
        <p className="result-header__title">{t("model.corruptedTitle")}</p>
        <p className="result-header__subtitle">{t("model.corruptedSubtitle")}</p>
      </div>

      <div className="actions actions--single">
        <button type="button" className="button button--primary" onClick={onReinstall}>
          {t("model.reinstall")}
        </button>
      </div>
    </div>
  );
}
