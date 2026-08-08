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

## Sécurité

Le frontend appelle uniquement des commandes Rust métier (`start_transcription`, `get_transcription_status`, `open_output_folder`). Aucune capability `shell:allow-execute` n'est exposée au frontend ; l'exécution des sidecars passe par `tauri_plugin_shell::ShellExt` côté Rust (`app.shell().sidecar(...)`), pas par `@tauri-apps/plugin-shell` côté JavaScript. Les seules permissions shell déclarées dans les capabilities ciblent explicitement les deux sidecars nommés, jamais un binaire ou des arguments arbitraires. L'ouverture du dossier de sortie utilise `tauri-plugin-opener` (`opener:allow-open-path`, scope `$HOME/**` et `/Volumes/**`), pas l'API `shell.open` dépréciée.

## Modèle

Le modèle MVP est `large-v3-turbo-q5_0` (voir ADR-001, section qualification française). Cette mission (M2) utilise l'emplacement canonique du modèle mais ne gère ni son téléchargement, ni sa vérification SHA-256, ni sa reprise après interruption — cela appartient à M3. Un script strictement développeur (`scripts/provision-dev-model.sh`) permet de placer manuellement un modèle déjà téléchargé à cet emplacement pour tester M2 ; il ne doit jamais devenir une fonctionnalité utilisateur et le modèle n'est jamais commité.

## Alternatives non retenues pour le packaging

* **Lier dynamiquement whisper.cpp** (comme le spike Mission 0) : rejeté — introduit une dépendance aux dylibs du clone de développement, absent chez l'utilisateur final.
* **Dépendre d'un FFmpeg Homebrew/système** : rejeté — viole la contrainte « aucune dépendance système chez l'utilisateur final » (voir contexte produit, Mission 0B).
* **Télécharger les sidecars à l'installation** : envisageable plus tard mais hors périmètre M2 ; complique la distribution offline-first sans bénéfice immédiat.
