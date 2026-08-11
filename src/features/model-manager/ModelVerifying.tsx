import { useTranslation } from "react-i18next";
import { DownloadCircleIcon } from "./icons";

export function ModelVerifying() {
  const { t } = useTranslation();

  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">{t("model.verifying")}</p>
      <section className="progress-field">
        <div className="progress-bar progress-bar--indeterminate" role="progressbar">
          <div className="progress-bar__fill" />
        </div>
      </section>
    </div>
  );
}
