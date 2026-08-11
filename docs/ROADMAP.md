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

Statut : `DONE`

Objectifs :

* annulation ;
* fermeture application ;
* processus enfants ;
* temporaires ;
* fichier invalide ;
* absence audio ;
* espace disque ;
* modèle corrompu.

Résultat : cycle de vie des jobs prouvé de bout en bout sur le `.app` empaqueté (voir [ADR-005](architecture/ADR-005-runtime-lifecycle-and-cancellation.md), `ACCEPTED`). Emplacement de job unique avec revendication atomique (double lancement refusé côté Rust), handles FFmpeg/whisper-cli détenus et tuables, annulation réelle d'un Whisper en cours, retry sans redémarrage, fermeture applicative tuant l'enfant et supprimant le workspace (`shutdown: killed child pid …` observé), récupération au démarrage bornée aux répertoires ST-IA, média invalide traité en erreur métier sans jamais lancer Whisper, garde d'espace disque. Réserve : l'annulation pendant l'étape FFmpeg n'a pas pu être observée (étape trop brève sur les médias qualifiés) — même chemin de code que Whisper.

## M5 — MVP macOS

Statut : `DONE`

Objectifs :

* UX finale ;
* build Release ;
* `.app` ;
* tests médias longs ;
* validation DaVinci Resolve.

Avancement : identité de production `com.romainbourbon.stia` adoptée et migration des données depuis l'ancien identifiant `.dev` prouvée sans retéléchargement (voir [ADR-006](architecture/ADR-006-release-identity-and-data-migration.md), `ACCEPTED`). Réserve de portabilité Apple Silicon close : `whisper-cli` reconstruit avec `GGML_NATIVE=OFF`, sans `-mcpu=native` ni extension SME propre à la machine de build, Metal conservé, sorties octet pour octet identiques. Endurance 5/15/30/60 min mesurée, icône de production intégrée (asset ST-IA approuvé, plus de logo Tauri par défaut), `.app` et `.dmg` produits, notices de licence embarquées. Gate DaVinci Resolve validé par l'utilisateur (piste de sous-titres, timecodes, synchronisation OK sur un média réel de 60 min) — limite de vocabulaire observée sur certains noms propres/termes techniques, reportée en future mission qualité (voir ci-dessous). Smoke test GUI final validé par l'utilisateur sur le `.app` empaqueté (icône ST-IA incluse) : parcours complet média → SRT/TXT → Ouvrir le dossier/Finder, et annulation → retour propre → retry sans redémarrage → transcription complète, tous deux confirmés OK.

Résultat : release candidate locale macOS 0.1.0 qualifiée de bout en bout — pipeline, gestionnaire de modèle, cycle de vie/annulation, identité de production, portabilité Apple Silicon, endurance jusqu'à 60 minutes, gate DaVinci Resolve et smoke test GUI. Non signée/notariée (hors périmètre local RC, voir [checklist de release](release/RELEASE_CHECKLIST.md)).

### Future — qualité de transcription (vocabulaire)

Non planifiée, non commencée. À étudier dans une mission ultérieure :

* vocabulaire contextuel / noms propres ;
* terminologie technique ;
* évaluer le biasing par prompt initial de whisper.cpp.

Pas d'implémentation, pas de nouveau modèle, pas de champ UI dans cette entrée.

## M6 — Visual Polish & Motion (future, non commencée)

Objectif indicatif : améliorer l'identité visuelle et le ressenti de ST-IA sans modifier son architecture fonctionnelle.

À couvrir plus tard :

* hiérarchie visuelle ;
* micro-interactions ;
* transitions ;
* feedback de drag & drop ;
* progression ;
* états succès/erreur ;
* cohérence light/dark ;
* motion accessible ;
* réduction des animations si macOS `prefers-reduced-motion`.

Rien codé pour M6 à ce stade.

## Future — Open Source / Sécurité (après M6, non commencée)

Périmètre indicatif à couvrir lors d'une mission dédiée :

* secrets Git/historique ;
* Tauri capabilities ;
* CSP ;
* IPC ;
* dépendances Rust ;
* dépendances JS ;
* supply chain ;
* sidecars ;
* modèle ;
* scripts de build ;
* privacy ;
* configuration sécurité GitHub ;
* `SECURITY.md`.

Audit non commencé.

## Post-MVP

Liste indicative, non engageante :

* diarisation ;
* chapitres YouTube ;
* résumés ;
* export formats additionnels ;
* support Windows.
