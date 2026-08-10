# Roadmap

## M0 — Technical spike

Statut : `DONE`

* whisper.cpp ;
* Apple Silicon arm64 ;
* Metal ;
* FFmpeg ;
* SRT ;
* TXT.

## M0B — Qualification française

Statut : `DONE`

Objectifs :

* vrai média français ;
* comparaison modèles ;
* segmentation SRT ;
* performances ;
* choix modèle MVP.

Résultat : modèle MVP retenu `large-v3-turbo-q5_0` (voir [ADR-001](architecture/ADR-001-transcription-engine.md#11-qualification-française-mission-0b)). Qualification basée sur un échantillon unique (~3 min) — à élargir avant M5.

## M1 — Desktop shell

Statut : `DONE`

Objectifs :

* Tauri 2 ;
* React ;
* TypeScript ;
* fenêtre unique ;
* drag & drop ;
* sélection média ;
* validation.

Résultat : shell Tauri 2 fonctionnel avec sélection native et validation Rust (`inspect_media`). Interface alignée sur les mockups validés (Mission 1B) et vérifiée manuellement par l'utilisateur.

## M2 — Pipeline intégré

Statut : `DONE`

Objectifs :

* Rust orchestration ;
* FFmpeg sidecar ;
* whisper.cpp sidecar ;
* traitement end-to-end ;
* SRT/TXT.

Résultat : pipeline local prouvé de bout en bout depuis l'application (voir [ADR-003](architecture/ADR-003-local-transcription-pipeline.md)). Sidecars FFmpeg 9.0 et whisper-cli statiques (aucune dépendance Homebrew/dev clone, vérifié `otool -L`), modèle `large-v3-turbo-q5_0` résolu via Application Support, SRT/TXT réels générés pour `IMG_8484.MOV`. Mode « Précis » et téléchargement de modèle restent hors périmètre (M3).

## M3 — Model manager

Statut : `DONE`

Objectifs :

* modèle absent ;
* téléchargement explicite ;
* progression ;
* SHA-256 ;
* stockage Application Support ;
* fonctionnement offline.

Résultat : gestionnaire de modèle prouvé de bout en bout, mode développement et `.app` empaqueté (voir [ADR-004](architecture/ADR-004-model-management.md)). Téléchargement réel depuis Hugging Face, SHA-256 vérifié exactement (`394221709c...`), promotion atomique, modèle corrompu détecté et rejeté, transcription réelle après installation, fonctionnement hors ligne prouvé (aucune requête réseau pendant le pipeline).

## M4 — Robustesse

Statut : `IN PROGRESS`

Objectifs :

* annulation ;
* fermeture application ;
* processus enfants ;
* temporaires ;
* fichier invalide ;
* absence audio ;
* espace disque ;
* modèle corrompu.

Avancement : cycle de vie des jobs implémenté et couvert par tests unitaires (voir [ADR-005](architecture/ADR-005-runtime-lifecycle-and-cancellation.md)) — emplacement de job unique avec revendication atomique (double lancement refusé côté Rust), handles FFmpeg/whisper-cli détenus et tuables, annulation réelle, nettoyage des temporaires à la fermeture et récupération au démarrage, garde d'espace disque, classification des échecs FFmpeg vérifiée contre le sidecar réel. Qualification manuelle depuis l'application (annulation pendant Whisper, retry sans redémarrage, fermeture pendant un job, média invalide) à confirmer avant passage à `DONE` et promotion de l'ADR-005 en `ACCEPTED`.

## M5 — MVP macOS

Objectifs :

* UX finale ;
* build Release ;
* `.app` ;
* tests médias longs ;
* validation DaVinci Resolve.

## Post-MVP

Liste indicative, non engageante :

* diarisation ;
* chapitres YouTube ;
* résumés ;
* export formats additionnels ;
* support Windows.
