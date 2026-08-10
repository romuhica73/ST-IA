# Changelog

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et [Semantic Versioning](https://semver.org/lang/fr/). Projet pré-MVP, aucune release publiée.

## [Unreleased]

### Added

* Initialisation du repository ST-IA.
* Technical spike whisper.cpp (v1.9.2, arm64, backend Metal).
* Preuve d'accélération Metal active sur Apple Silicon.
* Pipeline de bout en bout FFmpeg → whisper.cpp → SRT/TXT.
* ADR-001 — décision du moteur de transcription local.
* Gouvernance du projet : README, roadmap, index ADR, changelog.
* Qualification française (Mission 0B) : comparaison `small` / `medium` / `large-v3-turbo-q5_0` sur un échantillon français réel ; modèle MVP retenu `large-v3-turbo-q5_0` ; ADR-001 promue à `ACCEPTED`.
* Shell desktop Tauri 2 + React + TypeScript (Mission 1) : fenêtre unique, sélection native de fichier et drag & drop, validation de média côté Rust (`inspect_media`), gestion d'erreurs (format non supporté, fichier introuvable, fichier vide, fichier multiple). ADR-002 — architecture desktop du MVP.
* Alignement visuel sur les mockups validés (Mission 1B) : écran d'accueil (zone de dépôt, icône, lien de sélection, mention confidentialité) et écran fichier sélectionné (carte fichier, sections Langue / Mode / Sorties, actions) conformes aux deux maquettes de référence. Sections Langue/Mode/Sorties : état local frontend uniquement, non branché à un vrai pipeline.
* Pipeline de transcription local intégré (Mission 2) : sidecars FFmpeg 9.0 et whisper.cpp statiques reconstruits pour ST-IA (arm64, sans dépendance Homebrew ni au clone de développement — `scripts/build-ffmpeg-sidecar.sh`, `scripts/build-whisper-sidecar.sh`), orchestration Rust (`start_transcription`, `get_transcription_status`, `open_output_folder`) sans shell arbitraire côté frontend, résolution du modèle via Application Support, écrans de progression/succès/échec (Mission 2) conformes aux mockups 4/5/6. Génération réelle de SRT/TXT prouvée de bout en bout depuis l'application. ADR-003 — pipeline local et packaging des moteurs, `ACCEPTED`.
* Gestionnaire de modèle local (Mission 3) : téléchargement explicite du modèle `large-v3-turbo-q5_0` depuis Hugging Face (`get_model_status`, `get_model_manifest`, `install_model`), fichier temporaire pendant le téléchargement, vérification SHA-256, promotion atomique, écrans « Modèle requis » / téléchargement / vérification / modèle endommagé conformes au mockup 3. Fonctionnement hors ligne prouvé après installation. `scripts/provision-dev-model.sh` conservé comme outil développeur uniquement. ADR-004 — gestion et intégrité du modèle local, `ACCEPTED`.

* Robustesse runtime et annulation (Mission 4, en cours) : emplacement de job unique avec revendication atomique côté Rust (double lancement réellement impossible, plus seulement un bouton désactivé), propriété explicite des processus enfants FFmpeg et whisper-cli (handles détenus dans l'état du job, jamais exposés au frontend), bouton « Annuler » fonctionnel tuant le processus actif avec états `cancelling`/`cancelled` émis après terminaison effective, nettoyage des workspaces temporaires à la fermeture de l'application (`RunEvent::ExitRequested`/`Exit`) et récupération au démarrage bornée aux répertoires ST-IA avec garde-fous stricts sur les noms, suppression des téléchargements `.download` interrompus au lancement suivant, garde d'espace disque avant lancement de FFmpeg (`statvfs`, `taille source + 256 Mio`), refus d'un téléchargement de modèle pendant une transcription, non-publication d'un dossier de sortie partiel en cas d'échec de copie, classification des échecs FFmpeg en erreurs métier sans fuite de stderr. ADR-005 — cycle de vie des jobs, annulation et nettoyage, `PROVISIONAL`.

### Fixed

* Progression du téléchargement du modèle affichant « NaN undefined / NaN undefined » : les enums `ModelStatus` et `JobStatus` (internally-tagged, `#[serde(tag = "status", rename_all = "camelCase")]`) ne propageaient pas `rename_all` aux champs de leurs variantes struct-like — `downloaded_bytes`/`total_bytes` (Downloading) et `output_dir`/`transcript_text` (Completed) restaient sérialisés en snake_case malgré l'attribut au niveau de l'enum, donc lus comme `undefined` côté frontend. Corrigé en ajoutant `#[serde(rename_all = "camelCase")]` directement sur chaque variante struct-like concernée. Tests de régression ajoutés vérifiant la forme JSON exacte des deux enums.
* Bouton « Ouvrir le dossier » sans effet visible : conséquence directe du même bug (`output_dir` reçu `undefined`, erreur silencieusement avalée). La commande `open_output_folder` utilise désormais `reveal_item_in_dir` (mécanisme officiel du plugin `opener`) ciblant un fichier généré (`.srt`/`.txt`) plutôt qu'`open_path` sur le dossier, pour que Finder s'ouvre directement dessus. Permission ajustée en conséquence (`opener:allow-reveal-item-in-dir` remplace `opener:allow-open-path`, plus restrictive). L'échec silencieux est remplacé par un message utilisateur léger (« Impossible d'ouvrir le dossier. ») sans jamais faire planter l'application.
