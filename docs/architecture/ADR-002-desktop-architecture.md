# ADR-002 — Architecture desktop du MVP

## Statut

**ACCEPTED**

## Contexte

ST-IA a besoin d'une application desktop macOS qui soit native, légère, entièrement locale, simple à distribuer sans dépendance runtime lourde (pas de Python, pas de VM Electron/Chromium dédiée par app), et dont l'orchestration système (accès fichiers, futurs sidecars FFmpeg/whisper.cpp) reste sous contrôle strict plutôt que confiée au frontend.

## Décision

Utiliser :

- **Tauri 2** — conteneur applicatif natif (WebView système + noyau Rust).
- **React 19 + TypeScript** — interface utilisateur.
- **Vite** — bundler frontend.
- **Rust** — commandes système, validation, orchestration future.

## Frontière de confiance

- **React/TypeScript** : interface, interaction utilisateur, représentation de l'état local (idle / dragging / sélectionné / erreur). Ne valide jamais un fichier par lui-même — délègue systématiquement à Rust via `invoke`.
- **Rust** : seule couche autorisée à toucher le système de fichiers, valider les médias, et — dans les missions suivantes — orchestrer FFmpeg, whisper.cpp, les fichiers temporaires et le téléchargement de modèles.

Le frontend ne dispose d'aucun shell arbitraire ni d'accès filesystem générique : la capability Tauri de la fenêtre principale n'autorise que `core:default` et `dialog:allow-open` (voir section sécurité du rapport de mission M1).

## Point de vigilance — identifiant de bundle

L'identifiant `com.romainbourbon.st-ia.dev` (`src-tauri/tauri.conf.json`) est **provisoire**, réservé à la phase de développement. Il devra être revu et confirmé avant tout chantier de signature/notarisation Apple (hors périmètre de cette mission).

## Alternatives non retenues

- **Electron** : embarque son propre Chromium/Node, empreinte disque et mémoire nettement supérieure à Tauri pour un utilitaire local simple.
- **Application Swift native** : intégration système la plus légère possible, mais courbe d'apprentissage et vitesse de développement moins favorables pour ce projet ; reste une option pour une réécriture ultérieure si Tauri s'avère limitant.
- **Next.js desktop** (ex. via Electron/Tauri) : apporte du routing serveur et des conventions pensées pour le web, sans valeur ajoutée pour une fenêtre unique locale.
- **Application Python packagée** (ex. PyInstaller) : exclue par contrainte produit — aucune dépendance Python côté utilisateur final.
