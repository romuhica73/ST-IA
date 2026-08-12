# M9 — Delta de sécurité

Statut : `FULL_DELTA_REVIEW_PENDING_M10`

Date : 2026-08-12

Ce document couvre **uniquement les surfaces nouvelles ou modifiées par M9**.
Il ne rejoue pas la [revue de sécurité M8](M8_SECURITY_REVIEW.md), qui reste
la baseline de référence, ni le [modèle de menace](THREAT_MODEL.md), inchangé.

## Résumé

M9 n'élargit **ni la CSP, ni les permissions de plugins, ni la surface
réseau**. Il ajoute une seconde fenêtre, deux commandes non privilégiées, une
seconde passe Whisper, un second modèle téléchargeable — et il **corrige une
faiblesse réelle du contrôle d'accès** décrite ci-dessous, qui préexistait à
M9 et affectait déjà M8.

### Finding — les commandes applicatives n'étaient soumises à aucun ACL

**Sévérité : MEDIUM. Statut : corrigé.**

La revue M8 et la première version de ce document affirmaient que la fenêtre
splash « ne peut invoquer aucune commande » parce que son label n'apparaissait
dans aucune capability. **C'était faux.**

Dans Tauri 2, le système de capabilities ne gouverne par défaut que les
commandes de *plugins*. Les commandes définies par l'application et
enregistrées via `invoke_handler` sont autorisées **dans toutes les fenêtres**,
quel que soit le contenu de `capabilities/`, sauf si l'application déclare
explicitement un manifeste ACL (`tauri_build::AppManifest::commands`).

Conséquence concrète avant correctif : n'importe quel script s'exécutant dans
la fenêtre splash pouvait appeler `start_transcription`, `install_model`,
`open_output_folder`, `save_settings` — l'intégralité de la surface
applicative. Les protections de fond tenaient toujours (validation du chemin
média, dérivation de la cible Finder depuis l'état backend, épinglage de
l'URL de modèle), donc l'impact réel était limité ; mais la **propriété
d'isolation annoncée n'existait pas**.

**Ce qui est effectivement appliqué désormais** — et non simplement
« capabilities minimales » :

> L'autorisation des commandes applicatives est **imposée côté Rust**, par un
> ACL de commandes explicite déclaré à la compilation. Chaque commande est
> attribuée nommément à des fenêtres nommées. **La fenêtre splash ne peut
> invoquer ni la transcription, ni l'installation de modèle, ni l'ouverture du
> Finder, ni l'écriture des réglages, ni aucune autre commande privilégiée** :
> l'appel est refusé par le runtime Tauri avant d'atteindre le code de la
> commande, indépendamment de ce que le JavaScript de cette fenêtre tente de
> faire.

Correctif : `src-tauri/build.rs` déclare désormais un manifeste ACL listant
les 14 commandes de l'application. Les fichiers de capabilities deviennent
autoritatifs pour elles aussi, et les deux fenêtres sont scindées :

* `capabilities/main.json` — fenêtre `main` : `core:default`, dialogue,
  sidecars nommés, `reveal-item-in-dir`, et les 12 autres commandes
  applicatives ;
* `capabilities/splash.json` — fenêtre `splashscreen` : **une seule**
  permission, `allow-notify-splash-finished`.

Huit tests d'intégration verrouillent l'ensemble, dont un qui vérifie que le
manifeste ACL existe (sans lui, tout le reste redevient décoratif) et un qui
vérifie que toute commande enregistrée y figure.

Ce point est signalé comme correction explicite d'une affirmation antérieure
inexacte, et non comme une nouveauté.

## Surfaces évaluées

### 1. Nouvelle fenêtre `splashscreen`

| Aspect | Constat |
| --- | --- |
| Capabilities | **Une seule** : `allow-notify-splash-finished`. |
| IPC | Cette commande et rien d'autre — refus par l'ACL applicatif pour tout le reste. |
| Événements | Impossible — `core:event:allow-listen` n'est pas accordé. |
| Sidecars | Aucun accès (`shell:allow-execute` reste limité à `main`). |
| Filesystem | Aucun accès. |
| Opener / Finder | Aucun accès. |
| Réseau | Aucun — aucune ressource distante, et la CSP interdit toute origine externe. |
| Données utilisateur | Aucune. Ni chemin, ni nom de fichier, ni transcription n'atteint cette fenêtre. |
| Contenu affiché | HTML/CSS/JS locaux embarqués dans le binaire. |

Verrouillé par huit tests d'intégration
(`src-tauri/tests/capability_surface.rs`) : le manifeste ACL doit exister,
toute commande enregistrée doit y figurer, toute capability doit être
explicitement rattachée à des fenêtres nommées sans joker, le splash doit
détenir exactement une permission et aucune permission privilégiée ou de
plugin, et la fenêtre principale doit conserver les siennes.

Ce dernier test importe : sans lui, vider `capabilities/` ferait passer tous
les autres.

### 2. Deux nouvelles commandes de bascule

`notify_ui_ready` (fenêtre `main`) et `notify_splash_finished` (fenêtre
`splashscreen`). Ce sont les deux moitiés du même handshake.

* **Entrée** : aucune, pour les deux. Aucun chemin, aucune chaîne libre, donc
  rien à valider au sens de la frontière IPC M8.
* **Sortie** : rien.
* **Effet** : enregistrent un signal ; la bascule n'a lieu que lorsque les
  deux sont arrivés, et au plus une fois (test-and-set).
* **Rejouabilité** : un appelant hostile qui les invoquerait en boucle
  n'obtiendrait rien après la première bascule. Le seul pouvoir conféré est
  d'écourter une animation de 6 s sur sa propre fenêtre.
* **Fuite d'information** : aucune — aucune valeur de retour.

Surface de commandes totale : **14** (11 en M8 + les deux signaux de bascule + `get_model_cards`), désormais toutes
soumises à l'ACL applicatif et attribuées par fenêtre.

### 2 ter. Commande `get_model_cards`

Retourne la description publique des deux modèles (identifiant, taille,
SHA-256, provenance, moteur) pour le panneau de transparence.

* **Entrée** : aucune.
* **Sortie** : des constantes compilées, identiques à celles que le
  gestionnaire de modèle applique — un test vérifie que la carte reproduit
  exactement le manifeste, pour qu'on ne puisse pas afficher une empreinte
  que l'application ne vérifie pas.
* **Fuite d'information** : aucune. Rien d'un système de fichiers, rien de
  l'utilisateur ; ces valeurs sont déjà publiques dans le dépôt.

### 2 bis. Seconde passe Whisper et second modèle

* **Invocation de processus** : inchangée dans sa nature — le même sidecar
  `whisper-cli`, lancé sans shell, avec des arguments construits en Rust à
  partir de valeurs typées. La seconde passe ajoute le drapeau `-tr` et un
  chemin de modèle différent ; aucune entrée utilisateur libre n'entre dans
  la ligne de commande.
* **Concurrence** : les passes s'exécutent dans une boucle unique, sans
  `spawn` ni `join`. Au plus **un** enfant `whisper-cli` à tout instant — la
  garantie M4 est préservée littéralement, et testée.
* **Modèle de traduction** : mêmes garanties de téléchargement que le modèle
  existant (HTTPS strict, URL épinglée sur le **même commit vérifié**,
  redirections bornées, plafond de taille en flux, SHA-256 vérifié, fichier
  temporaire, promotion atomique). Taille et empreinte pinnées :
  3 095 033 483 o / `64d182b440…d1e2`.
* **Nettoyage** : les deux fichiers `.download` sont désormais purgés au
  démarrage, pas seulement celui du modèle de transcription.
* **Aucun téléchargement silencieux** : le modèle de traduction n'est
  récupéré que sur clic explicite, et uniquement si la version anglaise est
  demandée.
* **Sorties** : les noms de fichiers dérivent du nom du média source par une
  fonction pure, testée contre les collisions ; aucune combinaison ne produit
  deux fichiers de même nom.

### 3. Préférences transmises dans le fragment de l'URL

`splash.html#theme=…&motion=…` est construit en Rust à partir de valeurs
d'énumérations, jamais d'entrée utilisateur libre. Les six valeurs possibles
sont des mots ASCII minuscules, ce qu'un test vérifie explicitement — une
valeur nécessitant un percent-encoding casserait silencieusement la lecture
côté TypeScript.

Aucune donnée personnelle ne transite : le thème et la réduction d'animations
sont des préférences d'affichage.

Le splash traite toute valeur inconnue comme « système » plutôt que comme une
erreur, y compris sur une URL modifiée à la main. Une fenêtre de démarrage
n'a pas d'état d'échec à proposer.

### 4. CSP

**Inchangée.** Aucune directive ajoutée, retirée ou élargie.

Le splash a été écrit pour s'y conformer : feuille de style externe, script
module externe, aucun style ni script inline, aucune ressource distante,
aucune police téléchargée.

`src-tauri/tests/csp_policy.rs` (7 tests) épingle désormais la politique
elle-même — présence, `default-src 'self'`, absence de `unsafe-inline` et
`unsafe-eval`, `connect-src` strictement égal à `'self' ipc:
http://ipc.localhost`, directives d'embarquement à `'none'` — et vérifie que
`splash.html` s'y conforme réellement. C'est un durcissement net par rapport
à M8, où la CSP était appliquée mais non testée : une régression ne se serait
manifestée qu'à l'exécution.

### 5. Nouvelle dépendance : `tokio`

Ajoutée en dépendance directe avec la seule feature `time`, pour les timers
non bloquants du splash (plancher d'affichage, chien de garde).

* Déjà présente transitivement — c'est le runtime asynchrone de Tauri
  lui-même. Aucune nouvelle arborescence n'entre dans le projet.
* Feature `time` uniquement : ni `net`, ni `fs`, ni `process`.
* Même raisonnement que `libc` en M4 : déclarée parce qu'elle est appelée
  directement.

Le `Cargo.lock` le confirme : le diff complet contre la baseline M8 est
**d'une seule ligne**, l'ajout de `tokio` à la liste des dépendances de
`st-ia`. Aucun nouveau crate n'entre dans l'arborescence.

```diff
@@ -3433,6 +3433,7 @@ dependencies = [
   "tauri-plugin-opener",
   "tauri-plugin-shell",
   "tempfile",
+  "tokio",
  ]
```

Côté frontend, **aucune dépendance ajoutée** : le splash n'utilise ni React,
ni i18next, ni l'API Tauri.

### 6. Script de packaging

`scripts/package-release.sh` n'est **pas** exécuté par l'application et ne
fait pas partie du produit distribué. C'est un outil de release local.

* Aucun secret manipulé, aucune variable d'environnement de credentials lue.
* Aucun accès réseau.
* Aucune signature, aucune notarisation, aucun accès au Keychain — hors
  périmètre M9, explicitement.
* Écrit exclusivement dans `release-artifacts/`, qu'il recrée à chaque
  exécution et qui est ignoré par Git.
* `set -euo pipefail`, et refus de packager si l'audit de contenu échoue.

Le script **réduit** un risque existant : il vérifie automatiquement qu'aucun
modèle, média de test, `.env`, log, matériel de signature ou source map ne se
trouve dans le bundle, contrôles qui n'étaient auparavant faits qu'à la main.

### 7. Fenêtre principale masquée au démarrage

`"visible": false` sur la fenêtre `main`. Aucune conséquence de sécurité : la
fenêtre existe, son contexte JavaScript s'exécute normalement, seule sa
présentation change. Les chemins de récupération (chien de garde 10 s,
destruction du splash) garantissent qu'elle finit toujours par être affichée.

## Contrôles M8 rejoués

Uniquement ceux que M9 pouvait affecter (§56).

| Contrôle | Résultat |
| --- | --- |
| CSP sur build empaquetée | Inchangée, désormais testée automatiquement |
| Surface des capabilities | 1 capability, rattachée à `main` seule ; splash sans aucune |
| Surface des commandes | 12 commandes, +1 (`notify_ui_ready`, booléen, sans retour) |
| Invocation de processus | Inchangée — `whisper-cli` et `ffmpeg` en sidecars, jamais de shell |
| Absence de shell | Confirmée — M9 n'ajoute aucun appel de processus |
| Modèles exclus des artefacts | Vérifié sur le `.app` et sur le `.dmg` monté — **aucun** des deux modèles n'est embarqué |
| Réseau au repos | **0 socket ouvert** sur 5 configurations de démarrage (`lsof -i` par PID) |
| Avis de dépendances | `pnpm audit` : 0 vulnérabilité. `cargo audit` : 0 vulnérabilité, 17 warnings *allowed* — identiques à M8 (bindings GTK jamais compilés sur macOS) |
| Historique Git | Non rejoué (§56) — M9 n'ajoute aucun fichier binaire ni secret |

Le contrôle réseau a été rejoué sur les cinq cas de démarrage de §35 —
réglages absents, thème sombre, motion réduite, fichier corrompu, fichier
vide — chacun donnant **zéro socket réseau** ouvert par le processus, et un
démarrage propre dans tous les cas (les fichiers invalides retombant sur les
valeurs par défaut, sans crash).

## Ce que M9 n'a pas touché

* le modèle et les arguments de la transcription française (mêmes fichiers
  produits, mêmes noms, même vitesse) ;
* le cycle de vie des jobs et l'annulation (M4/M5) — étendus à deux passes,
  mais sans changer les garanties ;
* la validation des chemins média à la frontière IPC (M8) ;
* le gestionnaire de modèle, son épinglage d'endpoint et sa vérification
  SHA-256 (M3/M8) ;
* le nettoyage des temporaires et ses garde-fous (M4/M8) ;
* la persistance des réglages (M7) ;
* `open_output_folder` et sa dérivation depuis l'état backend (M8).

## Confidentialité

Aucune régression. Le splash n'atteint pas le réseau, ne lit aucun fichier
utilisateur et n'affiche aucun contenu localisé — il ne peut donc pas non plus
révéler la langue de l'utilisateur avant le chargement des réglages.

**La traduction anglaise est intégralement locale.** Elle est produite par le
même binaire `whisper-cli` déjà embarqué, exécuté sur la même machine, à
partir du même WAV temporaire. Aucun texte, aucun média, aucun chemin ne
quitte le Mac. Le seul accès réseau de tout le produit reste le téléchargement
explicite d'un modèle, et il n'envoie qu'une requête GET.

## Réserves

* `FULL_DELTA_REVIEW_PENDING_M10` — ce document est un delta ciblé, pas une
  revue complète. M10 (signature, notarisation, publication) devra reprendre
  la revue globale avant toute distribution publique.
* `PUBLIC_DISTRIBUTION_SIGNING_PENDING` — les artefacts produits ne sont ni
  signés ni notariés. Ils ne doivent pas être publiés en l'état.
* Les réserves ouvertes en M8 restent ouvertes : revue juridique du mode de
  distribution de FFmpeg sous LGPL, et reproductibilité des sidecars binaires
  suivis dans Git.
