import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import fr from "./locales/fr";
import en from "./locales/en";
import { FALLBACK_LANGUAGE, resolveSystemLanguage } from "./locale";

// Bundled resources only — no i18next-http-backend, no network fetch, ever.
// Initial language is resolved synchronously from navigator.language so the
// very first paint is already correct for a first launch (preference =
// "system", which is also the default — see settings/useSettings.ts for how
// an explicit fr/en preference overrides this once it loads).
void i18n
  .use(initReactI18next)
  .init({
    resources: {
      fr: { translation: fr },
      en: { translation: en },
    },
    lng: resolveSystemLanguage(typeof navigator !== "undefined" ? navigator.language : undefined),
    fallbackLng: FALLBACK_LANGUAGE,
    interpolation: { escapeValue: false }, // React already escapes.
    returnNull: false,
  });

export default i18n;
