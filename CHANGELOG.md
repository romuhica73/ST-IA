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
