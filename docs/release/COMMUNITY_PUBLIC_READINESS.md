# ST-IA Community — readiness pour publication publique

Établi par M10, mis à jour après remédiation de l'historique et arbitrage du
mode de publication. Remplace, pour la question « le code peut-il devenir
public ? », le rapport
[`OPEN_SOURCE_READINESS.md`](../security/OPEN_SOURCE_READINESS.md) de M8, qui
reste valable pour tout ce qu'il a établi.

**M10 n'a rendu aucun dépôt public, n'a créé aucun tag, n'a créé aucun dépôt et
n'a supprimé aucune branche.**

> **Deux dépôts, à partir de maintenant.** Le dépôt actuel
> `romuhica73/ST-IA` **reste privé définitivement** : il devient l'archive de
> développement. **ST-IA Community sera publié depuis un dépôt neuf**, créé en
> M11 à partir de l'historique réécrit. Voir §arbitrage.

---

## Question posée

> Le code ST-IA peut-il devenir public aujourd'hui sans exposer de secret, de
> donnée privée, d'artefact indésirable, de code commercial, de dépendance
> cachée ou de procédure de build non reproductible ?

## Réponse

**Oui.** Aucun blocage technique ne subsiste, et les décisions humaines du gate
M10 ont été prises et appliquées : frontière commerciale acceptée (ADR-012),
métadonnées d'auteur remédiées par réécriture d'historique, et les deux findings
de durcissement corrigés puis requalifiés.

Le dernier point ouvert — les références de pull request du dépôt actuel, qui
conservent l'ancien historique — est **tranché** : la publication se fera depuis
un **dépôt neuf**, ce qui supprime le résidu par construction plutôt que par
nettoyage. Le dépôt actuel reste privé définitivement.

**Aucun blocage ne subsiste.**

---

## Checklist

| # | Point | État | Preuve |
|---|---|---|---|
| 1 | **Licence MIT** | ✅ | `LICENSE` texte MIT standard non modifié, `Copyright (c) 2026 Romain Bourbon`. Champ `license` cohérent dans `package.json` et `Cargo.toml` |
| 2 | **Historique sûr** | ✅ | `gitleaks` sur `--all --full-history` (90 commits) : *no leaks found*. 3 balayages manuels indépendants : 0 hit |
| 3 | **Toutes les références auditées** | ✅ | 16 branches distantes + tags + notes + stash. Aucun contenu inattendu — [refs review](PUBLIC_REPOSITORY_REFS_REVIEW.md) |
| 4 | **Aucun secret** | ✅ | Aucun token, clé privée, certificat, `.env` sur aucune ref, à aucun moment. Le produit n'a ni compte, ni API key, ni backend |
| 5 | **Métadonnées d'auteur** | ✅ **remédiée** | Historique réécrit sur décision humaine : toutes les branches publiables portent `studio@romain-bourbon.com`. Arbres, dates, messages et topologie inchangés — §métadonnées |
| 6 | **README** | ✅ | Réécrit en anglais, factuel, sans badge mensonger, statut pré-release affiché |
| 7 | **SECURITY.md** | ✅ | Canal privé GitHub, périmètre, délais réalistes. Dépend de l'activation du *Private vulnerability reporting* |
| 8 | **CONTRIBUTING.md** | ✅ | Périmètre non négociable, style, sécurité. Référence ACL corrigée (`main.json`) |
| 9 | **BUILDING.md** | ✅ | Prérequis, versions de référence, tests, sidecars, dépannage. Aucun chemin développeur |
| 10 | **QUICKSTART.md** | ✅ | Inchangé, toujours exact |
| 11 | **AI_MODELS.md** | ✅ | Modèles, rôles, tailles, SHA-256, provenance épinglée, limites connues. Aucune revendication de conformité AI Act |
| 12 | **Composants tiers** | ✅ | `THIRD_PARTY_NOTICES.md` + textes de licence complets embarqués dans le `.app` |
| 13 | **Build depuis clone propre** | ✅ | Qualifiée de bout en bout — voir §build |
| 14 | **Modèles exclus** | ✅ | Aucun `.bin` dans Git ni dans l'historique. Aucun modèle embarqué dans le bundle |
| 15 | **Artefacts de release exclus** | ✅ | `/release-artifacts/` ignoré ; aucun `.dmg`/`.app` suivi |
| 16 | **Frontière commerciale** | ✅ | [ADR-012](../architecture/ADR-012-community-commercial-boundary.md). Aucun code de licensing/paiement/gating dans le dépôt |
| 17 | **Statut Windows honnête** | ✅ | `NOT_YET_SUPPORTED`, aucune date, [plan de portage](../platforms/WINDOWS_PORT_PLAN.md) |
| 18 | **Plan réglages GitHub** | ✅ | [checklist de publication](GITHUB_PUBLICATION_CHECKLIST.md) — préparée par M10, appliquée par M11 |

---

## Build depuis un clone propre — le gate majeur

Réalisé **depuis `origin`**, pas depuis l'arbre de travail, avec un `PATH`
volontairement restreint, en suivant `docs/BUILDING.md` à la lettre. Aucun
fichier n'a été copié depuis le dépôt de développement.

| Étape | Commande | Résultat |
|---|---|---|
| Clone | `git clone https://github.com/romuhica73/ST-IA.git` | ✅ baseline réécrite |
| Dépendances | `pnpm install --frozen-lockfile` | ✅ lockfile à jour, 104 paquets |
| Build frontend | `pnpm build` | ✅ tsc + vite |
| Tests frontend | `pnpm test` | ✅ **71 / 71** |
| Format Rust | `cargo fmt --check` | ✅ |
| Lint Rust | `cargo clippy --all-targets -- -D warnings` | ✅ 0 warning |
| Tests Rust | `cargo test` | ✅ **155 / 155** |
| Packaging | `pnpm tauri build` | ✅ `ST-IA.app` + `ST-IA_0.1.0_aarch64.dmg` (11,6 Mo) |

**Contrôles supplémentaires sur l'artefact produit :**

* **aucun modèle Whisper embarqué** dans le bundle — le premier lancement passe
  bien par le gestionnaire de modèles, comme prévu ;
* **aucune liaison Homebrew ni `/usr/local`** dans les trois exécutables du
  bundle (`otool -L`) — seuls des frameworks Apple ;
* **aucun chemin développeur dans le binaire de l'application** (`st-ia`).

**Dépendances cachées : aucune.** Aucun paquet Homebrew, fichier local, modèle
préexistant, cache personnel ni outil non documenté n'a été nécessaire. `cmake`
et `nasm` ne servent qu'à *reconstruire* les sidecars, ce que `BUILDING.md`
présente déjà comme rarement nécessaire.

### Reproductibilité

**`FUNCTIONALLY_REPRODUCIBLE`** — et non `BIT_REPRODUCIBLE`.

Un clone propre produit une application fonctionnellement identique, à partir
des seules sources publiées. Elle n'est pas *bit-for-bit* identique, et la
chaîne d'outils ne le garantit pas :

* horodatages de compilation dans les artefacts Rust et dans le `.app` ;
* métadonnées du bundler Tauri et de l'image disque DMG ;
* signature ad-hoc appliquée à la volée, différente à chaque build ;
* chemins absolus du répertoire de build présents dans les objets intermédiaires.

Exiger le bit-for-bit imposerait une chaîne d'outils dédiée
(`-ffile-prefix-map`, `SOURCE_DATE_EPOCH`, DMG déterministe) qui n'apporterait
rien tant qu'aucun binaire officiel n'est distribué. **À reconsidérer le jour
où une release signée est publiée** — c'est à ce moment que la reproductibilité
devient une propriété vérifiable par un tiers.

---

## Métadonnées d'auteur — remédiée

**Décision humaine du gate M10 : l'historique a été réécrit avant publication.**

Les commits portaient une adresse **professionnelle, sur un domaine
d'entreprise**, qui serait devenue définitivement publique dans chaque commit,
chaque clone, chaque miroir et chaque archive. L'auteur ne souhaitant pas
publier cette adresse, elle a été remplacée par l'adresse publique du projet :

```
Romain Bourbon <studio@romain-bourbon.com>
```

Cette adresse est vérifiée sur le compte GitHub de l'auteur et sélectionnée
comme adresse publique.

### Ce que la réécriture a fait

Seuls les commits dont l'`Author` **ou** le `Committer` portait exactement
l'ancienne adresse ont été réécrits. **Aucune autre identité n'a été touchée** —
les six commits Dependabot conservent leur auteur bot et leur committer GitHub.

Conservés à l'identique, et vérifiés :

| Propriété | Vérification |
|---|---|
| Arbres (contenu) | SHA d'arbre **identique** avant/après sur `main` et sur la branche M10 |
| Noms d'auteur | inchangés |
| Dates d'auteur et de commit | inchangées |
| Messages de commit | empreinte SHA-256 de l'ensemble des corps **identique** |
| Topologie | 93 commits avant, 93 après |

Le changement de SHA est la conséquence acceptée et attendue.

### Vérifications après réécriture

| Contrôle | Résultat |
|---|---|
| Ancienne adresse dans les métadonnées des branches | **0 occurrence** |
| Ancienne adresse dans les fichiers suivis | **0 occurrence** |
| Identités présentes sur les branches | `studio@romain-bourbon.com` uniquement |
| `gitleaks` sur l'historique réécrit | **no leaks found** |
| Clone propre + build + tests | rejoués intégralement |

### Ce que la réécriture n'a pas atteint — et pourquoi cela ne bloque plus

Il serait faux d'affirmer que l'ancienne adresse a disparu du dépôt actuel. Ce
qui suit a été **vérifié après le force-push**, pas supposé.

**1. Les références de pull request conservent l'ancien historique.** Les
quinze `refs/pull/N/head` pointent toujours vers les commits d'origine :

```
git ls-remote origin 'refs/pull/*/head'
→ refs/pull/14/head  9e55c65…   (ancien main)
→ refs/pull/1..13    anciens commits de jalons
```

**2. Les anciens objets restent récupérables par SHA :**

```
git fetch origin 9e55c65858ea637c3e828d0fea685f3d634b1f82
→ réussit
```

Un force-push ne supprime pas d'objets, et GitHub les conserve tant que son
ramasse-miettes ne les a pas collectés — sur un calendrier qu'il ne garantit
pas.

### Arbitrage tranché — dépôt neuf

**Décision humaine : `FRESH_REPOSITORY_ACCEPTED`.**

Le dépôt actuel `romuhica73/ST-IA` **reste privé définitivement** et devient
l'**archive de développement privée** du projet. Il ne sera jamais rendu
public.

**ST-IA Community sera publié depuis un dépôt neuf et indépendant**, créé en
M11 à partir de l'historique réécrit et propre.

Ce que cela résout, par construction plutôt que par nettoyage :

| Résidu | Sort dans un dépôt neuf |
|---|---|
| 15 `refs/pull/*/head` pointant vers l'ancien historique | **n'existent pas** — aucune PR n'a jamais été ouverte |
| Anciens objets récupérables par SHA | **n'existent pas** — jamais poussés |
| Ancienne adresse professionnelle | **absente** — seul l'historique réécrit est poussé |
| Dépendance à un tiers pour purger | **aucune** — rien à demander à GitHub Support |

Aucune purge ne sera demandée à GitHub Support : l'option retenue est la seule
dont le résultat ne dépend de personne d'autre, et elle est vérifiable avant
publication plutôt qu'espérée après.

Ce qui est perdu est réel mais mineur : l'historique des pull requests des
missions M1 à M10. Il n'existe aucune issue, le projet a un seul mainteneur, et
cet historique reste consultable dans l'archive privée. Les messages de commit —
qui portent l'essentiel du raisonnement de ce projet — sont intégralement
conservés, l'historique étant poussé tel quel.

**Conséquence sur ce document :** le dépôt actuel n'a plus vocation à devenir
public, donc les résidus ci-dessus ne sont plus un blocage de publication. Ils
restent documentés parce qu'ils décrivent l'archive privée, et parce qu'ils
expliquent pourquoi la publication passe par un dépôt neuf.

### Branches Dependabot — résolues d'elles-mêmes

Les six branches Dependabot et leurs PR (#8 à #13) ont été **fermées et
supprimées par Dependabot lui-même** lorsque la base a été réécrite : elles
étaient devenues non fusionnables. Cette mission n'a supprimé aucune branche.

La PR #15 a été fermée automatiquement par GitHub pour la même raison — elle
référençait l'ancien HEAD de la branche M10 et ne pouvait plus être rouverte.
Elle est remplacée par la **PR #16**.

---

## Signature et distribution binaire

| Élément | État | Ce que cela bloque |
|---|---|---|
| Apple Developer ID | **`APPLE_DEVELOPER_ID_NOT_AVAILABLE`** | la distribution macOS officielle signée/notarisée — **pas** la publication du source |
| Signature Windows | **`WINDOWS_CODE_SIGNING_NOT_CONFIGURED`** | une future distribution Windows officielle — sans objet aujourd'hui, le portage n'existe pas |

Une build locale reste utilisable : Gatekeeper affiche un avertissement,
contournable par clic droit → **Ouvrir**. C'est documenté dans le README et
dans `BUILDING.md`, sans euphémisme.

---

## Risques résiduels assumés

| Risque | Sévérité | Position |
|---|---|---|
| Chemins de build `/Volumes/Workspace/...` dans les deux sidecars (24 occurrences) | LOW | Cosmétique. Aucun nom d'utilisateur, aucun secret. Corrigeable au prochain rebuild via `-ffile-prefix-map` |
| Sidecars binaires publiés dans Git | INFORMATIONAL | Compromis assumé (STIA-SEC-106) : whisper.cpp ne publie pas de binaire arm64 statique, et exiger une build FFmpeg rendrait le projet inutilisable pour un contributeur |
| Obligation de relink LGPL FFmpeg | `LEGAL_REVIEW_RECOMMENDED` | Documentée (STIA-SEC-202), **réserve maintenue au gate humain**. Ne bloque pas le source ; à traiter avant distribution binaire |
| Transcriptions de la voix de l'auteur publiées | LOW | Script de démonstration écrit pour le test, ne nomme personne. **Acceptées** au gate humain |
| Avis RustSec « unmaintained » sur des crates GTK3 | INFORMATIONAL | 17 warnings, **0 vulnérabilité**. Crates Linux non compilées sur macOS. Déjà hors périmètre dans `SECURITY.md` |
| Anciennes références de PR portant l'historique d'avant réécriture | — | **Sans objet pour la publication** : elles vivent dans le dépôt actuel, qui reste privé définitivement |

---

## Verdict

**`M10_COMMUNITY_PUBLIC_READY_TO_PUBLISH`**

Aucun blocage ne subsiste. Les quatre gates humains sont fermés :

| Gate | Décision |
|---|---|
| Frontière Community / Desktop / Plus | acceptée — [ADR-012](../architecture/ADR-012-community-commercial-boundary.md) `ACCEPTED` |
| Métadonnées d'auteur | remédiées — historique réécrit, 0 occurrence sur les branches |
| Findings de durcissement M10-F04 / F11 | corrigés et requalifiés sur le `.app` empaqueté |
| Mode de publication | **dépôt neuf** — le dépôt actuel reste privé définitivement |

Ce qui reste ouvert n'est pas un blocage :

* `LEGAL_REVIEW_RECOMMENDED` (relink LGPL FFmpeg) — concerne la **distribution
  binaire**, explicitement pas le source ;
* `APPLE_DEVELOPER_ID_NOT_AVAILABLE` et `WINDOWS_CODE_SIGNING_NOT_CONFIGURED` —
  même périmètre ;
* les findings M10-F01, F02, F03, F05 et F09 — Low et Hardening, planifiables
  après publication.

Voir le [rapport de sécurité M10](../security/M10_COMMUNITY_PUBLIC_SECURITY_REVIEW.md)
pour le détail des findings.
