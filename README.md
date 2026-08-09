# ST-IA

## Objectif

Application macOS locale transformant un média audio/vidéo en :

* SRT ;
* TXT.

Le modèle de transcription (~547 Mo) est téléchargé une seule fois, sur action explicite de l'utilisateur, puis stocké localement (`Application Support`). Après cette installation, la transcription fonctionne entièrement hors ligne — vos médias ne quittent jamais votre Mac.

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

Pré-MVP — pipeline de transcription local intégré avec gestionnaire de modèle (Mission 3).

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

### Sidecars et modèle (développement)

Les sidecars FFmpeg/whisper.cpp sont commités dans `src-tauri/binaries/` ; pour les reconstruire :

```bash
scripts/build-whisper-sidecar.sh   # nécessite le clone engine/whisper.cpp au SHA pinné (voir ADR-001)
scripts/build-ffmpeg-sidecar.sh    # télécharge et compile FFmpeg depuis la source officielle
```

Le modèle Whisper n'est jamais commité. En usage normal, ST-IA le télécharge lui-même (écran « Modèle requis », voir ADR-004) — aucune manipulation manuelle n'est nécessaire. `scripts/provision-dev-model.sh` reste disponible comme **raccourci strictement développeur** (place un modèle déjà téléchargé à l'emplacement canonique, utile pour itérer sans retélécharger 547 Mo à chaque fois) :

```bash
scripts/provision-dev-model.sh /chemin/vers/ggml-large-v3-turbo-q5_0.bin
```

## Documentation

* [Roadmap](docs/ROADMAP.md)
* [Index des ADR](docs/architecture/README.md)
* [ADR-001 — Moteur de transcription](docs/architecture/ADR-001-transcription-engine.md)
* [ADR-002 — Architecture desktop](docs/architecture/ADR-002-desktop-architecture.md)
* [ADR-003 — Pipeline local et packaging des moteurs](docs/architecture/ADR-003-local-transcription-pipeline.md)
* [ADR-004 — Gestion et intégrité du modèle local](docs/architecture/ADR-004-model-management.md)
* [FFmpeg — sidecar et licence](docs/third-party/FFMPEG.md)
* [Changelog](CHANGELOG.md)
