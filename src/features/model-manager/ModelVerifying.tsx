import { DownloadCircleIcon } from "./icons";

export function ModelVerifying() {
  return (
    <div className="job model-gate">
      <DownloadCircleIcon />
      <p className="drop-zone__title">Vérification du modèle</p>
      <section className="progress-field">
        <div className="progress-bar progress-bar--indeterminate" role="progressbar">
          <div className="progress-bar__fill" />
        </div>
      </section>
    </div>
  );
}
