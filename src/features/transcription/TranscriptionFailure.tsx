import { useTranslation } from "react-i18next";
import { WarningIcon } from "../media-selection/icons";
import type { TranscriptionErrorCode } from "./types";

interface TranscriptionFailureProps {
  code: TranscriptionErrorCode;
  fileName: string;
  onChooseAnother: () => void;
  onRetry: () => void;
  onInstallModel: () => void;
}

export function TranscriptionFailure({
  code,
  fileName,
  onChooseAnother,
  onRetry,
  onInstallModel,
}: TranscriptionFailureProps) {
  const { t } = useTranslation();
  const isModelMissing = code === "modelMissing";
  const title = t(`error.codes.${code}.title`);

  return (
    <div className="job">
      <div className="result-header">
        <div className="result-header__icon result-header__icon--error" aria-hidden="true">
          <WarningIcon />
        </div>
        <p className="result-header__title">{title}</p>
        <p className="result-header__subtitle">{t(`error.codes.${code}.message`)}</p>
      </div>

      <div className="actions">
        <button type="button" className="button button--secondary" onClick={onChooseAnother}>
          {t("error.chooseAnother")}
        </button>
        {isModelMissing ? (
          <button type="button" className="button button--primary" onClick={onInstallModel}>
            {t("error.installModel")}
          </button>
        ) : (
          <button type="button" className="button button--primary" onClick={onRetry}>
            {t("error.retry")}
          </button>
        )}
      </div>

      <details className="tech-details">
        <summary>{t("error.technicalDetails")}</summary>
        <dl className="tech-details__grid">
          <dt>{t("error.fieldError")}</dt>
          <dd>{title}</dd>
          <dt>{t("error.fieldCode")}</dt>
          <dd>{code}</dd>
          <dt>{t("error.fieldFile")}</dt>
          <dd>{fileName}</dd>
        </dl>
      </details>
    </div>
  );
}
