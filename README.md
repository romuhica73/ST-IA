# ST-IA

## Objectif

Application macOS locale transformant un média audio/vidéo en :

* SRT ;
* TXT.

## Principes

* local-first ;
* privacy-first ;
* Apple Silicon first ;
* pas de cloud pour la transcription ;
* pas de dépendance Python utilisateur.

## Architecture cible

```text
Tauri / React
        ↓
Rust
        ↓
FFmpeg sidecar
        ↓
WAV temporaire
        ↓
whisper.cpp sidecar
        ↓
SRT + TXT
```

## Statut

Pré-MVP — shell desktop en cours (Mission 1).

## Stack

* Tauri 2 ;
* React 19 + TypeScript ;
* Vite ;
* Rust ;
* pnpm.

## Développement

```bash
pnpm install       # dépendances frontend
pnpm tauri dev      # lance l'application desktop en mode développement
pnpm build          # build frontend (tsc + vite)
cargo check          # depuis src-tauri/
cargo test           # depuis src-tauri/
cargo fmt --check    # depuis src-tauri/
```

## Documentation

* [Roadmap](docs/ROADMAP.md)
* [Index des ADR](docs/architecture/README.md)
* [ADR-001 — Moteur de transcription](docs/architecture/ADR-001-transcription-engine.md)
* [ADR-002 — Architecture desktop](docs/architecture/ADR-002-desktop-architecture.md)
* [Changelog](CHANGELOG.md)
