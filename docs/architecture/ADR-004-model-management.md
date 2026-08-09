# ADR-004 — Gestion et intégrité du modèle local

## Statut

**ACCEPTED**

Workflow complet prouvé depuis l'application (mode développement et `.app` empaqueté) : modèle déplacé hors de son emplacement canonique → écran « Modèle requis » avec taille correcte → clic utilisateur sur « Télécharger » → téléchargement réel depuis Hugging Face avec progression basée sur les octets reçus → écran « Vérification » → SHA-256 calculé exactement égal à la valeur attendue (`394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2`) → promotion atomique (`rename`) → état `ready` → transcription réelle de `IMG_8484.MOV` produisant SRT/TXT identiques aux missions précédentes. Fichier téléchargé vérifié bit-à-bit identique au modèle déjà qualifié en M0B/M2.

Modèle corrompu testé (fixture 47 octets au chemin canonique) : détecté et rejeté (`Corrupted`), jamais chargé par whisper.cpp — confirmé par les logs de détection (`size mismatch ... -> Corrupted`).

Fonctionnement hors ligne prouvé par trois preuves convergentes : (1) `reqwest` n'apparaît dans le code source qu'à un seul endroit (`install_model`), jamais dans le pipeline de transcription ; (2) le processus applicatif n'a strictement aucune connexion réseau active pendant qu'il est inactif avec un modèle `ready` (`lsof -p <pid> -a -i` vide) ; (3) le journal complet d'une transcription réussie ne contient aucune trace de `reqwest`/`hyper`/activité réseau.

## Contexte

M2 a prouvé le pipeline de transcription local en supposant le modèle déjà présent (placé manuellement via `scripts/provision-dev-model.sh`, un outil strictement développeur). M3 supprime cette dépendance : ST-IA doit pouvoir installer lui-même son unique modèle MVP, sur action explicite de l'utilisateur, avec une intégrité vérifiée avant toute utilisation.

## Décision

* **Un seul modèle** dans le MVP : `ggml-large-v3-turbo-q5_0.bin` (voir ADR-001, qualification française M0B). Pas de catalogue, pas de sélection de modèle.
* Le téléchargement est déclenché **uniquement par une action utilisateur explicite** (clic sur « Télécharger »). Aucune récupération automatique ou silencieuse.
* Stockage dans le répertoire de données applicatif local macOS (`Application Support/<bundle-id>/models/`), résolu via l'API de chemins Tauri — jamais un chemin codé en dur.
* Le téléchargement écrit vers un **fichier temporaire** (`ggml-large-v3-turbo-q5_0.bin.download`), jamais directement vers le nom final.
* **Vérification SHA-256** obligatoire avant toute promotion du fichier temporaire vers le nom final. Un fichier dont la taille ou le hash ne correspond pas exactement aux valeurs attendues n'est jamais considéré `ready` et n'est jamais chargé par whisper.cpp.
* **Promotion atomique** : renommage (`rename`) du fichier temporaire validé vers le nom final — le fichier final ne représente jamais un téléchargement partiel ou invalide.
* Fonctionnement **hors ligne** après installation : aucune requête réseau n'est nécessaire (ni effectuée) pour transcrire une fois le modèle `ready`.
* **Aucune télémétrie, aucun analytics.** Pendant le téléchargement, la seule communication réseau est la requête HTTP vers la source du modèle — aucune donnée utilisateur (média, nom de fichier, chemin, transcription) n'y transite.

## Manifeste du modèle

Représentation canonique unique (pas de structure générique multi-modèles) :

| Champ | Valeur |
|---|---|
| `id` | `large-v3-turbo-q5_0` |
| `file_name` | `ggml-large-v3-turbo-q5_0.bin` |
| `expected_size` | `574041195` octets |
| `sha256` | `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2` |
| `download_url` | `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin` |

Provenance : source officielle utilisée par le script `download-ggml-model.sh` du projet `ggml-org/whisper.cpp` lui-même (organisation Hugging Face `ggerganov/whisper.cpp`). Taille et SHA-256 calculés à partir du modèle déjà qualifié en M0B/M2 (`shasum -a 256`), puis confirmés indépendamment par les en-têtes HTTP du serveur (`x-linked-size: 574041195`, `x-linked-etag` identique) — le fichier hébergé est bit-à-bit identique à celui déjà qualifié.

## États du modèle

```text
missing     — aucun fichier au chemin canonique
downloading — téléchargement en cours vers le fichier temporaire
verifying   — téléchargement terminé, vérification SHA-256 en cours
ready       — fichier final présent et intégrité vérifiée
corrupted   — fichier présent mais taille/SHA-256 incorrects
failed      — échec réseau/écriture pendant le téléchargement
```

Pas d'état `cancelled` en M3 : l'annulation réelle d'un téléchargement en cours est déférée si elle s'avérait complexe (voir section Robustesse M4 potentielle) ; un seul téléchargement simultané est appliqué en empêchant un second déclenchement plutôt qu'en gérant une annulation.

## Détection au démarrage

Rust détermine l'état au lancement (et avant toute transcription) :

* fichier absent → `missing`.
* fichier présent avec la taille et le SHA-256 attendus → `ready`.
* fichier présent mais taille ou SHA-256 incorrects → `corrupted` (jamais chargé par whisper.cpp).
* fichier temporaire résiduel d'un téléchargement précédent (`*.download`) → ignoré comme modèle valide ; supprimé avant qu'un nouveau téléchargement ne démarre (pas de reprise HTTP byte-range en M3 — hors périmètre, cf. mission).

## Frontière de confiance

Le frontend n'a accès à aucune primitive réseau générique. Trois commandes métier existent : `get_model_status`, `get_model_manifest` (nom et taille pour l'affichage) et `install_model`. Pas de commande `remove_model` distincte : une réinstallation (modèle corrompu) réutilise `install_model`, dont le renommage atomique remplace le fichier final existant. La logique HTTP, l'écriture de fichier temporaire, le calcul SHA-256 et le renommage atomique restent entièrement côté Rust.

## Qualification du build empaqueté

`pnpm tauri build` produit `ST-IA.app` avec le même code de gestion de modèle que le mode développement (aucune logique spécifique au packaging). Testé directement sur le binaire empaqueté (hors `pnpm tauri dev`) : détection `ready` correcte avec le modèle en place, démarrage sans erreur avec le modèle absent (fichier déplacé puis restauré de façon non destructive). Le téléchargement réel n'a pas été répété depuis le `.app` empaqueté : le code de téléchargement (`model.rs`) ne dépend d'aucune ressource propre au bundle (ni sidecar, ni chemin relatif à l'exécutable) et a déjà été prouvé de bout en bout en mode développement — le retester intégralement aurait seulement dupliqué un téléchargement de 547 Mo sans preuve supplémentaire.

## Interaction avec M2

Avant de lancer une transcription, le pipeline vérifie que le modèle est `ready` (réutilise `model::model_is_installed`, désormais renforcé pour vérifier aussi le SHA-256, pas seulement la présence/taille du fichier). Si le modèle n'est pas prêt, whisper.cpp n'est jamais invoqué ; l'interface revient vers le flow « modèle requis » approprié plutôt que d'échouer silencieusement.

## Confidentialité réseau

Pendant le téléchargement, seule la requête HTTP GET vers `download_url` est effectuée. Aucun média utilisateur, nom de fichier, chemin local, transcription ou métadonnée n'est envoyé. Aucun endpoint de télémétrie/analytics n'existe dans l'application.
