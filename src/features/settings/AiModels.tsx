import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { formatBytes } from "../media-selection/formatBytes";
import { asSupportedLanguage } from "../../i18n/locale";
import type { ModelCard, ModelKind } from "../model-manager/types";
import type { ModelStatus } from "../model-manager/types";

interface AiModelsProps {
  /** Statuses already known to the app, so opening Settings does not trigger
   * a fresh multi-gigabyte hash just to render a badge. A model whose status
   * has not been resolved is shown as unknown rather than as missing. */
  statuses: Partial<Record<ModelKind, ModelStatus | null>>;
}

/** Factual transparency about the models ST-IA runs.
 *
 * Everything shown comes from `get_model_cards`, which reads the same pinned
 * constants the app verifies against — no size, hash or URL is restated in
 * TypeScript, so the panel cannot advertise something the app does not
 * actually enforce.
 *
 * Deliberately descriptive, not a compliance claim: it says what the models
 * are, where they come from and where they run. It does not assert
 * certification of any kind.
 */
export function AiModels({ statuses }: AiModelsProps) {
  const { t, i18n } = useTranslation();
  const language = asSupportedLanguage(i18n.language);
  const [cards, setCards] = useState<ModelCard[] | null>(null);

  useEffect(() => {
    void invoke<ModelCard[]>("get_model_cards")
      .then(setCards)
      .catch((error) => console.error("Failed to load model cards:", error));
  }, []);

  if (cards === null) return null;

  function statusLabel(kind: ModelKind): string {
    const status = statuses[kind];
    if (status === undefined || status === null) return t("aiModels.statusUnknown");
    switch (status.status) {
      case "ready":
        return t("aiModels.statusInstalled");
      case "downloading":
      case "verifying":
        return t("aiModels.statusInstalling");
      case "corrupted":
        return t("aiModels.statusCorrupted");
      default:
        return t("aiModels.statusNotInstalled");
    }
  }

  return (
    <section className="settings-section">
      <h3 className="settings-section__title">{t("aiModels.title")}</h3>
      <p className="settings-section__note">{t("aiModels.purpose")}</p>

      {cards.map((card) => (
        <div className="model-card" key={card.kind}>
          <div className="model-card__head">
            <p className="model-card__name">Whisper {card.id}</p>
            <p className="model-card__role">{t(`aiModels.role.${card.kind}`)}</p>
            <p className="model-card__runtime">{card.runtime}</p>
          </div>
          <p className="model-card__facts">
            {statusLabel(card.kind)} · {formatBytes(card.sizeBytes, language)}
          </p>
          <p className="model-card__local">{t("aiModels.runsLocally")}</p>
          <p className="model-card__network">{t("aiModels.networkOnlyForDownload")}</p>

          {/* The full provenance lives behind a disclosure: it is the kind of
              detail that must be available and verifiable, but that would
              bury the three facts most people actually want. */}
          <details className="model-card__details">
            <summary>{t("aiModels.technicalDetails")}</summary>
            <dl className="tech-details__grid">
              <dt>{t("aiModels.fieldId")}</dt>
              <dd>{card.id}</dd>
              <dt>{t("aiModels.fieldFile")}</dt>
              <dd>{card.fileName}</dd>
              <dt>{t("aiModels.fieldRuntime")}</dt>
              <dd>{card.runtime}</dd>
              <dt>{t("aiModels.fieldSize")}</dt>
              <dd>{card.sizeBytes.toLocaleString(language)} B</dd>
              <dt>{t("aiModels.fieldSha")}</dt>
              <dd className="model-card__hash">{card.sha256}</dd>
              <dt>{t("aiModels.fieldProvider")}</dt>
              <dd>{card.provider}</dd>
              <dt>{t("aiModels.fieldSource")}</dt>
              <dd className="model-card__hash">{card.sourceUrl}</dd>
              <dt>{t("aiModels.fieldDistribution")}</dt>
              <dd>
                {card.bundled ? t("aiModels.bundled") : t("aiModels.notBundled")}
                {card.downloadedOnDemand ? ` · ${t("aiModels.onDemand")}` : ""}
              </dd>
              <dt>{t("aiModels.fieldInference")}</dt>
              <dd>
                {card.localInference ? t("aiModels.localInference") : "—"}
                {!card.networkDuringInference ? ` · ${t("aiModels.noNetworkDuringInference")}` : ""}
              </dd>
            </dl>
          </details>
        </div>
      ))}

      <p className="settings-section__note settings-section__note--limits">
        <strong>{t("aiModels.limitationsTitle")}</strong> {t("aiModels.limitationsBody")}
      </p>
    </section>
  );
}
