import { useId } from "react";

interface Option<T extends string> {
  value: T;
  label: string;
}

interface SettingChoiceProps<T extends string> {
  label: string;
  value: T;
  options: Option<T>[];
  onChange: (value: T) => void;
}

/** Accessible three-way choice (theme/motion/language pickers) — same
 * radiogroup-of-buttons pattern already used for output-format selection,
 * reused rather than reinvented. Real <button role="radio">, never a
 * div onClick. */
export function SettingChoice<T extends string>({
  label,
  value,
  options,
  onChange,
}: SettingChoiceProps<T>) {
  const labelId = useId();

  return (
    <section className="field">
      <span className="field__label" id={labelId}>
        {label}
      </span>
      <div className="segmented" role="radiogroup" aria-labelledby={labelId}>
        {options.map((option) => (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={value === option.value}
            className={`segmented__option ${value === option.value ? "segmented__option--active" : ""}`}
            onClick={() => onChange(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>
    </section>
  );
}
