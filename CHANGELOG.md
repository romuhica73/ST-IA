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
