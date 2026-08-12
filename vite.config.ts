import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// This config is ESM ("type": "module"), so __dirname is not defined.
const __dirname = dirname(fileURLToPath(import.meta.url));

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Two HTML entry points: the app itself and the splash window (see
  // ADR-009). The splash is a separate document rather than a route so it
  // can paint without loading React, i18n or the settings round trip — and
  // so its window can be granted no capability at all.
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        splash: resolve(__dirname, "splash.html"),
      },
    },
  },

  // Pure-logic unit tests only (locale resolution, i18n catalogue shape,
  // settings defaults) — no DOM rendering, so no jsdom/happy-dom dependency
  // is needed. Scoped to src/ so Vitest's default glob doesn't also pick up
  // the vendored, gitignored engine/whisper.cpp clone's own Node test suite.
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
