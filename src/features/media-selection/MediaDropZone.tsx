import { useTranslation } from "react-i18next";
import { SUPPORTED_EXTENSIONS_LABEL } from "./constants";
import { CloudUploadIcon, LockIcon } from "./icons";
import type { MediaSelectionErrorCode } from "./types";

interface MediaDropZoneProps {
  isDragging: boolean;
  errorCode: MediaSelectionErrorCode | null;
  onSelectClick: () => void;
}

export function MediaDropZone({ isDragging, errorCode, onSelectClick }: MediaDropZoneProps) {
  const { t } = useTranslation();

  return (
    <div className="home">
      <div className={`drop-zone ${isDragging ? "drop-zone--active" : ""}`}>
        <CloudUploadIcon />
        <p className="drop-zone__title">{t("drop.title")}</p>
        <button type="button" className="drop-zone__link" onClick={onSelectClick}>
          {t("drop.selectFile")}
        </button>
        <p className="drop-zone__hint">{SUPPORTED_EXTENSIONS_LABEL}</p>
        {errorCode && (
          <p className="drop-zone__error" role="alert">
            {t(`error.mediaCodes.${errorCode}.message`)}
          </p>
        )}
      </div>
      <p className="home__footer">
        <LockIcon />
        {t("common.localProcessing")}
      </p>
    </div>
  );
}
