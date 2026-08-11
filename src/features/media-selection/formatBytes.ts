import type { SupportedLanguage } from "../../i18n/locale";

const UNITS: Record<SupportedLanguage, readonly string[]> = {
  fr: ["o", "Ko", "Mo", "Go"],
  en: ["B", "KB", "MB", "GB"],
};

export function formatBytes(bytes: number, language: SupportedLanguage): string {
  const units = UNITS[language];
  if (bytes <= 0) return `0 ${units[0]}`;

  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / Math.pow(1024, exponent);
  const formatted = exponent === 0 ? value.toString() : value.toFixed(1);

  return `${formatted} ${units[exponent]}`;
}
