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

## M6 — Visual Polish & Motion

Statut : `DONE`

Objectif : améliorer l'identité visuelle et le ressenti de ST-IA sans modifier son architecture fonctionnelle.

Périmètre :

* hiérarchie visuelle ;
* micro-interactions ;
* transitions ;
* feedback de drag & drop ;
* progression ;
* états succès/erreur ;
* cohérence light/dark ;
* motion accessible ;
* réduction des animations si macOS `prefers-reduced-motion`.

Avancement : identité visuelle dérivée de l'icône ST-IA (teal-cyan `#0c7180`/`#2dd4bf` échantillonné sur le dégradé réel de l'icône, calmé pour un usage UI — pas de neon, pas de gradient dans le chrome), remplaçant le bleu générique précédent. Fond graphite/anthracite en dark mode (`#1a1c20`) plutôt qu'un simple inversé. Tokens formalisés (rayons, ombre unique très subtile, échelle de motion `100–320ms`) réutilisés dans tous les écrans. Icônes glyphe (`✓`, `!`, `♪`, `✨`) remplacées par des SVG cohérents avec la famille existante. Zone de dépôt avec retour drag-over renforcé (échelle + halo, sobre). Boutons dotés d'états hover/active/focus-visible complets. Écran de progression : marqueurs d'étape animés à l'achèvement, barre indéterminée plus calme. Écran de succès : check en scale-in, liste des fichiers en apparition échelonnée. Écrans d'erreur passés d'un remplissage rouge plein à une icône teintée (rouge réservé à l'icône, jamais un bloc plein) — nettement moins anxiogène. `prefers-reduced-motion` respecté : coupe globale des durées/animations, avec un cas particulier traité explicitement (la barre de progression indéterminée passe à un état statique plutôt que d'être figée à mi-course par la règle générale). Toutes les corrections de contraste AA de M5 revérifiées et conservées, plus deux nouveaux tokens texte-sûrs (`--success-text` clair/sombre) pour le badge SRT qui ne les avait pas. Correction de microcopie : « Déposez votre vidéo ici » → « Déposez votre média ici » (incohérent avec le support audio existant).

Réserve : la validation par capture d'écran automatisée (§28) n'a pas pu être produite — trois tentatives via automatisation d'accessibilité macOS ont chacune capturé incidemment une fenêtre ou un panneau étranger à ST-IA exposant des noms de fichiers/dossiers personnels ; chaque capture a été supprimée immédiatement sans être exploitée. Validation visuelle confirmée par l'utilisateur via le gate GUI humain.

Validé humainement, convergé sur `main`.

## M7 — Settings, i18n, About & Versioning

Statut : `IN PROGRESS`

Objectif : ajouter la couche « application configurable » — réglages persistants, interface bilingue FR/EN, écran À propos avec version réelle — sans toucher au moteur de transcription ni au pipeline.

Périmètre :

* réglages (thème, motion, langue) persistés localement (`Application Support/settings.json`), jamais dans `localStorage` ;
* thème Système / Clair / Sombre, suit macOS en direct ;
* réduction des animations Système / Activé / Désactivé, cohérente avec le système de motion M6 ;
* internationalisation complète (i18next/react-i18next), catalogues FR/EN avec parité de clés testée automatiquement ;
* langue de l'interface strictement indépendante de la langue de transcription (le champ média renommé « Langue de transcription » / « Transcription language ») ;
* écran À propos avec version réelle (`app.package_info().version`, jamais codée en dur) ;
* vérification automatisée de cohérence de version entre `package.json`, `Cargo.toml`, `tauri.conf.json` ;
* sélecteur Rapide/Précis retiré (aucun comportement réel derrière, voir ADR-007 décision 7) — reporté en v0.2.

Voir [ADR-007](architecture/ADR-007-local-preferences-and-interface-localization.md) (`PROVISIONAL`) pour le détail des décisions.

## M8 — Open Source & Security Readiness (après M7, non commencée)

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

## M9 — Public Release 0.1.0 (après M8, non commencée)

Signature Developer ID, notarisation, publication GitHub. Non commencée — dépend de la disponibilité d'une identité de signature et d'une décision explicite de rendre le repository public.

## Post-MVP

Liste indicative, non engageante :

* diarisation ;
* chapitres YouTube ;
* résumés ;
* export formats additionnels ;
* support Windows.
