export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 o";

  const units = ["o", "Ko", "Mo", "Go"];
  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / Math.pow(1024, exponent);
  const formatted = exponent === 0 ? value.toString() : value.toFixed(1);

  return `${formatted} ${units[exponent]}`;
}
