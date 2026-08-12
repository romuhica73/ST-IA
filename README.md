# ST-IA

## Objectif

Application macOS locale transformant un média audio/vidéo en :

* SRT ;
* TXT.

La transcription est **en français** — la seule langue qualifiée à ce stade. ST-IA ne produit pas de traduction (voir [ADR-008](docs/architecture/ADR-008-bilingual-output-pipeline.md)).

Le modèle de transcription (~547 Mo) est téléchargé une seule fois, sur action explicite de l'utilisateur, puis stocké localement (`Application Support`). Après cette installation, la transcription fonctionne entièrement hors ligne — vos médias ne quittent jamais votre Mac.

## Principes

* local-first ;
* privacy-first ;
* Apple Silicon first ;
* pas de cloud pour la transcription ;
* pas de dépendance Python utilisateur.

## Interface

* Interface disponible en français et en anglais (Système / Français / English) ;
* thème Système / Clair / Sombre, suit macOS en direct si « Système » est sélectionné ;
* réduction des animations Système / Activé / Désactivé (accessibilité) ;
* préférences enregistrées localement (`Application Support`), jamais envoyées nulle part ;
* langue de l'interface et langue de transcription sont deux réglages indépendants — la transcription reste qualifiée en français quelle que soit la langue de l'interface.

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

## Configuration requise

* macOS sur Apple Silicon (arm64) — M1 ou plus récent ;
* environ 600 Mo d'espace disque pour le modèle, plus l'espace de travail temporaire d'une transcription.

Intel n'est pas pris en charge. Il n'existe pas de build Windows.

## Statut

Release candidate locale `0.1.0` : pipeline local complet, gestionnaire de modèle, annulation et récupération, endurance qualifiée jusqu'à 60 minutes, identité visuelle et motion (Mission 6), réglages/i18n FR-EN/À propos (Mission 7), revue de sécurité et licence MIT (Mission 8), écran de démarrage et packaging de release (Mission 9).

L'installation se fera à terme via une image disque `ST-IA-<version>-macos-arm64.dmg` publiée sur GitHub. **Aucune release n'est encore disponible** : les builds actuelles ne sont ni signées ni notariées, et macOS affichera un avertissement Gatekeeper à la première ouverture. En attendant, [construisez depuis les sources](docs/BUILDING.md). Voir la [checklist de release](docs/release/RELEASE_CHECKLIST.md).

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
pnpm test            # tests frontend (Vitest — i18n, locale, réglages, splash)
cargo check          # depuis src-tauri/
cargo test           # depuis src-tauri/
cargo fmt --check    # depuis src-tauri/
```

Pour assembler les artefacts de release macOS (DMG, archive, `SHA256SUMS.txt`) après un
`pnpm tauri build` :

```bash
scripts/package-release.sh          # collecte et audite une build existante
scripts/package-release.sh --build  # construit d'abord
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
* [ADR-005 — Cycle de vie des jobs, annulation et nettoyage](docs/architecture/ADR-005-runtime-lifecycle-and-cancellation.md)
* [ADR-006 — Identité de production, portabilité et migration](docs/architecture/ADR-006-release-identity-and-data-migration.md)
* [ADR-007 — Préférences locales et localisation](docs/architecture/ADR-007-local-preferences-and-interface-localization.md)
* [ADR-008 — Pipeline de sortie bilingue (rejeté)](docs/architecture/ADR-008-bilingual-output-pipeline.md)
* [ADR-009 — Splashscreen et packaging de release](docs/architecture/ADR-009-splashscreen-and-release-packaging.md)
* [Démarrage rapide](docs/QUICKSTART.md)
* [Construire depuis les sources](docs/BUILDING.md)
* [Contribuer](CONTRIBUTING.md)
* [Politique de sécurité](SECURITY.md)
* [Modèle de menace](docs/security/THREAT_MODEL.md)
* [Revue de sécurité M8](docs/security/M8_SECURITY_REVIEW.md)
* [Delta de sécurité M9](docs/security/M9_SECURITY_DELTA.md)
* [Checklist de release](docs/release/RELEASE_CHECKLIST.md)
* [Licence du projet (MIT)](LICENSE)
* [Composants tiers et licences](THIRD_PARTY_NOTICES.md)
* [FFmpeg — sidecar et licence](docs/third-party/FFMPEG.md)
* [Changelog](CHANGELOG.md)

## Licence

ST-IA est distribué sous licence [MIT](LICENSE).

Cette licence couvre **le code de ST-IA uniquement**. Les composants tiers distribués
avec l'application conservent chacun leur propre licence — voir
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) :

* **FFmpeg** — LGPL-2.1, distribué comme exécutable séparé
  (voir [`docs/third-party/FFMPEG.md`](docs/third-party/FFMPEG.md)) ;
* **whisper.cpp** — MIT.

Le modèle Whisper n'est pas redistribué : il est téléchargé depuis Hugging Face à la
demande explicite de l'utilisateur.
