import { useEffect, useState } from "react";

/**
 * The resolved motion preference, as an observable boolean.
 *
 * Reads the `data-motion` attribute that `useApplySettings` already resolves
 * and writes on the document root — the single source of truth for the M7
 * setting, which can force reduction *or* force full motion against the OS.
 * Reading it here rather than re-resolving means a component can never
 * disagree with the CSS about whether motion is reduced.
 */
export function useReducedMotion(): boolean {
  const read = () => document.documentElement.dataset.motion === "reduce";
  const [reduced, setReduced] = useState(read);

  useEffect(() => {
    // The attribute changes when the user flips the setting, and when the OS
    // preference changes while "System" is selected.
    const observer = new MutationObserver(() => setReduced(read()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-motion"],
    });
    return () => observer.disconnect();
  }, []);

  return reduced;
}
