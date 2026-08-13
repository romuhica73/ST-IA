# M10 — Revue de sécurité pour publication publique

Statut : `SOURCE_PUBLICATION_UNBLOCKED` — aucun finding Critical, High ou
Medium, et tous les gates humains fermés.

Date : 2026-08-13
Base auditée : `9e55c65` (release candidate 0.1.0)
Périmètre : delta M8 → M9 **complet**, plus l'historique Git entier et
**toutes les références distantes**.

Ce document **ferme le gate `FULL_DELTA_REVIEW_PENDING_M10`** laissé
volontairement ouvert par le [delta M9](M9_SECURITY_DELTA.md). Il complète la
[revue M8](M8_SECURITY_REVIEW.md) et le [modèle de menace](THREAT_MODEL.md),
qui restent la baseline.

Aucun secret n'est reproduit en clair dans ce rapport.

---

## Question posée

> Le dépôt peut-il devenir public sans exposer de secret, de donnée privée,
> d'artefact indésirable, de code commercial, de dépendance cachée ou de
> procédure de build non reproductible ?

## Réponse

**Oui.** Aucun finding ne bloque la publication du **source**. Les réserves qui
subsistent concernent la **distribution binaire officielle**, pas le dépôt.

Une distinction tenue tout au long de ce rapport :

| | Publication du source | Distribution binaire officielle |
|---|---|---|
| État | **débloquée** | **bloquée** (signature, notarisation, revue LGPL) |

---

## 1. Synthèse des findings

| Sévérité | Nombre |
|---|---|
| Critical | **0** |
| High | **0** |
| Medium | **0** |
| Low | **3** (M10-F01, F02, F08) |
| Hardening | **4** (M10-F03, F04, F05, F11) |
| Informational | **4** (M10-F06, F07, F09, F10) |

| ID | Titre | Sévérité | Statut | Bloque le source ? |
|---|---|---|---|---|
| M10-F01 | Garde de `install_model` en check-then-act, non atomique | Low | ouvert | non |
| M10-F02 | Aucune garde d'espace disque avant un téléchargement de 3,1 Go | Low | ouvert | non |
| M10-F03 | `core:default` accorde ~70 permissions là où 2 sont utilisées | Hardening | ouvert (hérité M8) | non |
| M10-F04 | `tokio` déclaré en dépendance directe sans aucun appelant | Hardening | **corrigé** | non |
| M10-F05 | `package-release.sh` : quoting fragile et audit de chemins trompeur | Hardening | ouvert | non |
| M10-F06 | Chemins de build développeur dans `whisper-cli` (M8 ne documentait que `ffmpeg`) | Informational | documenté | non |
| M10-F07 | `M9_SECURITY_DELTA.md` décrivait un état intermédiaire | Informational | **corrigé** | non |
| M10-F08 | Le scan de secrets ne tournait sur aucune PR de code | Low | **corrigé** | non |
| M10-F09 | Fichiers de permissions ACL périmés dans l'arbre développeur | Informational | ouvert (sans impact) | non |
| M10-F10 | `ADR-003` décrivait une capability opener qui n'existe plus | Informational | **corrigé** | non |
| M10-F11 | `shell:allow-execute` accordé à la WebView sans nécessité | Hardening | **corrigé** | non |

---

## 2. Le point central : l'ACL est réel, et il est exact

L'affirmation centrale de M9 était que les commandes applicatives sont
désormais bornées par un ACL Rust explicite. **Le mécanisme a été vérifié dans
le code de `tauri-build` lui-même, pas seulement dans les commentaires du
projet** : sans manifeste applicatif, les capabilities ne gouvernent que les
commandes de *plugins*, et les commandes de l'application sont autorisées
partout.

Réconciliation à trois voies, nom par nom :

| `lib.rs` (`invoke_handler`) | `build.rs` (manifeste ACL) | `capabilities/main.json` |
|---|---|---|
| **12 commandes** | **12 commandes** | **12 permissions de commande** |

**Correspondance exacte, aucun orphelin dans aucun sens.** `tauri-build` fait
échouer la compilation si `main.json` nomme une permission que le manifeste ne
définit pas : sur un checkout propre — ce que construit la CI — cette
correspondance est garantie par le compilateur, pas seulement par un test.

**Portée réelle, dite franchement.** Avec une seule fenêtre détenant les douze
permissions, l'ACL ne restreint **rien opérationnellement aujourd'hui**. Sa
valeur est structurelle : une future seconde fenêtre ne peut pas hériter de la
surface. C'est le test `only_the_single_application_window_is_granted_anything`
qui rend cette garantie porteuse. Le dire ainsi est plus utile que de
présenter l'ACL comme une protection active.

CSP vérifiée à `9e55c65` : ni `unsafe-inline`, ni `unsafe-eval`, `connect-src`
limité à `'self' ipc: http://ipc.localhost`, `object-src`/`frame-src`/
`worker-src`/`form-action` à `'none'`. **Aucun asset distant** n'est référencé
par le frontend (une seule occurrence de `http` : un espace de noms XML SVG
dans une URI `data:`, autorisée par `img-src 'self' data:`).

---

## 3. Findings

### M10-F01 — Garde de `install_model` en check-then-act

**Low** · `src-tauri/src/model.rs:152-236` · ouvert

Le verrou qui vérifie qu'aucun téléchargement n'est en cours est **relâché
avant** que l'état ne passe à `Downloading`. Deux appels concurrents peuvent
donc tous deux observer un état libre et poursuivre, en visant le même fichier
temporaire — que chacun supprime au démarrage.

*Impact.* Le résultat réaliste est l'échec des deux passes après avoir
transféré jusqu'à 2 × 3,1 Go, avec un verdict « corrompu » trompeur sur un
modèle qui ne l'est pas. Le pire cas théorique est que `rename` promeuve des
octets que le calcul de SHA-256 n'a jamais vus. Fortement atténué :
l'attaquant ne contrôle aucun octet (même URL épinglée, même empreinte
attendue), la fenêtre est de l'ordre de la microseconde, le bouton est
désactivé côté interface, et l'empreinte est recalculée au lancement suivant.
Atteignable uniquement depuis une WebView compromise, qui dispose déjà de toute
la surface de commandes.

*Remédiation.* Faire le test-and-set dans une seule section critique, comme le
fait déjà correctement `JobState::try_claim` dans `pipeline.rs`.

### M10-F02 — Aucune garde d'espace disque avant un téléchargement de 3,1 Go

**Low** · `src-tauri/src/model.rs:183-190` · ouvert

Le pipeline de transcription refuse de démarrer sur un disque plein. Le
gestionnaire de modèles n'a **pas** d'équivalent : il enchaîne directement sur
le téléchargement et ne s'arrête qu'au plafond de taille.

*Impact.* M8 avait borné le téléchargement à la taille attendue, dimensionnée
pour un modèle de 574 Mo. M9 a porté ce plafond à **3,1 Go** — un facteur 5,4
qui transforme le remplissage du disque d'un scénario hostile en accident
ordinaire. Aggravant : en cas d'échec en cours de transfert, le `.download`
partiel n'est pas supprimé sur le chemin d'erreur ; il survit jusqu'au
nettoyage du lancement suivant. L'utilisateur peut donc rester avec 3 Go
consommés et un badge « échec ». Auto-infligé et réversible, d'où Low.

*Remédiation.* Réutiliser `pipeline::available_bytes` avant le téléchargement,
et supprimer le fichier partiel sur le chemin d'échec.

### M10-F03 — `core:default` accorde bien plus que nécessaire

**Hardening** · `src-tauri/capabilities/main.json:7` · hérité de M8

`core:default` développe les permissions par défaut de neuf modules
(`app`, `event`, `image`, `menu`, `path`, `resources`, `tray`, `webview`,
`window`). Le frontend n'utilise en réalité que `invoke`, `listen`/`unlisten`
et l'écouteur de glisser-déposer, plus le dialogue d'ouverture de fichier.

*Impact.* Depuis une WebView compromise, les permissions excédentaires
permettent de remplacer le menu applicatif macOS ou de créer une icône de
barre de menus : usurpation d'interface, **aucun gain de privilège, aucun accès
à des données**.

Deux pistes plus graves ont été explorées puis **écartées, défense vérifiée** :
`core:image:allow-from-path` n'est pas une primitive de lecture de fichier
arbitraire ici (les features `image-ico`/`image-png` ne sont pas activées, la
commande se compile en stub qui renvoie une erreur), et `core:default`
n'accorde **aucune** permission de *création* de webview ou de fenêtre — donc
aucun canal d'exfiltration contournant la CSP.

### M10-F04 — `tokio` déclaré sans aucun appelant

**Hardening** · `src-tauri/Cargo.toml` · **corrigé par M10**

`tokio` est déclaré en dépendance directe avec ce commentaire :

> « Utilisé uniquement pour les timers non bloquants de la fenêtre splash
> (durée minimale d'affichage et chien de garde — voir `splash.rs`). »

Or `splash.rs` **n'existe plus** (supprimé par `9c0f865`), et aucune occurrence
de `tokio` ne subsiste dans `src/` ni dans `tests/`.

*Impact.* Aucun impact d'exécution : `tokio` est de toute façon présent comme
runtime de Tauri. Le problème est documentaire et il compte ici : c'est la
**seule dépendance ajoutée par M9**, sa justification est désormais fausse, et
elle est sur le point d'être lue par le public. Une dépendance directe
inutilisée survit aussi silencieusement au tri d'un futur `cargo audit` sous
prétexte qu'« on l'appelle directement », ce qui n'est plus vrai.

*Remédiation.* Retirer la ligne, ou corriger le commentaire. Envisager
`#![deny(unused_crate_dependencies)]` pour que cette dérive échoue en CI.

> **Appliqué.** La ligne a été retirée ; `Cargo.lock` change d'exactement une
> ligne, sans aucune mise à jour opportuniste de dépendance.

### M10-F05 — `package-release.sh` : audit de chemins trompeur

**Hardening** · `scripts/package-release.sh:113-115` · ouvert

Le script est par ailleurs solide (`set -euo pipefail`, chemins dérivés et
cités, aucun `curl | sh`, aucun accès au trousseau, contrôle croisé de version
sur trois manifestes). Deux défauts réels :

1. **Quoting.** `$APP_PATH` est interpolé dans une chaîne réanalysée par un
   `bash -c` imbriqué. Un chemin de checkout contenant une apostrophe casse le
   quoting — auto-infligé, mais c'est précisément le bloc censé inspirer
   confiance.
2. **Fausse assurance — le défaut le plus utile.** L'audit « aucun chemin
   développeur » n'inspecte que `Info.plist`, et passe. Or les binaires que le
   script vient de certifier « présents » **contiennent** des chemins de build
   (voir M10-F06). Un lecteur du rapport conclut raisonnablement qu'aucun
   chemin développeur n'est livré. Il y en a, dans deux binaires.

*Remédiation.* Étendre l'audit aux exécutables via `strings`, ou renommer le
contrôle pour qu'il dise exactement ce qu'il couvre.

### M10-F06 — Chemins de build dans `whisper-cli` aussi

**Informational** · documenté, non corrigé

STIA-SEC-104 (M8) ne documentait que le `--prefix` de FFmpeg. Le sidecar
whisper.cpp porte la même classe de chaînes, en plus grand nombre :

| Binaire | Occurrences de `/Volumes/Workspace` |
|---|---|
| `whisper-cli-aarch64-apple-darwin` | **21** (chemins `__FILE__` des assertions GGML) |
| `ffmpeg-aarch64-apple-darwin` | **3** (`--prefix`, `--datadir`) |

*Impact.* Catégorie B au sens de M8 : cela révèle une disposition de répertoire
de travail. **Aucun nom d'utilisateur, aucun répertoire personnel, aucun
secret** — vérifié, `/Users/` n'apparaît dans aucun des deux binaires, et
aucune adresse e-mail n'y figure. Les deux sont signés ad-hoc, sans identité
Apple ni team ID.

*Remédiation.* Neutraliser le préfixe (`-ffile-prefix-map`) au **prochain
rebuild légitime** des sidecars. Ne pas reconstruire « par sécurité » : cela
invaliderait la qualification octet-pour-octet de M2/M5 pour un gain nul.

### M10-F07 — Le delta M9 décrivait un état intermédiaire

**Informational** · **corrigé par M10**

`M9_SECURITY_DELTA.md` annonçait 14 commandes (12 en réalité), décrivait
`capabilities/splash.json` comme existant (le fichier a été supprimé), comptait
huit tests d'intégration (six), et attribuait le `+1` de surface à une commande
depuis retirée. Ces écarts sont le résidu du retrait de la fenêtre splash : la
section 1 du document annonce ce retrait, mais les sections rédigées avant
n'ont pas été réconciliées.

Cela compte davantage que pour une documentation ordinaire : ce sont les
artefacts de sécurité publics d'un projet dont l'argument est la vérifiabilité.
Un lecteur qui compare le document au code y perd confiance.

*Correction.* Un bloc de corrections a été ajouté en tête de
`M9_SECURITY_DELTA.md` — les erreurs sont **conservées et corrigées**, non
effacées, le document restant une trace historique. `THREAT_MODEL.md` (11 → 12
commandes) est corrigé. **Ce finding est le gate lui-même : le corriger le
ferme.**

### M10-F08 — Le scan de secrets ne tournait sur aucune PR de code

**Low** · **corrigé par M10**

Le job `gitleaks` vivait dans `security.yml`, dont le déclencheur
`pull_request` est filtré par `paths` sur les fichiers de dépendances. Le job
**héritait de ce filtre** : une PR touchant du code, des tests ou de la
documentation n'était jamais scannée. Seuls le cron hebdomadaire et le
déclenchement manuel couvraient ces cas — soit une fenêtre allant jusqu'à sept
jours.

*Correction.* Le scan a désormais sa propre workflow (`secret-scan.yml`), sans
filtre `paths`, sur `push` vers `main` et sur **toute** PR. Il reste un constat
*a posteriori* : le Push Protection natif de GitHub, qui bloque avant l'entrée
dans l'historique, demeure le complément indispensable (checklist de
publication).

### M10-F09 — Fichiers de permissions ACL périmés

**Informational** · sans impact

`src-tauri/permissions/autogenerated/` (ignoré par Git) contient 15 fichiers :
les 12 commandes actuelles plus trois commandes supprimées pendant M9.
`tauri-build` écrit les commandes courantes mais ne purge jamais.

*Impact.* **Aucun.** Une définition de permission n'est pas une autorisation :
`main.json` n'en accorde aucune et aucune de ces commandes n'est enregistrée.
Seule conséquence réelle : sur la machine du développeur, une définition
périmée pourrait masquer une incohérence `main.json` ↔ `build.rs` qu'un
checkout propre — donc la CI — fait correctement échouer.

### M10-F10 — `ADR-003` décrivait une capability opener supprimée

**Informational** · **corrigé par M10**

ADR-003 décrivait `opener:allow-open-path` avec un scope `$HOME/**` et
`/Volumes/**`. La capability réelle est `opener:allow-reveal-item-in-dir`
**seule**, sans scope — M8 (STIA-SEC-003) avait remplacé la première. Un
lecteur attentif à la sécurité aurait conclu à une surface **plus large** que
la réalité.

### M10-F11 — `shell:allow-execute` accordé à la WebView sans nécessité

**Hardening** · `src-tauri/capabilities/main.json` · **corrigé par M10**

`capabilities/main.json` accorde `shell:allow-execute` à la fenêtre, borné aux
deux sidecars nommés mais avec `"args": true`.

Or **cette permission n'est nécessaire à rien**. Les sidecars sont lancés
depuis Rust (`app.shell().sidecar(...)` dans `pipeline.rs`), et un appel
Rust-side ne traverse pas le système de capabilities — celui-ci ne gouverne que
les appels venant de la WebView. Le frontend, lui, n'a pas
`@tauri-apps/plugin-shell` dans ses dépendances et n'appelle jamais le shell :
les cinq seules commandes qu'il invoque sont des commandes applicatives.

*Impact.* Depuis une WebView compromise, la permission autorise l'exécution de
`ffmpeg` et `whisper-cli` avec des **arguments arbitraires**. FFmpeg est
construit sans réseau et sans composant GPL, mais il conserve le protocole
`file` : cela constitue une primitive de lecture de fichier et d'écriture WAV
plus large que la surface de commandes, qui elle est validée et contrainte.
Classé Hardening et non Low parce que le scénario suppose déjà une WebView
compromise (menace A3), position qui donne accès à toute la surface de
commandes — mais **retirer cette permission réduit strictement la surface sans
rien casser**, ce qui en fait un durcissement à coût nul.

*Remédiation.* Retirer le bloc `shell:allow-execute` de `main.json` et vérifier
qu'une transcription complète fonctionne toujours. Ajouter une assertion dans
`capability_surface.rs`. ADR-003 affirmait d'ailleurs déjà qu'« aucune
capability `shell:allow-execute` n'est exposée au frontend » — corriger la
configuration rendrait cette phrase vraie.

> **Appliqué et qualifié sur le `.app` empaqueté**, pas par raisonnement seul :
> transcription française et traduction anglaise du même média en un seul job
> (4 fichiers, sortie FR en français et EN en anglais), annulation en cours de
> passe tuant le sidecar sans publier de sortie partielle et sans laisser de
> répertoire de travail temporaire, relance sans redémarrage de l'application,
> et les deux modèles rapportés « Installé » par le panneau Modèles IA. L'argv
> du sidecar a été observé directement pendant l'exécution. Le test de
> capability affirme désormais qu'**aucune** permission `shell:` n'existe.

---

## 4. Pistes explorées et écartées

Une revue n'est digne de confiance que si elle dit aussi ce qu'elle **n'a pas**
trouvé, et pourquoi.

| Piste | Pourquoi elle est écartée |
|---|---|
| Injection d'arguments via le nouveau `-mc 0` | `-mc` et `0` sont des littéraux à la compilation. Aucune chaîne utilisateur n'atteint ces positions |
| Injection via un nom de fichier média hostile | La seule valeur contrôlée par l'utilisateur est l'opérande de `-i`. **Testé empiriquement** contre le sidecar épinglé avec un fichier nommé `-dash name.wav` : consommé comme valeur, jamais réinterprété comme option. Aucun shell dans la chaîne, et le chemin est validé en amont |
| Lecture de fichier arbitraire via `core:image:allow-from-path` | Features `image-ico`/`image-png` non activées : la commande se compile en stub renvoyant une erreur |
| Évasion de CSP par création d'une webview distante | `core:default` n'accorde **ni** `allow-create-webview` **ni** `allow-create-webview-window`. Aucune primitive de création, donc aucun canal d'exfiltration |
| `tauri-plugin-fs` dans l'arbre de dépendances | Présent comme dépendance de `plugin-dialog`, **jamais initialisé**, aucune permission `fs:` accordée, aucune commande `fs` enregistrée |
| Traversée de chemin via le nom du média en sortie | `file_stem` ne peut pas contenir de séparateur ; le nom de sortie est composé sans séparateur injectable |
| Suppression de données utilisateur préexistantes | Le dossier de sortie retourné n'existe jamais avant création ; seul ce dossier fraîchement créé est supprimé en cas d'échec |
| Fuite d'un résultat partiel à l'annulation d'un job bilingue | La publication a lieu une seule fois, après la dernière passe. Verrouillé par un test dédié |
| Deux processus `whisper-cli` concurrents | Les passes s'exécutent dans une boucle séquentielle sans `spawn`. La course spawn/register est fermée par une double vérification |
| Réglages comme vecteur de désérialisation | Trois énumérations fermées ; toute valeur inconnue fait retomber **tout** le fichier sur les valeurs par défaut. Aucun champ n'est un chemin, une URL ou une commande |
| Nettoyage supprimant des chemins non voulus | Gardes `symlink_metadata` + `is_dir` + motif de nom intactes, test adversarial sur lien symbolique présent |
| `open_output_folder` | Ne prend **aucun paramètre** : la cible est dérivée de l'état backend. Rien à valider |
| Durcissement du téléchargement du **second** modèle | Traitement identique : URL épinglée sur le même commit vérifié, HTTPS strict, redirections bornées, plafond de taille en flux, SHA-256 **et** taille exigés, écriture temporaire puis renommage atomique |

---

## 5. Scan de secrets

| Méthode | Périmètre | Résultat |
|---|---|---|
| `gitleaks` v8.30.1 | `--all --full-history`, 90 commits | **no leaks found** |
| `gitleaks dir` | par répertoire suivi (src, src-tauri, docs, scripts, .github, spike, licenses) | **0 finding** |
| Motifs à haute entropie | 535 blobs texte de tout l'historique — tokens `sk-`/`ghp_`/`github_pat_`/`hf_`/`AKIA`/`xox`/JWT, clés privées, certificats | **0 hit** |
| Types de fichiers | tous les fichiers jamais ajoutés, toutes refs | **aucun** `.p12`, `.pem`, `.cer`, `.key`, `.mobileprovision`, `.env` |
| Mots-clés | 20 blobs candidats, tous adjugés | **100 % faux positifs** |

Les faux positifs valent d'être nommés, parce qu'ils reviendront à chaque
scan : la revue M8 **énumère les motifs qu'elle recherche** (c'est de la
documentation de scan, pas des credentials) ; `media.rs` contient
`/etc/passwd` comme **fixture de test** vérifiant que le validateur le rejette,
et un fichier de test nommé `secret.txt` ; `.gitignore` mentionne
`secrets.json` dans une règle d'exclusion.

Un balayage de l'arbre de travail complet (400 Mo, répertoires ignorés inclus)
remonte 9 findings, **tous dans des artefacts tiers ignorés par Git** (sources
FFmpeg, clone whisper.cpp, métadonnées de crates Rust), **aucun suivi**.

**Le produit est architecturalement dépourvu de credentials** : ni compte, ni
clé d'API, ni backend. Il n'y a pas de secret à fuiter.

Les SHA-256 publiés dans la documentation sont des **valeurs d'intégrité de
modèles**, destinées à être publiques — pas des secrets.

## 6. Historique et références distantes

Rendre un dépôt public expose **toutes** ses branches. N'auditer que `main`
aurait laissé seize références non examinées.

| Élément | Résultat |
|---|---|
| Branches `feat/m*` (10) | toutes entièrement fusionnées, **0 commit unique** |
| Branches Dependabot (6) | 1 commit chacune, **uniquement** fichiers de dépendances et CI |
| Tags | **aucun** |
| Notes Git, stash, espaces de noms inhabituels | **aucun** |
| Branches locales sans distant | aucune ne détient de commit absent de `origin/main` |
| Objets non atteignables | 13 blobs, **locaux uniquement**, jamais transmis par `push` |

Détail et recommandations :
[`PUBLIC_REPOSITORY_REFS_REVIEW.md`](../release/PUBLIC_REPOSITORY_REFS_REVIEW.md).
**Aucune branche n'a été supprimée.**

## 7. Métadonnées d'auteur — remédiée

**État à l'audit** (avant remédiation) :

| Identité | Commits | Nature |
|---|---|---|
| `Romain Bourbon <…@…>` (adresse professionnelle, redacted) | **85** | domaine d'entreprise |
| `dependabot[bot]` | 6 | bot, adresse noreply |

Deux identités, aucune faute de frappe, aucune adresse périmée : les métadonnées
étaient propres. Le point n'était pas leur qualité mais leur **nature** — une
adresse professionnelle sur un domaine d'entreprise, qui serait devenue
définitivement publique dans chaque commit, chaque clone et chaque miroir, et
qui aurait rattaché publiquement ce projet personnel à cette organisation.

**Décision humaine, puis remédiation appliquée.** L'historique complet a été
réécrit avant publication : **90 commits**, `Author` et `Committer`, vers
l'adresse publique du projet. Seuls les commits portant exactement l'ancienne
adresse ont été touchés — les commits Dependabot conservent leur identité, comme
demandé.

| Contrôle après réécriture | Résultat |
|---|---|
| Identités sur les branches | `studio@romain-bourbon.com` **uniquement** |
| Ancienne adresse dans les métadonnées | **0** |
| Ancienne adresse dans le contenu de l'historique | **0** |
| Ancienne adresse dans les fichiers suivis | **0** |
| Arbres avant / après | **SHA identiques** — contenu inchangé |
| Noms, dates d'auteur et de commit | inchangés |
| Corps des commits | empreinte SHA-256 de l'ensemble **identique** |
| Topologie | 93 commits avant, 93 après |
| `gitleaks` sur l'historique réécrit | **no leaks found** |

Une **seconde passe** a été nécessaire, et mérite d'être notée : la réécriture
des métadonnées laissait l'adresse dans le **texte** des rapports M10 décrivant
ce finding. Les publier aurait republié exactement ce que la réécriture venait
de retirer.

**Ce que la réécriture n'atteint pas.** Un force-push ne supprime pas d'objets.
Vérifié explicitement plutôt que supposé : les quinze `refs/pull/N/head` du
dépôt pointent toujours vers les commits d'avant réécriture, et un
`git fetch origin <ancien-SHA>` **réussit encore**.

**Arbitrage : `FRESH_REPOSITORY_ACCEPTED`.** Le dépôt de développement reste
**privé définitivement** et devient l'archive du projet ; Community est publié
depuis un **dépôt neuf**, amorcé avec le seul `main` réécrit. Le résidu
disparaît par construction plutôt que par nettoyage, sans dépendre d'un tiers,
et le résultat est vérifiable avant publication plutôt qu'espéré après.

Détail :
[`COMMUNITY_PUBLIC_READINESS.md`](../release/COMMUNITY_PUBLIC_READINESS.md).

## 8. Dépendances

**Rust — une seule addition sur tout M9** : `tokio` (feature `time`), déjà
présent transitivement comme runtime de Tauri ; le diff de `Cargo.lock` fait
littéralement une ligne. **Sans appelant à `9e55c65`** — M10-F04.

**JavaScript — zéro addition.** Le diff de `package.json` et `pnpm-lock.yaml`
entre M8 et M9 est **vide**.

| Audit | Résultat |
|---|---|
| `pnpm audit --prod` | **0 vulnérabilité** |
| `pnpm audit` (tout) | **0 vulnérabilité** |
| `cargo audit` | 505 crates, **0 vulnérabilité**, 17 warnings tolérés |

Les 17 warnings sont des avis « unmaintained » / « unsound » sur des bindings
GTK3 (**jamais compilés sur macOS**), sur `unic-*` et sur `proc-macro-error`
(temps de build). Population **identique** à celle analysée par M8 : aucune
nouvelle classe d'avis. Déjà hors périmètre dans `SECURITY.md`.

## 9. Contenu du dépôt et sidecars

**202 fichiers suivis, 12,13 Mo.** `.git` pèse 14 Mo.

Aucun `.DS_Store`, log, crash dump, cache, `.app`, `.dmg` ni réglage local
suivi. `mockups/`, `test-media/`, `release-artifacts/`, `build-tmp/`,
`node_modules/`, `target/` et le clone whisper.cpp sont ignorés — **12 Go
d'arbre de travail ne contribuent que 12,13 Mo au dépôt**.

**Aucun modèle Whisper n'a jamais été committé**, vérifié par trois méthodes
indépendantes : aucun fichier de ce type dans tout ce qui a jamais été ajouté ;
le plus gros blob de tout l'historique pèse **3,4 Mo** ; `.git` fait 14 Mo, ce
qui exclut par construction un fichier de 574 Mo ou 3,1 Go.

**Sidecars suivis** (compromis assumé, STIA-SEC-106) :

| | `ffmpeg-aarch64-apple-darwin` | `whisper-cli-aarch64-apple-darwin` |
|---|---|---|
| Taille | 3 527 480 o | 3 275 928 o |
| SHA-256 | `87610d78…6a673d` | `a106f36d…8f3def3` |
| Architecture | Mach-O arm64, non-fat, statique | idem |
| Version | FFmpeg 9.0 | whisper.cpp 1.9.2 |
| Licence | LGPL-2.1-or-later | MIT |
| `otool -L` | **frameworks Apple uniquement** | idem |
| Signature | ad-hoc, sans identité Apple | idem |

Les deux empreintes sont **identiques à celles auditées par M8** : les binaires
n'ont pas changé. Aucun `@rpath`, aucune référence Homebrew — c'est ce qui rend
un clone propre constructible sur n'importe quel Mac Apple Silicon.

**Médias suivis : 3 fichiers**, tous des échantillons JFK du domaine public.

**Transcriptions suivies (`spike/out/`).** Lues intégralement. Deux corpus : les
sorties JFK (domaine public, risque nul) et les transcriptions d'un
enregistrement de l'auteur. Ce dernier est un **script de démonstration écrit
pour le test**, à propos du projet ST-IA lui-même. Il ne nomme aucun tiers,
aucun individu, et ne contient ni adresse, ni coordonnée, ni credential. Les
seuls faits personnels sont la configuration de la machine et des préférences
d'outillage. Les journaux sont **assainis** (chemins personnels remplacés par
des marqueurs). Le média source n'a jamais été committé. **Exposition faible et
auto-référentielle** — décision humaine, déjà tranchée en M8 (STIA-SEC-103).

## 10. Licences

| Composant | Version | Licence | Notices |
|---|---|---|---|
| ST-IA | 0.1.0 | **MIT** standard non modifié | `LICENSE` |
| whisper.cpp | v1.9.2, commit épinglé et **vérifié par le script de build** | MIT | texte complet présent |
| FFmpeg | 9.0, tarball officiel, SHA-256 vérifié | **LGPL-2.1**, construit `--disable-gpl --disable-nonfree --disable-version3` | texte complet présent |

Les deux textes de licence et `THIRD_PARTY_NOTICES.md` sont **embarqués dans le
`.app`**. La question GPL est tranchée par la configuration lue **dans le
binaire livré lui-même** : aucun composant GPL, aucune bibliothèque externe
liée.

**Réserve maintenue : `LEGAL_REVIEW_RECOMMENDED`** (STIA-SEC-202) sur
l'obligation de relink LGPL. La position documentée du projet est que le
sidecar est un **exécutable séparé**, remplaçable dans le bundle, et que la
source correspondante est fournie (version exacte, URL officielle, SHA-256,
script reproduisant la build). Le projet ne conclut pas lui-même et recommande
un avis juridique. **Ne bloque pas la publication du source** ; à trancher avant
toute distribution binaire officielle.

## 11. Absence de code commercial

Balayage sur `licensing`, `premium`, `subscription`, `billing`, `payment`,
`entitlement`, `paywall`, `trial`, `activation key` et apparentés, sur tout le
code.

**Aucune logique de licensing, facturation, compte, abonnement, gating
commercial, essai ou paiement n'existe dans le dépôt.** Tous les hits sont des
faux positifs : les champs de métadonnées MIT, le panneau « À propos » listant
les licences tierces, le nom de la mission M9 (« Premium Splashscreen »), et
des correspondances sur la sous-chaîne « ser**i**al » dans `Serialize`.

Le seul appel réseau sortant de tout le codebase est le `GET` de téléchargement
de modèle — sans corps, sans paramètre de requête, sans en-tête ajouté, et ni
le chemin du média ni la transcription ne sont dans la portée de cette
fonction.

[ADR-012](../architecture/ADR-012-community-commercial-boundary.md) **interdit
structurellement** d'introduire un tel code dans ce dépôt à l'avenir.

## 12. Build depuis un clone propre

Voir [`COMMUNITY_PUBLIC_READINESS.md`](../release/COMMUNITY_PUBLIC_READINESS.md)
pour le détail. Résumé : clone `origin` → `pnpm install --frozen-lockfile` →
71 tests frontend → `fmt` / `clippy -D warnings` → **155 tests Rust** →
`pnpm tauri build` → `.app` et `.dmg`. **Aucune dépendance cachée.** Aucun
modèle embarqué, aucune liaison Homebrew, aucun chemin développeur dans le
binaire de l'application.

Reproductibilité : **`FUNCTIONALLY_REPRODUCIBLE`**, pas `BIT_REPRODUCIBLE`.

---

## 13. Risques résiduels

| Risque | Sévérité | Position |
|---|---|---|
| Chemins de build dans les deux sidecars | LOW | Cosmétique. Corrigeable au prochain rebuild légitime |
| Sidecars binaires publiés dans Git | INFORMATIONAL | Compromis assumé (STIA-SEC-106) |
| Relink LGPL FFmpeg | `LEGAL_REVIEW_RECOMMENDED` | Ne bloque pas le source |
| Artefacts non signés / non notariés | `PUBLIC_DISTRIBUTION_SIGNING_PENDING` | Bloque la distribution binaire uniquement |
| Avis RustSec sur crates GTK3 | INFORMATIONAL | 0 vulnérabilité ; non compilées sur macOS |
| Voix de l'auteur publiée en transcription | LOW | Décision humaine |
| Métadonnées d'auteur sur domaine d'entreprise | **DÉCISION** | Gate humain requis |

## 14. Disposition

**Publication du source : DÉBLOQUÉE.** Aucun finding Critical, High ou Medium.
Rien dans le delta M9 n'expose de secret, de donnée privée ou de chemin de code
commercial.

Cinq findings ont été traités avant le commit de publication :

1. **M10-F07** — *fait*. Les documents de sécurité décrivent désormais le code.
2. **M10-F08** — *fait*. Scan de secrets sur toute PR.
3. **M10-F10** — *fait*. ADR-003 corrigé.
4. **M10-F04** — *fait*. `tokio` retiré ; `Cargo.lock` change d'une ligne.
5. **M10-F11** — *fait*. `shell:allow-execute` retiré de la capability et
   requalifié sur le `.app` empaqueté (FR, EN, annulation, relance,
   gestionnaire de modèles).

M10-F01, F02, F03, F05 et F09 sont sans danger à planifier après la
publication.

**Remédiation de l'historique.** Sur décision humaine du gate M10,
l'historique complet a été réécrit avant publication pour remplacer l'adresse
de commit professionnelle par l'adresse publique du projet. Arbres, noms,
dates, messages et topologie sont inchangés et vérifiés.

Un force-push ne supprimant pas les objets, le dépôt actuel conserve l'ancien
historique dans ses `refs/pull/*` — vérifié explicitement, un `git fetch` d'un
ancien SHA réussit encore. L'arbitrage humain a tranché
`FRESH_REPOSITORY_ACCEPTED` : **le dépôt actuel reste privé définitivement** et
devient l'archive de développement, la publication se faisant depuis un **dépôt
neuf** amorcé avec le seul `main` réécrit. Le résidu est ainsi supprimé par
construction plutôt que par nettoyage, et sans dépendre d'un tiers.

Détail et preuves :
[`COMMUNITY_PUBLIC_READINESS.md`](../release/COMMUNITY_PUBLIC_READINESS.md).

**Distribution binaire : toujours BLOQUÉE** par les réserves portées, inchangées
et correctement auto-déclarées par M9 : `PUBLIC_DISTRIBUTION_SIGNING_PENDING`,
`LEGAL_REVIEW_RECOMMENDED` (LGPL FFmpeg), `APPLE_DEVELOPER_ID_NOT_AVAILABLE` et
`WINDOWS_CODE_SIGNING_NOT_CONFIGURED`.

**Le gate `FULL_DELTA_REVIEW_PENDING_M10` est fermé.**
