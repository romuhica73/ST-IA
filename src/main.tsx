import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./i18n";
import "./styles/global.css";

// Resolves theme/motion against the OS synchronously, before React's first
// paint — matchMedia is available immediately, no async wait needed for the
// "System" default (which first launch also happens to be). If the user has
// an explicit saved preference instead, useApplySettings corrects the
// attribute once the async settings load completes; only that non-default
// case can have a (sub-perceptible, local-IPC-speed) flash. See ADR-007.
document.documentElement.dataset.theme = window.matchMedia("(prefers-color-scheme: dark)").matches
  ? "dark"
  : "light";
document.documentElement.dataset.motion = window.matchMedia("(prefers-reduced-motion: reduce)")
  .matches
  ? "reduce"
  : "full";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
