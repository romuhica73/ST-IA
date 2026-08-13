# ADR-003 — Pipeline local et packaging des moteurs

## Statut

**ACCEPTED**

Preuve de bout en bout obtenue depuis l'application ST-IA elle-même (pas seulement en ligne de commande) : `IMG_8484.MOV` sélectionné dans l'UI → sidecar FFmpeg ST-IA → `audio.wav` (16 kHz/mono/PCM16) → sidecar whisper-cli ST-IA (modèle `large-v3-turbo-q5_0`, langue forcée `fr`, backend Metal) → `IMG_8484.srt` (4959 octets, UTF-8, timecodes) et `IMG_8484.txt` (3428 octets, UTF-8) écrits dans `IMG_8484-sous-titres/`, temporaire nettoyé. `otool -L` sur les deux sidecars à leur emplacement d'exécution réel (`target/debug/ffmpeg`, `target/debug/whisper-cli`) ne montre que des frameworks système Apple — aucune dépendance Homebrew ni au clone `engine/whisper.cpp`.

Un bug de résolution de sidecar a été identifié et corrigé pendant cette qualification (voir section « Point de vigilance — nommage des sidecars » ci-dessous) ; la preuve ci-dessus a été obtenue après correction.

## Contexte

ADR-001 a retenu `whisper.cpp` (modèle `large-v3-turbo-q5_0`) comme moteur de transcription. ADR-002 a fixé la frontière de confiance Tauri (React = interface, Rust = orchestration système). Cette ADR documente comment ces deux décisions se concrétisent en un pipeline exécutable : quels binaires tournent, comment ils sont construits, où ils vivent dans l'application, et comment Rust les orchestre sans jamais exposer de shell arbitraire au frontend.

## Décision

* FFmpeg est exécuté comme **sidecar local**, binaire statique arm64 construit depuis la source officielle (voir `docs/third-party/FFMPEG.md`) — jamais `/opt/homebrew/bin/ffmpeg` ni un FFmpeg système.
* `whisper-cli` est exécuté comme **sidecar local**, binaire statique arm64 construit depuis le clone pinné `engine/whisper.cpp` (v1.9.2, commit `306c88f4d1286aec1bf96e544632897886af5501`) — sans dépendance aux dylibs `@rpath` du clone de développement.
* L'orchestration (choix des chemins, construction des arguments, lancement des process, lecture des sorties) est **exclusivement côté Rust**. React ne lance jamais de process et ne dispose d'aucune primitive générique `run_command(path, args)`.
* Un fichier WAV intermédiaire (16 kHz, mono, PCM16) est créé dans un répertoire temporaire propre au job, jamais à la place du média source.
* Le modèle (`ggml-large-v3-turbo-q5_0.bin`) vit dans le répertoire de données applicatif local macOS (`Application Support/<bundle-id>/models/`), résolu via l'API de chemins Tauri — jamais un chemin absolu codé en dur.
* Aucune dépendance Python ni Homebrew au runtime.

## Packaging

Les deux sidecars sont préparés par des scripts de build dédiés et reproductibles :

* `scripts/build-whisper-sidecar.sh` — reconfigure et recompile `engine/whisper.cpp` avec `BUILD_SHARED_LIBS=OFF`, `GGML_STATIC=ON`, `GGML_METAL=ON`, `GGML_METAL_EMBED_LIBRARY=ON`, vérifie l'absence de dépendance `@rpath`/Homebrew/clone via `otool -L`, copie le résultat vers `src-tauri/binaries/whisper-cli-aarch64-apple-darwin`.
* `scripts/build-ffmpeg-sidecar.sh` — télécharge FFmpeg 9.0 depuis la source officielle (SHA-256 vérifié), compile une configuration minimale sans GPL/non-free, vérifie l'architecture et l'absence de dépendance Homebrew, copie le résultat vers `src-tauri/binaries/ffmpeg-aarch64-apple-darwin`.

Les deux binaires (~3 Mo chacun) sont **commités** dans le dépôt : ils sont petits, déterministes à reconstruire, et leur présence évite à chaque contributeur/CI de recompiler FFmpeg (plusieurs minutes) juste pour lancer l'application. Le clone `engine/whisper.cpp` reste gitignoré (c'est un dépôt Git à part entière avec des modèles de plusieurs Go) ; seuls les binaires de sortie sont versionnés.

Tauri 2 est configuré via `bundle.externalBin` (`binaries/whisper-cli`, `binaries/ffmpeg`), convention de nommage source `<nom>-<target-triple>` — vérifiée pour `aarch64-apple-darwin`.

### Point de vigilance — nommage des sidecars à l'exécution

`externalBin` (dans `tauri.conf.json`) et les capabilities/appels `.sidecar(...)` n'utilisent **pas** la même forme de nom, et confondre les deux casse la résolution du sidecar au runtime :

* `externalBin` référence le **chemin source** (`binaries/ffmpeg` → cherche `src-tauri/binaries/ffmpeg-aarch64-apple-darwin`).
* `tauri-build` (`copy_binaries`) copie ce fichier **à plat** à côté de l'exécutable (`target/debug/ffmpeg` en dev), en retirant à la fois le sous-dossier `binaries/` et le suffixe `-aarch64-apple-darwin`.
* `app.shell().sidecar(...)` et le champ `name` des capabilities doivent donc utiliser le **nom nu** (`"ffmpeg"`, `"whisper-cli"`), sans préfixe de dossier.

Ce point a été découvert lors de la qualification M2 : un premier essai utilisait `.sidecar("binaries/ffmpeg")`, qui cherchait un fichier `target/debug/binaries/ffmpeg` inexistant, provoquant un échec silencieux classé `audioPreparationFailed`. Confirmé par lecture du code source de `tauri-build`/`tauri-plugin-shell` et par test isolé du sidecar (succès hors application) avant correction.

### Qualification du build empaqueté

`pnpm tauri build` produit `ST-IA.app` avec les deux sidecars placés à plat dans `Contents/MacOS/` (`ffmpeg`, `whisper-cli`), confirmant que la convention de nommage ci-dessus est identique en développement et en production. `otool -L` sur les deux binaires du bundle ne montre que des frameworks système Apple. L'application se lance sans blocage macOS (seul un message `spctl` informatif sur l'absence de ressources signées apparaît — pas un blocage de lancement). Pipeline testé deux fois depuis le `.app` empaqueté avec `IMG_8484.MOV` : SRT et TXT générés à l'identique du test en mode développement (4959 et 3428 octets), dossiers de sortie `-2`/`-3` créés correctement par la stratégie anti-collision.

### Réserve de portabilité — `PORTABILITY_APPLE_SILICON_TO_REQUALIFY_BEFORE_PUBLIC_DISTRIBUTION`

Le sidecar `whisper-cli` est compilé avec `-mcpu=native` (auto-détecté par le CMake de `ggml` en l'absence d'une cible explicite), ce qui optimise le binaire pour le CPU exact de la machine de build (Apple M4). Sa compatibilité avec d'autres puces Apple Silicon (M1/M2/M3, ou de futures générations) n'a pas été vérifiée et pourrait théoriquement provoquer une instruction illégale sur du matériel plus ancien. Cette réserve n'a pas été levée pendant M2 (hors périmètre) et devra être requalifiée au plus tard avant M5 (distribution publique) — soit en fixant une cible `-mcpu` compatible avec la plage Apple Silicon visée, soit en documentant une exigence matérielle minimale.

## Sécurité

Le frontend appelle uniquement des commandes Rust métier (`start_transcription`, `get_transcription_status`, `open_output_folder`). L'exécution des sidecars passe par `tauri_plugin_shell::ShellExt` côté Rust (`app.shell().sidecar(...)`), pas par `@tauri-apps/plugin-shell` côté JavaScript — ce paquet n'est d'ailleurs pas une dépendance du frontend. Les permissions shell déclarées dans les capabilities ciblent explicitement les **deux sidecars nommés**, jamais un binaire arbitraire.

> **Corrigé par M10 sur deux points où ce paragraphe ne décrivait plus le code.**
>
> 1. Il affirmait qu'« aucune capability `shell:allow-execute` n'est exposée au frontend ». C'est inexact : `capabilities/main.json` **accorde bien** `shell:allow-execute` à la fenêtre, borné aux deux sidecars nommés mais avec `"args": true`. La permission n'est pas nécessaire au fonctionnement — c'est Rust qui lance les sidecars, et un appel Rust ne traverse pas le système de capabilities — donc la retirer réduirait la surface sans rien casser. Voir le finding M10-F11.
> 2. L'ouverture du dossier de sortie n'utilise plus `opener:allow-open-path` avec un scope `$HOME/**` et `/Volumes/**` : M8 (STIA-SEC-003) l'a remplacé par `opener:allow-reveal-item-in-dir` **seul**, sans scope de chemin. La surface réelle est donc plus étroite que ce que décrivait cette ligne.

## Modèle

Le modèle MVP est `large-v3-turbo-q5_0` (voir ADR-001, section qualification française). Cette mission (M2) utilise l'emplacement canonique du modèle mais ne gère ni son téléchargement, ni sa vérification SHA-256, ni sa reprise après interruption — cela appartient à M3. Un script strictement développeur (`scripts/provision-dev-model.sh`) permet de placer manuellement un modèle déjà téléchargé à cet emplacement pour tester M2 ; il ne doit jamais devenir une fonctionnalité utilisateur et le modèle n'est jamais commité.

## Alternatives non retenues pour le packaging

* **Lier dynamiquement whisper.cpp** (comme le spike Mission 0) : rejeté — introduit une dépendance aux dylibs du clone de développement, absent chez l'utilisateur final.
* **Dépendre d'un FFmpeg Homebrew/système** : rejeté — viole la contrainte « aucune dépendance système chez l'utilisateur final » (voir contexte produit, Mission 0B).
* **Télécharger les sidecars à l'installation** : envisageable plus tard mais hors périmètre M2 ; complique la distribution offline-first sans bénéfice immédiat.
