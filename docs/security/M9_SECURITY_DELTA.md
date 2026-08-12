# M9 — Delta de sécurité

Statut : `FULL_DELTA_REVIEW_PENDING_M10`

Date : 2026-08-12

Ce document couvre **uniquement les surfaces nouvelles ou modifiées par M9**.
Il ne rejoue pas la [revue de sécurité M8](M8_SECURITY_REVIEW.md), qui reste
la baseline de référence, ni le [modèle de menace](THREAT_MODEL.md), inchangé.

## Résumé

M9 n'a introduit **aucune nouvelle commande privilégiée, aucune nouvelle
permission, aucune nouvelle dépendance réseau et aucun élargissement de la
CSP**. La seule surface réellement nouvelle est une seconde fenêtre, créée
délibérément sans aucune capability.

Trois des surfaces prévues au périmètre M9 n'existent pas : l'option de
traduction, la seconde passe Whisper et la sélection de langue de sortie
n'ont pas été implémentées (voir
[ADR-008](../architecture/ADR-008-bilingual-output-pipeline.md)). Le pipeline
de transcription est **strictement inchangé** par rapport à M8 — aucun
fichier de `src-tauri/src/pipeline.rs` ni de
`src-tauri/src/domain/transcription.rs` n'a été modifié.

## Surfaces évaluées

### 1. Nouvelle fenêtre `splashscreen`

| Aspect | Constat |
| --- | --- |
| Capabilities | **Aucune.** Le label n'apparaît dans aucun fichier de `capabilities/`. |
| IPC | Impossible — sans capability, tout `invoke` est refusé. |
| Événements | Impossible — `core:event:allow-listen` n'est pas accordé. |
| Sidecars | Aucun accès (`shell:allow-execute` reste limité à `main`). |
| Filesystem | Aucun accès. |
| Opener / Finder | Aucun accès. |
| Réseau | Aucun — aucune ressource distante, et la CSP interdit toute origine externe. |
| Données utilisateur | Aucune. Ni chemin, ni nom de fichier, ni transcription n'atteint cette fenêtre. |
| Contenu affiché | HTML/CSS/JS locaux embarqués dans le binaire. |

Verrouillé par trois tests d'intégration
(`src-tauri/tests/capability_surface.rs`) : toute capability doit être
explicitement rattachée à des fenêtres nommées sans joker, aucune ne doit
citer le splash, et la fenêtre principale doit continuer d'en détenir une.

Le troisième test importe : sans lui, supprimer tout le contenu de
`capabilities/` ferait passer les deux premiers.

### 2. Nouvelle commande `notify_ui_ready`

Unique ajout à la surface IPC. Exposée à la fenêtre principale, qui détient
déjà les capabilities de l'application.

* **Entrée** : un booléen (`reduced_motion`). Aucun chemin, aucune chaîne
  libre, donc rien à valider au sens de la frontière IPC M8.
* **Sortie** : rien.
* **Effet** : affiche la fenêtre principale et ferme le splash, au plus une
  fois (test-and-set).
* **Rejouabilité** : un appelant hostile qui l'invoquerait en boucle
  n'obtiendrait rien après le premier appel, et son seul pouvoir serait
  d'écourter une animation de 820 ms sur sa propre fenêtre.
* **Fuite d'information** : aucune — la commande ne retourne aucune valeur.

Surface de commandes totale : **12** (11 en M8 + celle-ci).

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
| Modèle exclu des artefacts | Vérifié sur le `.app` et sur le `.dmg` monté |
| Réseau au repos | **0 socket ouvert** sur 5 configurations de démarrage (`lsof -i` par PID) |
| Avis de dépendances | `pnpm audit` : 0 vulnérabilité. `cargo audit` : 0 vulnérabilité, 17 warnings *allowed* — identiques à M8 (bindings GTK jamais compilés sur macOS) |
| Historique Git | Non rejoué (§56) — M9 n'ajoute aucun fichier binaire ni secret |

Le contrôle réseau a été rejoué sur les cinq cas de démarrage de §35 —
réglages absents, thème sombre, motion réduite, fichier corrompu, fichier
vide — chacun donnant **zéro socket réseau** ouvert par le processus, et un
démarrage propre dans tous les cas (les fichiers invalides retombant sur les
valeurs par défaut, sans crash).

## Ce que M9 n'a pas touché

* le pipeline de transcription et son cycle de vie (M4/M5) ;
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

La traduction anglaise ayant été abandonnée, la question « la traduction
est-elle bien locale ? » est sans objet en M9 : il n'y a pas de traduction.

## Réserves

* `FULL_DELTA_REVIEW_PENDING_M10` — ce document est un delta ciblé, pas une
  revue complète. M10 (signature, notarisation, publication) devra reprendre
  la revue globale avant toute distribution publique.
* `PUBLIC_DISTRIBUTION_SIGNING_PENDING` — les artefacts produits ne sont ni
  signés ni notariés. Ils ne doivent pas être publiés en l'état.
* Les réserves ouvertes en M8 restent ouvertes : revue juridique du mode de
  distribution de FFmpeg sous LGPL, et reproductibilité des sidecars binaires
  suivis dans Git.
