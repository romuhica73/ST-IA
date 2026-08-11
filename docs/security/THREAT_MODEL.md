# ST-IA — Threat model

Statut : `ACCEPTED` (Mission 8)
Périmètre : ST-IA 0.1.0, macOS Apple Silicon, MVP local.

Ce document décrit ce que ST-IA protège, contre qui, et **ce qu'elle ne protège pas**.
Les propriétés listées en §4 sont celles qui sont réellement implémentées et vérifiables
dans le code à ce commit — pas des intentions. Chaque propriété renvoie au fichier qui la
tient.

---

## 1. Assets

| Asset | Emplacement | Sensibilité |
|---|---|---|
| Média source de l'utilisateur | n'importe où sur le disque, choisi par l'utilisateur | **Élevée** — peut contenir de la parole privée, professionnelle, médicale, juridique |
| Transcription (contenu textuel) | temporaire `{TMPDIR}/ST-IA/<job>/`, puis `<média>-sous-titres/` | **Élevée** — c'est le contenu du média en clair |
| Fichiers SRT/TXT produits | `<dossier du média>/<nom>-sous-titres/` | Élevée (même contenu) |
| Modèle Whisper (574 Mo) | `~/Library/Application Support/com.romainbourbon.stia/models/` | Moyenne — intégrité critique, confidentialité nulle (fichier public) |
| Réglages | `~/Library/Application Support/com.romainbourbon.stia/settings.json` | Faible — trois énumérations, aucun secret |
| Workspace temporaire | `{TMPDIR}/ST-IA/<pid>-<nanos>/` | Élevée pendant la durée du job (contient le WAV et le transcript) |
| Sidecars FFmpeg / whisper-cli | `src-tauri/binaries/`, puis `ST-IA.app/Contents/MacOS/` | **Intégrité critique** — exécutables committés dans le dépôt |
| Artefacts de build (`.app`, `.dmg`) | `src-tauri/target/` | Intégrité critique à la distribution (hors périmètre M8, voir M9) |
| Dépôt source public | GitHub | Confidentialité — ne doit contenir ni secret ni donnée personnelle |

**Le média et la transcription sont les assets primaires.** Tout le reste n'a d'importance
que dans la mesure où il permet de les atteindre.

---

## 2. Trust boundaries

```text
┌─────────────────────────────────────────────────────────┐
│ WebView (React/TS)                    ← NON DE CONFIANCE│
│   pas de réseau, pas de fs, pas de process              │
└───────────────────────┬─────────────────────────────────┘
                        │  ❶ IPC Tauri (invoke) — FRONTIÈRE PRINCIPALE
┌───────────────────────▼─────────────────────────────────┐
│ Backend Rust (11 commands)              ← DE CONFIANCE  │
│   seul détenteur du fs, des process et du réseau        │
└──┬──────────────┬───────────────┬───────────────┬───────┘
   │ ❷ argv       │ ❸ argv        │ ❹ fs          │ ❺ HTTPS
┌──▼─────┐  ┌─────▼───────┐  ┌────▼──────┐  ┌─────▼────────┐
│ FFmpeg │  │ whisper-cli │  │ filesystem│  │ Hugging Face │
│(file:  │  │  (local)    │  │  local    │  │ (modèle seul)│
│ seul)  │  │             │  │           │  │              │
└────────┘  └─────────────┘  └───────────┘  └──────────────┘
```

**❶ est la frontière qui compte.** Le frontend est traité comme hostile : tout ce qui
franchit `invoke()` est une entrée attaquant. C'est l'hypothèse de travail de tout le
§14 de l'audit M8.

❷/❸ : les sidecars sont lancés via `tauri_plugin_shell::sidecar()` avec un **vecteur
argv**, jamais une chaîne de commande. Il n'y a pas de shell dans la chaîne, donc pas
d'interprétation de `;`, `&&`, `$(...)`, backticks ou glob.

❺ : unique sortie réseau de l'application. Voir §4.

---

## 3. Attaquants et entrées

| # | Attaquant / entrée | Capacité supposée | Traité par |
|---|---|---|---|
| A1 | **Média malveillant** (fichier conçu pour exploiter un décodeur) | l'utilisateur ouvre un fichier piégé | Surface FFmpeg réduite au strict nécessaire (§4.6). Résiduel accepté. |
| A2 | **Nom de fichier / chemin hostile** (`;`, `$()`, `../`, unicode, très long) | l'utilisateur ou le frontend fournit un nom exotique | argv sans shell + `validate_media_path` |
| A3 | **Frontend compromis / XSS** | exécution JS arbitraire dans la WebView | **modèle d'attaque principal** — voir §5 |
| A4 | **Réglages corrompus** (fichier édité, tronqué, hostile) | accès en écriture au fichier de réglages | `Settings::parse` → défauts en bloc |
| A5 | **Modèle téléchargé compromis** (endpoint hostile, MITM, mirror) | contrôle de la réponse HTTP | SHA-256 + taille épinglés, plafond de taille |
| A6 | **Dépendance compromise** (npm ou crate) | code arbitraire au build ou au runtime | lockfiles committés, audits, Dependabot |
| A7 | **Clone/build hostile du dépôt** | un tiers construit ST-IA depuis les sources | sources pinnées + checksums dans les scripts de build |
| A8 | **Artefact de build compromis** (`.app` modifié après build) | substitution du binaire distribué | **hors périmètre M8** — signature/notarisation = M9 |

### Hors périmètre (explicitement)

* Un attaquant qui a déjà l'exécution de code **natif** sous le compte de l'utilisateur.
  Il a déjà accès direct aux médias ; ST-IA ne peut rien y ajouter.
* Un macOS compromis, un noyau compromis, un attaquant physique.
* La distribution binaire signée/notarisée (M9).
* Le multi-utilisateur : ST-IA est une application mono-utilisateur de bureau.

---

## 4. Propriétés de sécurité réellement tenues

Chacune est vérifiable dans le code à ce commit.

**4.1 — Les médias ne quittent jamais la machine.**
Aucun `fetch`, `XMLHttpRequest` ou `WebSocket` dans le frontend (0 occurrence).
Une seule sortie réseau existe dans tout le codebase : `reqwest` dans
`src-tauri/src/model.rs`, vers l'URL du modèle uniquement. Le média, son chemin et la
transcription ne sont jamais passés à cette fonction.

**4.2 — Le sidecar FFmpeg est physiquement incapable d'accéder au réseau.**
Construit avec `--disable-network --disable-everything --enable-protocol=file`. Vérifié
sur le binaire committé : `ffmpeg -protocols` ne liste que `file` en entrée et en sortie.
`-i http://…` et `-i concat:…` échouent avec `Protocol not found`. Ce n'est pas une
politique, c'est une absence de code.

**4.3 — Aucun shell arbitraire n'est atteignable depuis le frontend.**
La capability n'accorde que `shell:allow-execute` restreint à deux sidecars **nommés**
(`whisper-cli`, `ffmpeg`), jamais `shell:allow-spawn` ni une permission générique.
Les arguments sont construits côté Rust (`build_ffmpeg_args`, `build_whisper_args`),
pas fournis par le frontend.

**4.4 — Les chemins fournis par le frontend sont bornés.**
`start_transcription` re-valide `media_path` via `validate_media_path` : fichier régulier
existant, non vide, lisible, extension parmi six. `open_output_folder` ne prend **aucun
paramètre** : la cible est dérivée de l'état `Completed` du backend.

**4.5 — Les suppressions ne peuvent pas sortir des répertoires ST-IA.**
`clean_stale_job_dirs` ne lit que les enfants directs de `{TMPDIR}/ST-IA/`, exige une
entrée qui n'est pas un lien symbolique (`symlink_metadata`), exige un répertoire réel, et
exige un nom `<pid>-<nanos>` en chiffres ASCII. `prune_empty_legacy_dirs` utilise
`remove_dir` (non récursif, échoue si non vide).

> Le contrôle de nom n'est pas décoratif : WKWebView crée son propre répertoire
> `WebKit/` **dans ce même `{TMPDIR}/ST-IA/`** (il se namespace par nom de processus).
> Observé sur une build release empaquetée, où le log confirme
> `skipping unrecognized entry WebKit`. Sans ce contrôle, le nettoyage au démarrage
> supprimerait le répertoire de travail de la WebView.

**4.6 — La surface de décodage est réduite au minimum fonctionnel.**
5 demuxers, 9 décodeurs, 3 parsers, 1 muxer, 1 encodeur, 3 filtres. Tout le reste de
FFmpeg est absent du binaire. Un CVE dans un décodeur non compilé n'est pas atteignable.

**4.7 — Le modèle n'est jamais `ready` avant validation cryptographique.**
Téléchargement vers `<nom>.download`, vérification taille **et** SHA-256, puis `rename`
atomique. URL épinglée à un commit immuable de Hugging Face (pas `main`). Client HTTPS
strict (`https_only`), redirections bornées, plafond de taille pendant le stream.

**4.8 — Les sorties sont atomiques et ne peuvent pas écraser des données existantes.**
`resolve_output_dir` n'émet qu'un chemin qui n'existe pas (`-2`, `-3`, …). En cas d'échec
de copie, le dossier fraîchement créé est supprimé — jamais de résultat partiel qui
ressemble à un succès.

**4.9 — Aucun secret n'existe dans le dépôt.**
ST-IA n'a ni compte, ni API key, ni backend, ni télémétrie. Il n'y a rien à fuiter.
Vérifié : `gitleaks` sur les 43 commits de l'historique → 0 finding.

**4.10 — Le contenu injecté est affiché comme texte, jamais comme HTML.**
0 occurrence de `dangerouslySetInnerHTML`, `innerHTML`, `eval`, `new Function`. Les noms
de fichiers, transcriptions et messages d'erreur passent par le rendu React, qui échappe.
`escapeValue: false` dans i18next est sans effet ici : aucun `<Trans>` n'est utilisé.

**4.11 — CSP restrictive appliquée.**
`default-src 'self'` avec `object-src`/`frame-src`/`worker-src`/`media-src 'none'`,
`frame-ancestors 'none'`, `form-action 'none'`. Ni `unsafe-inline` ni `unsafe-eval` :
le build Vite n'émet aucun script ni style inline, et le code n'utilise aucun
`style={{…}}`.

**4.12 — Les réglages ne peuvent pas devenir un vecteur.**
Trois énumérations fermées (`serde` rejette toute valeur inconnue). Aucun champ n'est un
chemin, une URL ou une commande. Un fichier corrompu retombe intégralement sur les
valeurs par défaut.

---

## 5. Le scénario central : « et si le frontend est compromis ? »

C'est la question que pose §14 de la mission, et la seule qui rende l'IPC intéressante.
Après M8, un attaquant disposant d'exécution JS arbitraire dans la WebView peut :

| Il peut | Impact |
|---|---|
| Lire les réglages, la version de l'app, le manifeste du modèle | Négligeable |
| Écrire les réglages (3 énumérations) | Négligeable |
| Déclencher le téléchargement du modèle | Consommation réseau/disque bornée |
| Lancer/annuler une transcription **sur un fichier média valide existant** | Consommation CPU ; lecture d'un média que l'utilisateur possède déjà |
| Récupérer la transcription du job qu'il a lancé | **Divulgation** : le contenu d'un média audio/vidéo lisible par l'utilisateur |
| Révéler le dossier de sortie du dernier job dans le Finder | Négligeable |

Il **ne peut pas** : exécuter une commande arbitraire, lire un fichier arbitraire (le
whitelist d'extensions et les décodeurs disponibles bornent à des médias réels),
écrire dans un répertoire arbitraire, atteindre le réseau, ni exfiltrer quoi que ce soit
(CSP `connect-src 'self' ipc:` + aucun protocole réseau dans FFmpeg).

**Risque résiduel assumé :** un frontend compromis peut transcrire un média que
l'utilisateur possède et en lire le texte, sans pouvoir le sortir de la machine. Le
supprimer demanderait une confirmation native par job, ce qui détruirait l'ergonomie
du produit pour un gain nul tant qu'aucun canal d'exfiltration n'existe.

Rappel : ST-IA ne charge **aucun contenu distant**. Il n'y a pas de vecteur XSS connu ;
cette section décrit la défense en profondeur si un jour il y en avait un (dépendance
npm compromise, par exemple — A6).

---

## 6. Risques résiduels connus

| # | Risque | Sévérité | Pourquoi il est accepté |
|---|---|---|---|
| R1 | Un média piégé exploite un décodeur FFmpeg | Moyenne | Surface minimale (§4.6), FFmpeg 9.0 pinné, mais un décodeur reste du parsing de format complexe. FFmpeg tourne dans un processus séparé, sans réseau. Pas de sandbox dédiée. |
| R2 | Sidecars committés en binaire | Moyenne | Un contributeur ne peut pas trivialement vérifier que le binaire correspond à la source. Mitigé par des scripts de build déterministes et pinnés. Voir `M8_SECURITY_REVIEW.md` STIA-SEC-011. |
| R3 | Pas de signature/notarisation | Moyenne | M9. Le `.app` n'est pas distribué à ce stade. |
| R4 | Transcription lisible par un frontend compromis | Faible | Voir §5. Pas de canal d'exfiltration. |
| R5 | Chemin de développeur embarqué dans le binaire FFmpeg | Informationnel | `--prefix` du build apparaît dans `ffmpeg -version`. Corrigé au prochain rebuild du sidecar. |
| R6 | Les sorties sont écrites à côté du média source | Faible (par conception) | Choix produit assumé (ADR-003). L'utilisateur choisit implicitement le répertoire en choisissant son média. |

---

## 7. Ce que ce modèle ne couvre pas

* La chaîne de distribution binaire (M9 : signature, notarisation, checksums publiés).
* Une revue formelle de la sécurité mémoire des sidecars C/C++.
* Les paramètres GitHub du dépôt (recommandés dans
  `docs/release/GITHUB_PUBLICATION_CHECKLIST.md`, non appliqués par M8).
