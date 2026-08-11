# Contribuer à ST-IA

*English speakers: contributions and issues in English are welcome. This guide is in
French, matching the rest of the project's documentation.*

Merci de votre intérêt. ST-IA est un projet volontairement petit, avec un périmètre
étroit et assumé. Avant d'écrire du code, lisez la section **Périmètre** — c'est ce qui
détermine si une contribution a une chance d'être fusionnée.

## Périmètre — les trois règles non négociables

Ces règles sont l'identité du produit, pas des préférences.

1. **Local-first.** Aucun média, chemin, transcription ou métadonnée ne quitte la
   machine. La seule sortie réseau autorisée est le téléchargement explicite du modèle,
   déclenché par un clic de l'utilisateur.
2. **Pas de télémétrie, pas d'analytics, pas de crash reporting, pas d'auto-update.**
   Aucun de ces éléments ne sera accepté, même optionnel, même opt-in, sans une
   discussion préalable en issue **et** une ADR.
3. **Pas de dépendance Python côté utilisateur, pas de cloud pour la transcription.**

Une PR qui ajoute un appel réseau, un SDK tiers ou une dépendance runtime lourde sans
discussion préalable sera refusée, quelle que soit sa qualité.

## Cible supportée

**macOS Apple Silicon (arm64) uniquement.** C'est la seule plateforme qualifiée.

Le code n'est pas hostile à la portabilité, mais Intel macOS, Windows et Linux ne sont
ni testés, ni construits, ni supportés. Une PR de portage devra apporter sa propre
qualification — voir [`docs/BUILDING.md`](docs/BUILDING.md).

## Mise en place

Prérequis et procédure complète : [`docs/BUILDING.md`](docs/BUILDING.md).

En résumé :

```sh
pnpm install
pnpm tauri dev
```

Au premier lancement, l'application demande le téléchargement du modèle (574 Mo).

## Avant d'ouvrir une PR

Tout doit passer :

```sh
pnpm build                        # tsc + vite
pnpm test                         # Vitest

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

La CI rejoue exactement ces commandes sur chaque PR.

## Style

Suivez le code existant plutôt qu'un guide abstrait. Concrètement :

* **Les commentaires expliquent le *pourquoi*, jamais le *quoi*.** Le codebase
  documente les décisions non évidentes et les pièges — pas les mécanismes lisibles
  dans la ligne d'en dessous. Si un commentaire paraphrase le code, supprimez-le.
* Rust : `rustfmt` par défaut, zéro warning clippy.
* TypeScript : mode strict, pas de `any`, pas de `@ts-ignore`.
* React : composants fonctionnels, contrôles accessibles réels (`<button>`,
  `role="radiogroup"`), **jamais** un `<div onClick>`.
* i18n : toute chaîne visible passe par les catalogues, avec une clé structurée
  (`settings.themeLabel`), jamais la phrase française comme clé. FR et EN doivent
  rester strictement à parité — un test le vérifie.

## Branches et commits

* Branchez depuis `main` : `feat/…`, `fix/…`, `docs/…`, `security/…`, `chore/…`.
* Commits atomiques, en anglais, à l'impératif : `fix: reject symlinks in temp cleanup`.
* Le corps du commit explique le raisonnement, pas la liste des fichiers touchés.
* Ne poussez jamais directement sur `main`.

## Décisions d'architecture

Un changement structurel (nouvelle dépendance significative, changement de moteur,
nouveau format de stockage, nouvelle frontière de sécurité) demande une ADR dans
[`docs/architecture/`](docs/architecture/). Ouvrez d'abord une issue pour en discuter.

## Sécurité

**Ne signalez jamais une vulnérabilité dans une issue ou une PR publique.**
Suivez [`SECURITY.md`](SECURITY.md).

Si votre contribution touche une frontière de sécurité, dites-le explicitement dans la
PR. Sont concernés :

* toute nouvelle `#[tauri::command]`, ou tout nouvel argument à une commande existante ;
* les capabilities dans `src-tauri/capabilities/default.json` ;
* la CSP dans `src-tauri/tauri.conf.json` ;
* tout ce qui construit un chemin, supprime un fichier ou lance un processus ;
* la vérification d'intégrité du modèle.

Le modèle de menace ([`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md))
traite le frontend comme **non de confiance**. Toute valeur qui franchit `invoke()` est
une entrée attaquant et doit être validée côté Rust — le fait que l'UI ne puisse
« pas » produire une valeur hostile n'est pas un argument.

## Sidecars et binaires

`src-tauri/binaries/` contient deux exécutables committés (FFmpeg, whisper-cli) —
voir [`docs/architecture/ADR-003`](docs/architecture/ADR-003-local-transcription-pipeline.md).

**N'y committez jamais un binaire construit à la main.** Ils sont produits uniquement
par `scripts/build-ffmpeg-sidecar.sh` et `scripts/build-whisper-sidecar.sh`, qui
épinglent la version source et vérifient le checksum. Une mise à jour de sidecar est
une PR dédiée, avec dans sa description : la version, le SHA-256 du binaire, la sortie
de `otool -L`, et la raison du changement.

## Ce qui aide le plus

* Rapports de bug avec un média reproductible **anonymisé**.
* Corrections d'accessibilité et de contraste.
* Améliorations des traductions FR/EN.
* Tests, en particulier adversariaux (chemins, noms de fichiers, états corrompus).
* Documentation.
