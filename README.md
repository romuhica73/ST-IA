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

Technical spike / pré-MVP.

## Documentation

* [Roadmap](docs/ROADMAP.md)
* [Index des ADR](docs/architecture/README.md)
* [ADR-001 — Moteur de transcription](docs/architecture/ADR-001-transcription-engine.md)
* [Changelog](CHANGELOG.md)
