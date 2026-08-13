# Portage Windows — plan

Statut public : **`NOT_YET_SUPPORTED`**

Aucune build Windows n'existe, aucune n'est testée, et **aucune date n'est
annoncée**. Ce document existe pour que le travail nécessaire soit connu et
chiffrable, pas pour promettre une échéance.

M10 n'implémente rien de ce qui suit.

---

## Où en est le code aujourd'hui

Le code n'est pas *hostile* au portage, mais il n'est pas portable non plus.
Trois choses le clouent à macOS Apple Silicon :

1. **Les sidecars committés sont des Mach-O arm64.** `ffmpeg-aarch64-apple-darwin`
   et `whisper-cli-aarch64-apple-darwin` ne s'exécuteront jamais sur Windows.
2. **L'accélération est Metal.** whisper.cpp est compilé avec le backend Metal,
   qui n'existe pas sur Windows.
3. **Rien d'autre n'est testé.** Aucune CI Windows, aucune QA Windows.

Ce qui est déjà favorable : la logique métier Rust est majoritairement pure et
testée hors plateforme (`domain/`), l'interface est du web, et Tauri 2 cible
Windows nativement.

---

## 1. Cible et toolchain

| Élément | macOS (actuel) | Windows (à faire) |
|---|---|---|
| Target Rust | `aarch64-apple-darwin` | `x86_64-pc-windows-msvc` |
| Webview | WKWebView | WebView2 (Edge) — **runtime à vérifier ou embarquer** |
| Toolchain | Xcode CLT | Visual Studio Build Tools (MSVC) + Windows SDK |
| Archi retenue | arm64 | **x64 d'abord.** arm64 Windows plus tard, s'il y a une demande |

WebView2 est le premier piège de distribution : il est présent sur Windows 11
et sur un Windows 10 à jour, mais pas garanti. Tauri sait produire un
installateur qui l'embarque ou le télécharge — décision à prendre au moment du
packaging, pas avant.

## 2. Sidecars

Le plus gros du travail.

**FFmpeg** — `scripts/build-ffmpeg-sidecar.sh` est un script shell POSIX qui
construit avec `configure`. Sur Windows il faut soit un équivalent MSYS2/MinGW,
soit un binaire officiel Windows LGPL vérifié par checksum. **La contrainte de
licence ne change pas** : pas de composant GPL, pas de `--enable-gpl`, mêmes
options minimales (voir [`../third-party/FFMPEG.md`](../third-party/FFMPEG.md)).
Attention : beaucoup de builds Windows FFmpeg publiées sont GPL — les reprendre
telles quelles changerait la licence du produit distribué.

**whisper.cpp** — build CMake, portable en principe. Le point ouvert est le
backend d'accélération.

## 3. Backend d'accélération — décision ouverte

| Backend | Portée matérielle | Coût |
|---|---|---|
| **CPU seul** | universel | simple, mais lent sur un modèle `large-v3` non quantisé |
| **Vulkan** | AMD, Intel, NVIDIA | un seul backend pour tout le parc — candidat le plus sérieux |
| **CUDA** | NVIDIA uniquement | rapide, mais exclut la moitié du parc et alourdit la distribution |
| **DirectML** | large | support whisper.cpp moins mûr |

**Recommandation : CPU comme socle correct, Vulkan comme accélération.** Ne pas
livrer un portage qui n'est utilisable que sur NVIDIA.

À mesurer avant de trancher : le temps réel du modèle de traduction `large-v3`
(3,1 Go, non quantisé) en CPU seul sur une machine de milieu de gamme. S'il est
inacceptable, l'accélération devient bloquante plutôt qu'optionnelle.

## 4. Chemins et système de fichiers

À reprendre systématiquement — c'est là que se cachent les bugs silencieux :

* **Répertoire de données** : `~/Library/Application Support/com.romainbourbon.stia`
  devient `%APPDATA%\com.romainbourbon.stia`. L'API Tauri le résout déjà ; ce
  qui compte est de vérifier qu'aucun chemin n'est construit à la main.
* **Séparateurs** : tout passage de chemin en `String` plutôt qu'en `PathBuf`
  est suspect.
* **Longueur maximale** : `MAX_PATH` (260) s'applique encore à beaucoup d'API.
  Un média au nom long dans un dossier profond peut échouer là où macOS passe.
* **Caractères interdits** : `\ / : * ? " < > |` sont refusés dans un nom de
  fichier Windows. La dérivation du nom de sortie à partir du nom du média doit
  les assainir.
* **Casse** : le système de fichiers Windows est insensible à la casse mais la
  préserve — comme APFS par défaut. Peu de surprises attendues.
* **Répertoire temporaire** : `%TEMP%`, et surtout **le nettoyage** — voir §6.

## 5. Intégration Explorer

`open_output_folder` révèle actuellement le fichier dans le Finder. L'équivalent
est `explorer.exe /select,"<chemin>"`.

Point de sécurité : la cible doit rester **dérivée de l'état backend**, jamais
d'une chaîne fournie par le frontend — c'est la propriété que le modèle de
menace tient déjà sur macOS, et elle doit être tenue à l'identique. Les
guillemets et la virgule de `/select,` demandent un échappement soigneux.

## 6. Processus, annulation et verrous de fichiers

C'est la différence de comportement la plus profonde entre les deux systèmes.

* **Annulation** : il n'y a pas de `SIGTERM`. L'arrêt propre d'un sidecar passe
  par `TerminateProcess` ou par les *job objects*. Le contrat actuel
  (annulation immédiate, nettoyage complet du répertoire de travail) doit être
  requalifié, pas supposé.
* **Verrous de fichiers** : Windows **refuse de supprimer un fichier ouvert**.
  Sur macOS, `unlink` sur un fichier encore ouvert par whisper-cli réussit. Tout
  le nettoyage (`cleanup.rs`) doit être rejoué : un `remove_dir_all` qui passe
  sur macOS peut échouer sur Windows si le sidecar n'est pas complètement
  terminé.
* **Processus orphelins** : si l'application est tuée, les sidecars doivent
  mourir avec elle. Les job objects sont le mécanisme correct.

## 7. Gestionnaire de modèles

Peu de changements attendus : `reqwest`/rustls est portable, la vérification
SHA-256 aussi. À vérifier :

* écriture atomique (`rename` sur le même volume) — sémantique Windows différente,
  `rename` échoue si la cible existe ;
* espace disque libre — `statvfs` (via `libc`) n'existe pas ; il faut
  `GetDiskFreeSpaceExW` ;
* Windows Defender peut analyser un fichier de 3,1 Go à la fermeture et
  retarder la promotion du fichier temporaire.

## 8. Tests

* Les tests `domain/` doivent passer tels quels — ils sont purs. **Si l'un échoue
  sur Windows, c'est un vrai bug de portabilité**, pas un test à ajuster.
* Les tests qui touchent le système de fichiers demandent des variantes.
* `shell_contract.rs`, `capability_surface.rs` et `csp_policy.rs` restent
  valables : ils décrivent l'architecture, pas la plateforme.
* Ajouter une matrice CI `windows-latest` — mais **ne l'ajouter que quand elle
  peut passer**, sinon elle rend le badge CI mensonger.

## 9. Packaging — plus tard

NSIS ou MSI via Tauri. Hors périmètre tant que l'application ne fonctionne pas.

**Signature Windows** : voir
[`../release/COMMUNITY_PUBLIC_READINESS.md`](../release/COMMUNITY_PUBLIC_READINESS.md).
Aucun certificat n'est configuré (`WINDOWS_CODE_SIGNING_NOT_CONFIGURED`). Sans
signature, SmartScreen affichera un avertissement — l'équivalent de Gatekeeper.
Cela ne bloque pas la publication du **source**.

## 10. Ce qu'il faudra qualifier humainement

Un portage n'est pas fini parce qu'il compile. Au minimum, sur une vraie
machine Windows :

* une transcription complète, de bout en bout ;
* une annulation en milieu de traitement, avec vérification qu'aucun fichier
  temporaire ne subsiste ;
* un média au nom contenant des caractères non-ASCII et des espaces ;
* le téléchargement des deux modèles ;
* la révélation du dossier de sortie dans Explorer ;
* le comportement au premier lancement sans WebView2.

---

## Ordre de travail suggéré

1. Compiler le cœur Rust pour `x86_64-pc-windows-msvc`, tests `domain/` au vert.
2. Produire les deux sidecars Windows, licence FFmpeg vérifiée.
3. Trancher le backend d'accélération, chiffres à l'appui.
4. Reprendre chemins, annulation, verrous, nettoyage.
5. Explorer, gestionnaire de modèles, espace disque.
6. CI Windows.
7. QA humaine.
8. Packaging et signature — seulement ensuite.

Les étapes 1 à 4 sont l'essentiel du risque. Le reste est du travail connu.
