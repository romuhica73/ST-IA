# Revue des références publiées — avant passage en public

Établie par M10 sur `9e55c65`. **Aucune branche n'a été supprimée.** Ce document
prépare une décision ; il ne l'applique pas.

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

| Branche | HEAD | Fusionnée dans `main` | Commits uniques | Risque | Recommandation |
|---|---|---|---|---|---|
| `feat/m0b-french-model-qualification` | `d2154cd` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m1-desktop-shell` | `ab8e806` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m2-local-transcription-pipeline` | `d54277e` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m3-model-manager` | `8b752be` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m4-runtime-robustness` | `610195f` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m5-macos-release-candidate` | `5241bdc` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m6-visual-polish-motion` | `5e1410b` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m7-settings-i18n-versioning` | `5fdf487` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m8-open-source-security-readiness` | `3e3fc2c` | oui | **0** | aucun | DELETE_BEFORE_PUBLICATION |
| `feat/m9-bilingual-splash-release-packaging` | `9e55c65` | oui — **HEAD identique à `main`** | **0** | aucun | DELETE_BEFORE_PUBLICATION |

Vérification appliquée à chacune :

```sh
git merge-base --is-ancestor <ref> origin/main   # exit 0 pour les 10
git rev-list --count origin/main..<ref>          # 0 pour les 10
```

**Aucun commit n'existe sur ces branches qui ne soit déjà dans `main`.** Les
supprimer ne perd rien — l'historique complet reste atteignable depuis `main`, et
les messages de commit comme les diffs restent intacts. Les garder ne présente
aucun risque de sécurité non plus : c'est un choix de lisibilité.

**Recommandation : supprimer après publication de `v0.1.0`**, pas avant. Tant
qu'aucun tag n'existe, ces branches sont le seul repère nommé des jalons du
projet. Une fois `v0.1.0` taguée, elles deviennent purement redondantes.

> Cette recommandation est plus prudente que « supprimer avant publication » :
> l'ordre importe peu puisque le risque est nul, et attendre le tag évite de
> perdre les seuls repères de milestone existants.

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

« REVIEW_REQUIRED » désigne ici une décision de **maintenance**, pas de sécurité :
fusionner ou fermer les PR correspondantes. Supprimer la branche sans fermer la
PR est inutile — Dependabot la recrée.

Cinq d'entre elles sont en retard de 31 commits sur `main`. `sha2-0.11.0` est à
jour et constitue une vraie décision de dépendance (changement de version
majeure).

> ⚠️ Une conséquence à connaître : **`gitleaks-action` v2 → v3 et les bumps
> d'actions modifient `security.yml`, que M10 vient également de modifier**
> (extraction du scan de secrets vers `secret-scan.yml`). Ces PR entreront donc
> en conflit et devront être rebasées ou refaites.

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

## Décision demandée à l'humain

- [ ] **Branches `feat/m*`** — supprimer les 10 après le tag `v0.1.0`, ou les
      conserver ? *(recommandation : supprimer après le tag ; aucun impact
      sécurité dans les deux cas)*
- [ ] **PR Dependabot** — fusionner ou fermer les 6, en tenant compte du conflit
      annoncé sur `security.yml`
- [ ] Confirmer qu'aucune branche ne doit être supprimée **avant** la publication

Tant que ces cases ne sont pas cochées, **aucune branche ne doit être
supprimée** : M10 s'interdit toute suppression de référence.
