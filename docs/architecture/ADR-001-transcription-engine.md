# ADR-001 — Moteur de transcription local pour SousTitres

## Statut

**PROVISIONAL**

Accepté comme candidat principal sur la base des preuves techniques recueillies durant la Mission 0 (spike). Non promu à `ACCEPTED` tant que la qualité de transcription française n'a pas été mesurée sur un échantillon représentatif (`NEEDS_FRENCH_SAMPLE`, cf. rapport de mission).

## 1. Contexte

`SousTitres` est une application macOS (Apple Silicon) dont l'unique fonction MVP est :

```
VIDEO/AUDIO → TRANSCRIPTION ENTIÈREMENT LOCALE → SRT + TXT
```

Contraintes non négociables :

- Aucune donnée (audio, vidéo, texte) ne doit transiter par un service distant.
- L'utilisateur final ne doit jamais installer ni utiliser : Terminal, Python, Homebrew, pip, pipx.
- Cible matérielle actuelle : MacBook Pro Apple Silicon M4, 16 Go RAM.
- Priorité qualité absolue : le français.

Cette ADR documente le choix du moteur de transcription et l'architecture d'intégration envisagée, sur la base des résultats du spike technique (Mission 0).

## 2. Options évaluées

| Option | Statut |
|---|---|
| `whisper.cpp` (ggml-org, C/C++, backend Metal) | **Candidat principal — testé** |
| `openai-whisper` (Python, PyTorch) | Rejeté pour le runtime produit |
| WhisperKit / Argmax OSS (Swift, CoreML) | Alternative viable, non testée dans ce spike |
| API cloud (OpenAI, Deepgram, etc.) | Exclu — viole la contrainte de confidentialité locale |

## 3. Décision

`whisper.cpp` est retenu comme **moteur candidat principal** pour la V1, sous réserve de la qualification de la qualité française (voir rapport Mission 0, section 8).

Preuves recueillies pendant le spike :

- Build natif `aarch64-apple-darwin`, release `v1.9.2` (commit `306c88f4d1286aec1bf96e544632897886af5501`, 2026-08-04).
- Backend Metal réellement actif et utilisé (`ggml_metal_device_init: GPU name: MTL0 (Apple M4)`, `whisper_backend_init_gpu: using MTL0 backend`) — pas une supposition documentaire.
- Transcription fonctionnelle de bout en bout (WAV → SRT/TXT) validée pour les modèles `small`, `medium`, `large-v3-turbo-q5_0`.
- Pipeline complet FFmpeg (extraction/conversion) → whisper.cpp (transcription) validé sur un conteneur MP4 synthétique (vidéo H.264 + audio AAC 44.1 kHz stéréo → WAV mono 16 kHz PCM16).
- Aucune dépendance Python nécessaire à l'exécution du binaire `whisper-cli`.

## 4. Pourquoi `openai-whisper` (Python) n'est pas retenu pour le runtime

- Nécessite un interpréteur Python, PyTorch, et généralement un environnement géré par pip/conda — en contradiction directe avec la contrainte « l'utilisateur final ne doit jamais utiliser Python/pip/pipx ».
- Empaqueter un runtime Python + PyTorch dans une app Tauri alourdit considérablement le `.app` (plusieurs centaines de Mo à plusieurs Go) et complique la signature/notarisation.
- `whisper.cpp` réimplémente l'inférence en C/C++ sans dépendance à un interpréteur, ce qui correspond nativement au modèle de distribution d'une app native macOS avec sidecars binaires.
- Une installation Python Whisper déjà présente sur la machine de développement ne doit pas être considérée comme une dépendance du produit — elle n'existe pas sur la machine de l'utilisateur final.

## 5. Pourquoi `whisper.cpp` est candidat principal

- Binaire natif unique (`whisper-cli`), sans runtime interprété, facilement embarquable comme sidecar Tauri.
- Accélération Metal native sur Apple Silicon, confirmée observable (voir rapport Mission 0, section 3) — pas de dépendance à CoreML/ANE pour fonctionner.
- Écosystème de modèles GGML mature, avec variantes quantifiées (`q5_0`, `q8_0`) permettant d'arbitrer taille/vitesse/qualité.
- Formats de sortie natifs incluant SRT et TXT (et JSON/VTT, hors périmètre MVP) sans post-traitement additionnel.
- Projet activement maintenu par l'organisation `ggml-org`, releases taguées, reproductible par commit SHA.

## 6. Pourquoi WhisperKit / Argmax OSS reste une alternative possible

WhisperKit (Swift, backend CoreML/ANE, projet Argmax) n'a pas été testé dans ce spike et reste une alternative sérieuse à réévaluer, notamment parce que :

- Il peut tirer parti du Neural Engine (ANE), pas seulement du GPU Metal, avec un profil énergie/performance potentiellement meilleur.
- Il s'intègre nativement en Swift, ce qui simplifierait une éventuelle version distribuée sans sidecar séparé.

Il n'est pas retenu comme candidat principal à ce stade car :

- Il n'a pas été mesuré empiriquement (aucune preuve recueillie durant ce spike, contrairement à `whisper.cpp`).
- Son intégration dans une architecture Tauri/Rust (sidecar) est moins directe qu'un binaire CLI C/C++ multiplateforme comme `whisper-cli`.

Il devra être évalué dans un spike ultérieur si `whisper.cpp` échoue la qualification française ou si les performances/consommation mémoire s'avèrent insuffisantes en usage réel.

## 7. Architecture sidecar envisagée

```
Tauri / React (frontend, aucune commande shell libre)
        ↓ invoke commande Tauri
Commandes Rust (orchestration exclusive des process)
        ↓
FFmpeg (sidecar binaire embarqué)
        ↓
WAV temporaire (mono, 16 kHz, PCM 16 bits)
        ↓
whisper.cpp / whisper-cli (sidecar binaire embarqué)
        ↓
Fichiers SRT + TXT
```

Aucune objection technique bloquante identifiée durant le spike. Le frontend ne doit à aucun moment invoquer un shell ou un binaire directement — seul le code Rust orchestre les process (spawn, timeout, capture stdout/stderr, nettoyage des fichiers temporaires).

Points de vigilance identifiés (non bloquants, à traiter en implémentation) :
- Le premier appel à Metal après installation peut déclencher une compilation/mise en cache de la bibliothèque shader Metal par le système (~7 secondes observées au premier lancement, quasi instantané ensuite). Ce coût unique doit être anticipé dans l'UX (ex. écran de chargement au premier lancement) plutôt que traité comme une régression de performance.
- `whisper-cli` charge l'intégralité du modèle en mémoire résidente (jusqu'à ~2,2 Go observés pour `medium`) — sur une machine à 16 Go de RAM, l'usage simultané d'un gros modèle et d'autres applications doit être surveillé.

## 8. Gestion FFmpeg envisagée

- **Cible produit : FFmpeg embarqué comme sidecar de l'application**, binaire statique/autonome fourni avec le `.app`, invoqué exclusivement par le code Rust.
- **Explicitement rejeté pour le produit final** : `brew install ffmpeg` côté utilisateur, ou toute dépendance à un FFmpeg système préexistant.
- Pour ce spike uniquement, le FFmpeg déjà installé sur la machine de développement (Homebrew, `/opt/homebrew/bin/ffmpeg`, version 8.1.2) a été utilisé pour prouver la faisabilité technique de la conversion. Cet usage est strictement limité au spike et ne doit pas être interprété comme une dépendance du produit.
- Non résolu dans ce spike, à traiter ultérieurement : choix du binaire FFmpeg à embarquer (build statique arm64 minimal), gestion de la licence (LGPL vs GPL selon les codecs inclus — impact sur la distribution et la conformité App Store le cas échéant), taille finale du binaire embarqué.

## 9. Gestion des modèles envisagée

Architecture évaluée (non implémentée dans ce spike) :

```
Application installée (modèle absent du bundle)
        ↓
Utilisateur demande explicitement le téléchargement d'un modèle
        ↓
Téléchargement local (HTTPS, source officielle whisper.cpp / Hugging Face)
        ↓
Vérification SHA-256
        ↓
Renommage atomique (fichier temporaire → nom final)
        ↓
Stockage dans Application Support
        ↓
Fonctionnement offline ensuite
```

Cette approche est jugée pertinente : elle évite d'alourdir le `.app` de plusieurs centaines de Mo à ~1,5 Go selon le modèle choisi, et elle est cohérente avec le fait que le téléchargement initial des modèles nécessite de toute façon une connexion réseau (déjà le cas pour le code source et pour whisper.cpp lui-même). Le téléchargement doit rester ponctuel et explicite (jamais automatique/silencieux) pour respecter l'esprit « aucune donnée envoyée, mais aussi aucune surprise réseau » du produit.

Non traité dans ce spike : implémentation du gestionnaire de téléchargement, UI de sélection de modèle, gestion des reprises/erreurs réseau.

## 10. Risques

- **Qualité française non qualifiée** (`NEEDS_FRENCH_SAMPLE`) — aucun échantillon audio français représentatif n'était disponible dans le dépôt pour ce spike ; c'est le risque le plus critique avant de figer la décision en `ACCEPTED`.
- **Consommation mémoire du modèle `medium`** (~2,2 Go de pic mesuré) sur une machine à 16 Go — à valider en conditions réelles avec d'autres applications ouvertes.
- **Coût de chargement Metal au premier lancement** (~7 s observés) — à masquer côté UX.
- **Empaquetage FFmpeg** (licence, taille, build statique arm64) — non résolu, nécessite un spike ou une décision dédiée avant la V1.
- **whisper.cpp est un projet tiers en développement actif** — nécessité de figer une version (commit/tag) et de revalider à chaque mise à jour.
- **Alternative WhisperKit non comparée empiriquement** — la décision pourrait changer si le budget mémoire/énergie s'avère un problème réel sur du matériel contraint (ex. MacBook Air 8 Go, hors périmètre de la machine cible actuelle mais potentiellement pertinent plus tard).
