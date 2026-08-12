import { useTranslation } from "react-i18next";
import type { OutputLanguages } from "./outputs";

interface OutputLanguageCardsProps {
  languages: OutputLanguages;
  /** Rendered under the English card when its model is not installed yet —
   * the selection is still allowed, the download is just offered alongside. */
  englishNotice?: React.ReactNode;
  onChange: (languages: OutputLanguages) => void;
}

/** Which versions of the transcript to produce.
 *
 * Real `<input type="checkbox">` elements, visually restyled as cards — not
 * `<div onClick>`. That is what gives keyboard focus, space to toggle, the
 * checkbox role and checked state to VoiceOver, and the native label
 * association, all for free. The card is the `<label>`, so the whole surface
 * is the hit target without any of that being reimplemented.
 */
export function OutputLanguageCards({
  languages,
  englishNotice,
  onChange,
}: OutputLanguageCardsProps) {
  const { t } = useTranslation();

  return (
    <div className="language-cards">
      <label className="language-card">
        <input
          type="checkbox"
          className="language-card__input"
          checked={languages.french}
          onChange={(e) => onChange({ ...languages, french: e.target.checked })}
        />
        <span className="language-card__check" aria-hidden="true" />
        <span className="language-card__text">
          <span className="language-card__name">{t("outputs.french")}</span>
          <span className="language-card__role">{t("outputs.original")}</span>
        </span>
      </label>

      <label className="language-card">
        <input
          type="checkbox"
          className="language-card__input"
          checked={languages.english}
          onChange={(e) => onChange({ ...languages, english: e.target.checked })}
        />
        <span className="language-card__check" aria-hidden="true" />
        <span className="language-card__text">
          <span className="language-card__name">{t("outputs.english")}</span>
          <span className="language-card__role">{t("outputs.translation")}</span>
          {englishNotice}
        </span>
      </label>
    </div>
  );
}
