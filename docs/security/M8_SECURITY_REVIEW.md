# ST-IA — Revue de sécurité M8

Date : 2026-08-11
Périmètre : ST-IA 0.1.0 (`main` @ `5fdf487`), macOS Apple Silicon.
Question posée : *peut-on rendre ce code source public sans exposer de données
privées, de secrets, de vulnérabilité manifeste ou de chaîne de build incontrôlée ?*

Méthode : DISCOVERY → VALIDATION → SEVERITY → REMEDIATION → VERIFICATION.
Aucune modification de code n'a été faite avant que l'inventaire des findings soit clos.

**Aucun secret n'est reproduit dans ce document.** Il n'y en a d'ailleurs aucun à
rédiger : voir STIA-SEC-101.

---

## Résumé

| Sévérité | Nombre | Corrigés en M8 |
|---|---|---|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 3 | 3 |
| LOW | 2 | 2 |
| HARDENING | 1 | 1 |
| INFORMATIONAL | 6 | 3 (3 documentés) |

**Un point bloquant subsiste, et il n'est pas technique** : le projet n'a pas de
licence principale (STIA-SEC-201). C'est une décision utilisateur, pas un défaut de
sécurité.

Audits de dépendances : **0 vulnérabilité** côté npm, **0 vulnérabilité** côté Rust.
Scan de secrets sur les 43 commits de l'historique : **0 finding**.

---

## Findings

### STIA-SEC-001 — Chemin média non validé à la frontière IPC

* **Severity:** MEDIUM
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/src/commands/transcription.rs`, `src-tauri/src/pipeline.rs`

**Evidence.** `start_transcription` acceptait `media_path: String` et le transmettait
tel quel à `pipeline::run`, qui l'utilisait pour deux choses sans jamais le valider :
`build_ffmpeg_args` (argument `-i` de FFmpeg) et `resolve_output_dir` (le répertoire de
sortie est dérivé du **répertoire parent du média**). La validation existait bien —
`validate_media_path` — mais uniquement dans la commande `inspect_media`, que rien
n'oblige un appelant à invoquer d'abord.

**Impact.** Depuis une exécution JavaScript arbitraire dans la WebView :

1. transcrire n'importe quel média lisible par l'utilisateur, hors de ceux qu'il a
   choisis, et en récupérer le texte via le champ `transcriptText` de l'état
   `Completed` ;
2. plus sérieusement, provoquer la création d'un répertoire `<chemin>-sous-titres/`
   et l'écriture de fichiers `.srt`/`.txt` **à n'importe quel emplacement inscriptible
   par l'utilisateur** — par exemple `~/Library/LaunchAgents/`.

**Circonstances atténuantes** (qui expliquent MEDIUM et non HIGH) : le sidecar FFmpeg
est compilé sans réseau et avec le seul protocole `file`, donc aucune SSRF n'est
possible ; les sidecars sont lancés avec un vecteur argv sans shell, donc aucune
injection de commande ; et le contenu écrit n'est pas contrôlé par l'attaquant.

**Remediation.** `start_transcription` appelle désormais `validate_media_path` avant de
revendiquer le slot de job. Le chemin doit être un fichier régulier existant, non vide,
lisible, avec une extension parmi les six supportées. Les échecs sont mappés sur
`audioPreparationFailed`, un code d'erreur qui existe déjà dans les catalogues FR/EN —
aucun changement de contrat ni d'i18n.

**Verification.** 6 nouveaux tests dans `domain/media.rs` couvrant répertoires,
chaînes d'URL (`http://`, `concat:`, `pipe:`), traversée (`../../../../etc/passwd`),
fichiers sans extension, liens symboliques vers un non-média, et — dans l'autre sens —
l'acceptation de noms légitimes mais hostiles en apparence (`$(whoami)-\`id\`.mp3`,
`semi;colon && pipe|.wav`, unicode, emoji, tiret initial).

---

### STIA-SEC-002 — CSP désactivée

* **Severity:** MEDIUM
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/tauri.conf.json`

**Evidence.** `app.security.csp` valait `null`, signalé comme réserve dès M5 sans être
traité depuis.

**Impact.** Aucune défense en profondeur : tout script injecté (par une dépendance npm
compromise, seul vecteur réaliste ici) disposait de la surface IPC complète et pouvait
tenter d'ouvrir un canal d'exfiltration.

**Remediation.** Politique construite à partir des ressources réellement émises par le
build, pas copiée d'un modèle générique. Le build Vite produit exactement un JS et un
CSS externes, sans script ni style inline, et le code n'utilise aucun `style={{…}}` —
ce qui permet une politique **sans `unsafe-inline` ni `unsafe-eval`** :

```
default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;
font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; media-src 'none';
object-src 'none'; frame-src 'none'; worker-src 'none'; child-src 'none';
manifest-src 'none'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'
```

`connect-src` inclut `ipc: http://ipc.localhost` parce que l'IPC Tauri v2 en a besoin ;
il n'y a aucun hôte distant autorisé. Le téléchargement du modèle est fait par `reqwest`
côté Rust et n'est donc pas soumis à la CSP.

**Verification.** Vérifiée sur la **build release empaquetée** (`pnpm tauri build`) :
la politique est bien embarquée dans le binaire, l'application démarre, le frontend
s'initialise et l'IPC fonctionne (le log `model detect` prouve que
`get_model_status` a bien été invoqué depuis React), **0 violation CSP**. La
qualification interactive complète (Réglages, i18n, transcription, opener) reste un
gate humain.

---

### STIA-SEC-003 — `open_output_folder` acceptait un chemin arbitraire

* **Severity:** LOW
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/src/commands/transcription.rs`

**Evidence.** La commande prenait `path: String` du frontend et le passait directement
à `reveal_item_in_dir`.

**Impact.** Depuis un frontend compromis : ouvrir le Finder sur un emplacement
arbitraire, et surtout utiliser la réponse ok/erreur comme **oracle d'existence de
fichier** pour cartographier le disque. Pas d'exécution de code : la capability
n'accorde que `opener:allow-reveal-item-in-dir`, jamais `allow-open-path`.

**Remediation.** Le paramètre a été **supprimé**, plutôt que validé. La commande lit
maintenant l'état `Completed` du backend et révèle le premier fichier généré (repli sur
`output_dir`). Il n'y a plus de chemin fourni par le frontend à valider. Le frontend
appelle `invoke("open_output_folder")` sans argument.

**Verification.** `cargo test` (78 tests) et build release verte. Le comportement
utilisateur est identique : la même cible était déjà celle que le frontend
choisissait.

---

### STIA-SEC-004 — Téléchargement du modèle sans plafond de taille

* **Severity:** MEDIUM
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/src/model.rs`

**Evidence.** `download_to_temp` écrivait chaque chunk reçu jusqu'à ce que le serveur
ferme le flux. La vérification SHA-256 n'intervient qu'**après** la fin du
téléchargement.

**Impact.** Un endpoint hostile (mirror compromis, DNS détourné) pouvait streamer
indéfiniment et **remplir le disque** avant que le contrôle d'intégrité n'ait
l'occasion de rejeter le fichier. L'intégrité n'était pas menacée ; le coût pour la
constater l'était.

**Remediation.** Le stream est interrompu dès que le cumul dépasse
`MODEL_EXPECTED_SIZE`, avec une `ModelError::network` explicite. Le fichier `.download`
partiel est supprimé par le nettoyage au démarrage suivant.

**Verification.** Tests existants du gestionnaire de modèle inchangés et verts. La
condition est un simple comparateur sur un compteur déjà présent.

---

### STIA-SEC-005 — Nettoyage au démarrage suivait les liens symboliques

* **Severity:** LOW
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/src/cleanup.rs`

**Evidence.** `clean_stale_job_dirs` utilisait `path.is_dir()`, qui **suit** les liens
symboliques. Un lien nommé `<pid>-<nanos>` pointant hors du répertoire temporaire aurait
été rapporté comme un répertoire, puis passé à `remove_dir_all`.

**Impact.** Réel mais étroit : `std::env::temp_dir()` renvoie sur macOS un répertoire
`$TMPDIR` **propre à l'utilisateur** (mode 700), pas `/tmp` partagé. Un attaquant
capable d'y créer un lien a déjà l'exécution de code sous ce compte. Corrigé au titre
de la défense en profondeur.

**Remediation.** `symlink_metadata` au lieu de `is_dir()`, avec refus explicite de
toute entrée qui est un lien — quelle que soit sa cible, puisque ST-IA ne supprime
jamais que des répertoires qu'elle a elle-même créés.

**Verification.** Test adversarial `never_follows_a_job_shaped_symlink` : un lien nommé
`1234-56789` pointant vers un répertoire contenant un fichier utilisateur ; le fichier
cible et le lien doivent tous deux survivre. Plus
`traversal_shaped_names_are_not_job_dirs` sur `..`, `.`, `../..`, `1234-56789/../..`.

---

### STIA-SEC-006 — Client HTTP autorisant un déclassement de schéma

* **Severity:** HARDENING
* **Status:** VALID → **CORRIGÉ**
* **Component:** `src-tauri/src/model.rs`

**Evidence.** `reqwest::Client::new()` utilise la politique de redirection par défaut,
qui suit jusqu'à 10 redirections **y compris de `https` vers `http`**. L'URL du modèle
sur Hugging Face redirige effectivement vers un CDN.

**Impact.** Faible : le SHA-256 rend inexploitable toute substitution de fichier, et
aucun secret n'est envoyé dans la requête. C'est un problème de transport, pas
d'intégrité.

**Remediation.** `https_only(true)`, redirections bornées à 5, `connect_timeout` de
30 s (pour qu'un hôte injoignable ne fige pas indéfiniment l'état « Téléchargement »).
Volontairement **pas** de timeout global sur le corps : 574 Mo sur une liaison lente
est légitimement long.

---

### STIA-SEC-101 — Aucun secret dans le dépôt ni dans l'historique

* **Severity:** INFORMATIONAL
* **Status:** VÉRIFIÉ — rien à corriger

**Méthode.**

1. `gitleaks git --log-opts="--all"` sur la totalité de l'historique (43 commits,
   ~654 Ko scannés) → **0 finding**.
2. Extraction manuelle des **265 blobs texte** ayant jamais existé, puis grep sur
   ~30 motifs (`api_key`, `token`, `password`, `BEGIN … PRIVATE KEY`, `AKIA…`,
   `ghp_…`, `xox…`, `sk-…`, `APPLE_ID`, `TEAM_ID`, `.p12`, `.mobileprovision`,
   `keychain`, …) → **0 correspondance**.
3. Comparaison exhaustive des chemins : tout objet de tout arbre de tout commit a été
   listé et comparé à `git ls-files`. **Aucun fichier n'a jamais existé dans
   l'historique sans être présent dans HEAD** — il n'y a donc aucun fichier supprimé
   dans lequel un secret pourrait subsister.

**Conclusion.** `HISTORY_REWRITE_REQUIRED` : **NON**. Aucune réécriture d'historique
n'est nécessaire.

C'est cohérent avec la nature du produit : ST-IA n'a ni compte, ni backend, ni API key,
ni télémétrie. Il n'existe aucun secret à fuiter.

> Un scan `gitleaks dir .` du répertoire de travail remonte 9 correspondances, toutes
> dans des chemins **non suivis et gitignorés** : `build-tmp/` (source FFmpeg amont),
> `engine/whisper.cpp/` (clone amont) et `src-tauri/target/` (artefacts de compilation).
> Aucune n'est dans le dépôt, et toutes sont des faux positifs `generic-api-key`.

---

### STIA-SEC-102 — Chemin de développeur dans les logs de spike suivis

* **Severity:** INFORMATIONAL
* **Status:** VALID → **CORRIGÉ (HEAD uniquement)**
* **Component:** `spike/out/*/run.stderr.log`

**Evidence.** 10 lignes contenaient `/Volumes/Workspace/Projects/ST-IA/…`, sortie brute
de whisper.cpp lors de la qualification M0.

**Impact.** Catégorie B — chemin de développeur sans donnée sensible : pas de nom
d'utilisateur, pas de répertoire personnel. Aucune valeur pour un attaquant.

**Remediation.** Préfixe remplacé par `$REPO_ROOT` dans HEAD. **Aucune** mesure
d'écriture (durées, nombre d'échantillons, sorties du modèle) n'a été modifiée : la
valeur probante de ces logs est intacte.

**L'historique n'est délibérément pas réécrit** pour ce point : un chemin de
développeur non sensible ne justifie pas de réécrire 44 commits et de casser les
références des 6 PR déjà fusionnées.

---

### STIA-SEC-103 — Nom de média personnel référencé dans la documentation

* **Severity:** INFORMATIONAL
* **Status:** VALID — **décision utilisateur**
* **Component:** `docs/ROADMAP.md`, ADR-003/004/005/007

**Evidence.** `IMG_8484.MOV` apparaît 28 fois dans la documentation comme fixture de
qualification.

**Le fichier lui-même n'a jamais été committé** — vérifié exhaustivement (voir
STIA-SEC-101, point 3). Il vit dans `mockups/`, correctement gitignoré. Seul le **nom**
apparaît, et il révèle uniquement qu'il s'agit d'un enregistrement iPhone.

**Recommandation : ne rien changer.** Ce nom est l'identifiant de la chaîne de preuves
qui relie cinq ADR entre elles ; le remplacer par un nom générique casserait la
traçabilité de la qualification sans gain de confidentialité réel. Si vous préférez
malgré tout l'anonymiser, c'est une modification de documentation triviale (HEAD
uniquement, sans réécriture d'historique) — dites-le et je la fais.

---

### STIA-SEC-104 — Chemin de développeur embarqué dans le binaire FFmpeg

* **Severity:** INFORMATIONAL
* **Status:** VALID — **non corrigé, documenté**
* **Component:** `src-tauri/binaries/ffmpeg-aarch64-apple-darwin`

**Evidence.** `ffmpeg -version` affiche sa chaîne de configuration, qui contient
`--prefix=/Volumes/Workspace/Projects/ST-IA/build-tmp/ffmpeg-9.0/dist`.

**Impact.** Même catégorie que STIA-SEC-102, mais dans un binaire **distribué**.

**Remediation proposée (non appliquée).** Utiliser un `--prefix` neutre dans
`scripts/build-ffmpeg-sidecar.sh`. Le sidecar n'a **pas** été reconstruit : la mission
proscrit explicitement de reconstruire « par sécurité », et un rebuild produirait un
binaire dont la qualification M2/M5 (portabilité, sorties octet pour octet) devrait
être rejouée. À traiter au prochain rebuild légitime.

---

### STIA-SEC-105 — Avis RustSec : 17 warnings, 0 vulnérabilité

* **Severity:** INFORMATIONAL
* **Status:** ANALYSÉ — aucune action

`cargo audit` sur 505 crates transitives : **0 vulnérabilité**, 17 warnings.

| Groupe | Crates | Analyse |
|---|---|---|
| Bindings GTK3 | `atk`, `gdk`, `gdkx11`, `gdkwayland-sys`, `gtk`, `gtk-sys`, `atk-sys`, … (11) | `unmaintained`. Dépendances **Linux uniquement** de Tauri, jamais compilées sur macOS. Non atteignables. |
| `glib` 0.18.5 | 1 | `unsound` (RUSTSEC-2024-0429), `VariantStrIter`. Même chaîne GTK3, non compilée sur macOS. |
| `unic-*` | 5 | `unmaintained`. Transitives (chaîne IDNA). Pas de vulnérabilité connue. |
| `proc-macro-error` | 1 | `unmaintained`. **Build-time uniquement**, absente du binaire livré. |

Aucune ne justifie une mise à jour forcée. Toutes disparaîtront en amont via Tauri.
Le workflow `security.yml` les resurveille chaque semaine.

---

### STIA-SEC-106 — Sidecars binaires suivis dans Git

* **Severity:** INFORMATIONAL (compromis assumé)
* **Status:** DOCUMENTÉ

| Binaire | Taille | SHA-256 (16 premiers) | Provenance |
|---|---|---|---|
| `ffmpeg-aarch64-apple-darwin` | 3,53 Mo | `87610d7842f2c3f3` | FFmpeg 9.0, source officielle, tarball vérifié SHA-256 par le script de build |
| `whisper-cli-aarch64-apple-darwin` | 3,28 Mo | `a106f36d8c32f148` | whisper.cpp v1.9.2 @ `306c88f4d128…`, commit épinglé et vérifié par le script |

**Pourquoi ils sont suivis.** whisper.cpp ne publie pas de binaire arm64 statique
officiel, et exiger une build FFmpeg complète pour lancer l'application rendrait le
projet inutilisable pour un contributeur.

**Le risque, énoncé franchement.** Un contributeur ne peut pas vérifier trivialement
que ces binaires correspondent à leur source. C'est une relation de confiance envers le
mainteneur. Mitigations : scripts de build déterministes et pinnés (checksum du tarball
FFmpeg, SHA du commit whisper.cpp), garde-fous qui font échouer la build en cas de
dépendance rpath/Homebrew ou de `-mcpu=native`, `otool -L` vérifié (frameworks Apple
uniquement), SHA-256 identiques entre les binaires committés et ceux du `.app`
empaqueté (vérifié en M8).

**Taille totale : 6,8 Mo** — négligeable pour un dépôt Git.

Une distribution par release GitHub plutôt que par Git serait plus propre. C'est un
sujet M9, pas M8.

---

### STIA-SEC-201 — Aucune licence principale pour le projet

* **Severity:** **BLOQUANT pour l'ouverture** (non technique)
* **Status:** **`PROJECT_LICENSE_DECISION_REQUIRED`**

**Evidence.** Aucun fichier `LICENSE` ni `COPYING` à la racine. `package.json` ne
déclare aucun champ `license`. Le README ne mentionne que les licences **tierces**.

**Impact.** Du code publié sans licence reste sous **copyright exclusif par défaut**.
Personne n'a le droit légal de l'utiliser, de le modifier ni de le redistribuer, quand
bien même les sources sont visibles. Un dépôt public sans licence n'est pas un projet
open source.

**Aucune licence n'a été choisie par M8** — c'est une décision qui appartient à
l'auteur. Voir `OPEN_SOURCE_READINESS.md` §Licence pour les implications factuelles des
options courantes.

**Contrainte à connaître** : ST-IA distribue FFmpeg (LGPL-2.1) comme exécutable
sidecar séparé. Voir STIA-SEC-202.

---

### STIA-SEC-202 — Conformité LGPL de FFmpeg : à confirmer par un tiers

* **Severity:** INFORMATIONAL — **`LEGAL_REVIEW_RECOMMENDED`**
* **Status:** DOCUMENTÉ

**État des faits.** FFmpeg 9.0 est distribué comme **exécutable séparé** dans
`ST-IA.app/Contents/MacOS/ffmpeg`, invoqué par création de processus. Il n'est ni lié
statiquement ni dynamiquement au code de ST-IA. La build est explicitement
`--disable-gpl --disable-nonfree --disable-version3` : seuls des composants LGPL-2.1
sont présents. Le texte de la licence est embarqué dans le bundle
(`Contents/Resources/licenses/`), les notices dans `THIRD_PARTY_NOTICES.md`, et la
version, la configuration et l'URL source exactes sont documentées dans
`docs/third-party/FFMPEG.md` — un tiers peut retrouver et reconstruire la source
correspondante.

**Ce que M8 ne fait pas.** Émettre une conclusion juridique. La distribution en
exécutable séparé est largement considérée comme la voie la plus simple pour la
conformité LGPL, mais « largement considérée » n'est pas un avis juridique, et cette
mission n'est pas qualifiée pour en rendre un.

**Ce n'est pas un défaut de sécurité.** C'est un point à faire confirmer avant une
distribution publique large, en même temps que la décision de licence
(STIA-SEC-201).

---

## Surface d'audit — commandes Tauri

Les 11 commandes exposées, telles qu'elles sont **après** M8.

| Commande | Entrée frontend | Validation | fs | process | réseau | Abus depuis un frontend compromis |
|---|---|---|---|---|---|---|
| `inspect_media` | `path: String` | ✅ `validate_media_path` | lecture métadonnées | — | — | Oracle d'existence, borné aux 6 extensions |
| `start_transcription` | `path`, 2 bool | ✅ **ajouté en M8** | lecture média, écriture temp + sortie | ffmpeg, whisper | — | Transcrire un média que l'utilisateur possède |
| `get_transcription_status` | — | n/a | — | — | — | Aucun |
| `cancel_transcription` | — | n/a | — | kill enfant | — | Déni de service sur son propre job |
| `open_output_folder` | **aucune** ✅ *(M8)* | dérivé de l'état | reveal Finder | — | — | Négligeable |
| `get_model_status` | — | n/a | lecture + SHA-256 | — | — | Aucun |
| `get_model_manifest` | — | n/a | — | — | — | Aucun |
| `install_model` | — | n/a | écriture models/ | — | ✅ URL épinglée | Consommation bornée (plafond M8) |
| `get_settings` | — | n/a | lecture settings | — | — | Aucun |
| `save_settings` | `Settings` | ✅ 3 énumérations fermées | écriture settings | — | — | Aucun |
| `get_app_version` | — | n/a | — | — | — | Aucun |

**Aucune commande ne prend une URL, un argument de processus ou un chemin de
suppression.** Aucune ne renvoie un contenu de fichier arbitraire.

### Capabilities

```json
["core:default", "dialog:allow-open",
 { "identifier": "shell:allow-execute",
   "allow": [{ "name": "whisper-cli", "sidecar": true, "args": true },
             { "name": "ffmpeg", "sidecar": true, "args": true }] },
 "opener:allow-reveal-item-in-dir"]
```

Auditées, **inchangées** : elles sont déjà minimales. Pas de `shell:allow-spawn`, pas
de permission shell générique, pas de plugin `fs` (le frontend n'a aucun accès direct au
système de fichiers), `opener` limité à `reveal-item-in-dir` seul. M7 (réglages/i18n)
n'a élargi aucune permission — `get_settings`/`save_settings` passent par l'IPC normale.

Les sidecars sont nommés explicitement ; `args: true` autorise le passage d'arguments,
mais ces arguments sont construits **côté Rust** par `build_ffmpeg_args` /
`build_whisper_args`, jamais fournis par le frontend.

---

## Vérification

| Vérification | Résultat |
|---|---|
| `pnpm build` | ✅ |
| `pnpm test` | ✅ 21 tests |
| `cargo fmt --check` | ✅ |
| `cargo clippy --all-targets` | ✅ 0 warning |
| `cargo test` | ✅ **78 tests** (67 avant M8) |
| `pnpm tauri build` | ✅ `.app` + `.dmg` |
| Lancement de l'app empaquetée sous CSP | ✅ frontend initialisé, IPC fonctionnelle, **0 violation CSP** |
| `pnpm audit` | ✅ 0 vulnérabilité (165 paquets) |
| `cargo audit` | ✅ 0 vulnérabilité (505 crates), 17 warnings analysés |
| `gitleaks` historique complet | ✅ 0 finding (43 commits) |
| SHA-256 sidecars : committés vs `.app` | ✅ identiques |
| `otool -L` sidecars | ✅ frameworks Apple uniquement |
| `ffmpeg -protocols` | ✅ `file` seul, en entrée comme en sortie |
| Fichiers suivis masqués par le nouveau `.gitignore` | ✅ **0** |
