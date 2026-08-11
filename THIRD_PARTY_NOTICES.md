# Composants tiers redistribués par ST-IA

ST-IA embarque deux binaires tiers dans son bundle macOS et télécharge un modèle tiers à la demande de l'utilisateur. Ce document liste ces composants, leur provenance vérifiable et leur licence telle qu'observée. Les textes de licence correspondants sont fournis dans `licenses/` et inclus dans l'application distribuée (`ST-IA.app/Contents/Resources/licenses/`).

Ce document rapporte des faits observables (version, provenance, options de build, licence annoncée par le composant). Il ne constitue pas un avis juridique.

**Ce document est distinct de la licence de ST-IA.** Le code de ST-IA est sous licence MIT (voir [`LICENSE`](LICENSE)) ; les composants listés ci-dessous conservent chacun leur propre licence, indépendante de celle-ci. Aucun n'est relicencié par ST-IA.

---

## 1. FFmpeg

| | |
|---|---|
| Version | 9.0 |
| Rôle dans ST-IA | Extraction et rééchantillonnage de la piste audio (média → WAV 16 kHz mono PCM16) |
| Provenance | `https://ffmpeg.org/releases/ffmpeg-9.0.tar.xz` |
| SHA-256 de la source | `7f607a00dd0d28a729d5a4811205812eef01cf6ef6155025febb6f36a9062d52` |
| Licence observée | LGPL version 2.1 or later — texte : `licenses/FFmpeg-LGPL-2.1.txt` |
| Site du projet | `https://ffmpeg.org` |

Compilé avec `--disable-gpl --disable-nonfree --disable-version3` : aucun composant GPL ou non-free n'est activé, et aucune bibliothèque externe (x264, LAME, FDK-AAC…) n'est liée. La configuration complète est reproduite par `scripts/build-ffmpeg-sidecar.sh` et reste lisible dans le binaire lui-même (`ffmpeg -version`).

**Sources correspondantes.** La version exacte, l'URL officielle et le SHA-256 ci-dessus permettent de retrouver et de recompiler la source utilisée ; `scripts/build-ffmpeg-sidecar.sh` reproduit le build à l'identique.

**Réserve LGPL avant distribution publique.** La LGPL 2.1 autorise la liaison statique dans une application propriétaire à condition de fournir un moyen de relier une version modifiée de la bibliothèque. ST-IA distribue FFmpeg comme un **exécutable séparé** (sidecar) invoqué en sous-processus, et non comme une bibliothèque liée dans l'exécutable ST-IA — ce qui simplifie la situation, l'utilisateur pouvant remplacer le binaire dans le bundle. Ce point doit être confirmé par un avis juridique avant toute distribution publique. Voir `docs/third-party/FFMPEG.md`.

---

## 2. whisper.cpp (et ggml)

| | |
|---|---|
| Version | v1.9.2 |
| Commit épinglé | `306c88f4d1286aec1bf96e544632897886af5501` |
| Rôle dans ST-IA | Transcription locale (WAV → SRT/TXT), accélérée par Metal |
| Provenance | `https://github.com/ggml-org/whisper.cpp` |
| Licence observée | MIT — texte : `licenses/whisper.cpp-MIT.txt` |
| Copyright | © 2023-2026 The ggml authors |

ggml est développé et distribué dans le même dépôt, sous la même licence MIT.

Compilé en statique, arm64, avec Metal et bibliothèque Metal embarquée, et **sans optimisation spécifique à la machine de build** (`GGML_NATIVE=OFF`, voir ADR-006). Le build est reproduit par `scripts/build-whisper-sidecar.sh`, qui vérifie le commit épinglé avant de compiler.

---

## 3. Modèle Whisper `ggml-large-v3-turbo-q5_0`

| | |
|---|---|
| Fichier | `ggml-large-v3-turbo-q5_0.bin` |
| Taille | 574 041 195 octets |
| SHA-256 | `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2` |
| Provenance | `https://huggingface.co/ggerganov/whisper.cpp` (révision épinglée `5359861c739e955e79d9a303bcbc70fb988958b1`) |

**Le modèle n'est pas redistribué par ST-IA.** Il n'est ni dans le dépôt, ni dans le `.app`, ni dans le `.dmg`. Il est téléchargé depuis sa source d'origine, sur action explicite de l'utilisateur, puis vérifié par SHA-256 avant utilisation (voir ADR-004).

Les poids Whisper proviennent d'OpenAI (`https://github.com/openai/whisper`), publiés sous licence MIT, et sont redistribués au format ggml par le projet whisper.cpp sur Hugging Face. ST-IA ne modifie pas ces poids.

---

## 4. Dépendances de l'application

L'exécutable ST-IA est construit avec Tauri 2 (Rust) et React/TypeScript. Ces dépendances ne sont pas redistribuées comme composants tiers distincts mais liées dans l'exécutable ; leurs licences respectives (majoritairement MIT/Apache-2.0) sont celles déclarées par `Cargo.toml`/`package.json` et leurs arbres de dépendances.

Aucun composant sous licence GPL ou non-free n'est embarqué.
