# Revue des références publiées — avant passage en public

Établie par M10 sur `9e55c65`, puis mise à jour après la réécriture de
l'historique et l'arbitrage du mode de publication. **Aucune branche n'a été
supprimée par cette mission.**

> ## Ce que l'arbitrage a changé
>
> La publication se fera depuis un **dépôt neuf** ; le dépôt actuel reste
> **privé définitivement** et devient l'archive de développement
> (`FRESH_REPOSITORY_ACCEPTED`).
>
> Cette revue garde toute sa valeur — elle a établi qu'aucune référence ne
> contenait de secret, de média privé ni de contenu inattendu — mais ses
> **recommandations de suppression deviennent sans objet** : des branches qui
> ne seront jamais publiées n'ont pas besoin d'être supprimées avant
> publication. Elles restent dans l'archive, qui est leur place.
>
> Seul `main` sera poussé vers le dépôt Community. Les branches `feat/m*` n'y
> seront pas recréées.
>
> **Les six branches Dependabot ont disparu d'elles-mêmes** : Dependabot les a
> fermées et supprimées quand la réécriture a rendu leurs PR non fusionnables.

## Pourquoi ce document existe

Rendre un dépôt GitHub public ne publie pas seulement `main` : **toutes les
branches distantes deviennent visibles**, avec leur contenu et leur historique.
Auditer `main` seul aurait laissé seize autres références non examinées.

## Résultat en une ligne

Aucune référence distante ne contient de secret, de média privé, de document
interne ou de code abandonné. **Aucune suppression n'est nécessaire pour des
raisons de sécurité.** Ce qui suit est de l'hygiène, pas une remédiation.

---

## 1. Branches de mission `feat/m*` — 10 branches

> Les SHA de ce tableau sont ceux **d'avant la réécriture d'historique**, état
> dans lequel l'audit a été mené. La correspondance ancien → nouveau est
> conservée dans la sauvegarde de la mission ; le contenu, lui, est identique
> (arbres inchangés).

| Branche | HEAD (avant réécriture) | Fusionnée dans `main` | Commits uniques | Risque | Recommandation |
|---|---|---|---|---|---|
| `feat/m0b-french-model-qualification` | `d2154cd` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m1-desktop-shell` | `ab8e806` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m2-local-transcription-pipeline` | `d54277e` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m3-model-manager` | `8b752be` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m4-runtime-robustness` | `610195f` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m5-macos-release-candidate` | `5241bdc` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m6-visual-polish-motion` | `5e1410b` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m7-settings-i18n-versioning` | `5fdf487` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m8-open-source-security-readiness` | `3e3fc2c` | oui | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |
| `feat/m9-bilingual-splash-release-packaging` | `9e55c65` | oui — **HEAD identique à `main`** | **0** | aucun | KEEP_IN_PRIVATE_ARCHIVE |

Vérification appliquée à chacune :

```sh
git merge-base --is-ancestor <ref> origin/main   # exit 0 pour les 10
git rev-list --count origin/main..<ref>          # 0 pour les 10
```

**Aucun commit n'existe sur ces branches qui ne soit déjà dans `main`.**

**Recommandation finale : les conserver dans l'archive privée.** L'arbitrage du
mode de publication a rendu la question de leur suppression sans objet — elles
ne seront jamais publiées, puisque seul `main` sera poussé vers le dépôt
Community. Dans une archive de développement, ces branches sont exactement ce
qu'on veut garder : le repère nommé de chaque jalon, avec sa PR.

## 2. Branches Dependabot — 6 branches

| Branche | HEAD | Commits uniques | Fichiers touchés | Recommandation |
|---|---|---|---|---|
| `dependabot/cargo/src-tauri/sha2-0.11.0` | `7e9849b` | 1 | `Cargo.lock`, `Cargo.toml` | REVIEW_REQUIRED |
| `dependabot/github_actions/actions/checkout-7` | `533b7e5` | 1 | `ci.yml`, `security.yml` | REVIEW_REQUIRED |
| `dependabot/github_actions/actions/setup-node-7` | `7679c53` | 1 | `ci.yml`, `security.yml` | REVIEW_REQUIRED |
| `dependabot/github_actions/gitleaks/gitleaks-action-3` | `89fc6df` | 1 | `security.yml` | REVIEW_REQUIRED |
| `dependabot/github_actions/pnpm/action-setup-6` | `5dfc0aa` | 1 | `ci.yml`, `security.yml` | REVIEW_REQUIRED |
| `dependabot/npm_and_yarn/dev-tooling-c3afe7b1d2` | `a115add` | 1 | `package.json`, `pnpm-lock.yaml` | REVIEW_REQUIRED |

Ces six branches ne touchent **que** des fichiers de dépendances et de CI. Aucun
média, aucun secret, aucun binaire nouveau. Elles sont publiables telles quelles.

> **Résolu depuis.** Ces six branches et leurs PR (#8 à #13) ont été **fermées
> et supprimées par Dependabot lui-même** lorsque la réécriture de l'historique
> a rendu leurs PR non fusionnables. Cette mission n'en a supprimé aucune. Le
> conflit annoncé ci-dessous s'est donc réglé de la manière prévue : Dependabot
> recréera les mises à jour encore pertinentes, contre la nouvelle base.

Le conflit anticipé était réel : `gitleaks-action` v2 → v3 et les bumps
d'actions modifient `security.yml`, que M10 a également modifié en extrayant le
scan de secrets vers `secret-scan.yml`.

## 3. Tags, notes et autres références

| Type | Commande | Résultat |
|---|---|---|
| Tags | `git tag --list` | **aucun** — pas de `v0.1.0`, conforme à l'état attendu |
| Notes Git | `git notes list` | aucune |
| Stash | `git stash list` | vide |
| Espaces de noms inhabituels | `git for-each-ref` complet | uniquement `refs/heads` et `refs/remotes/origin` |
| `.gitattributes` | — | absent (aucun LFS, aucun filtre) |

**Aucune ancienne release ni archive privée ne serait exposée par accident.**

## 4. Branches locales sans équivalent distant

| Branche | HEAD | En avance sur `main` | Analyse |
|---|---|---|---|
| `feat/m0-whisper-engine-spike` | `1392f8a` | 0 | jamais poussée ; contenu entièrement dans `main` |
| `feat/m10-community-public-release-readiness` | *(branche M10)* | — | branche de travail de cette mission |

Aucune branche locale ne détient de commit absent de `origin/main`. Rien ne
serait exposé ni perdu par surprise.

## 5. Objets non atteignables

`git fsck --unreachable` signale 13 blobs non atteignables **en local
uniquement**. Ils ne sont référencés par aucune ref et **ne sont jamais
transmis par `git push`** : ils n'apparaîtront pas sur GitHub. Aucune action.

---

## Décisions — closes

- [x] **Branches `feat/m*`** — **conservées** dans l'archive privée. La question
      « supprimer avant publication » ne se pose plus : elles ne seront pas
      publiées. Elles ont été réécrites et ne portent plus l'ancienne adresse.
- [x] **PR et branches Dependabot** — fermées et supprimées **par Dependabot
      lui-même** lors de la réécriture de la base. Aucune suppression manuelle.
- [x] **Aucune branche supprimée avant publication** — sans objet : la
      publication passe par un dépôt neuf, amorcé avec `main` seul.

M10 n'a supprimé aucune référence.
