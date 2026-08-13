# ST-IA Community — readiness pour publication publique

Établi par M10, mis à jour après remédiation de l'historique. Remplace, pour la question « le dépôt
peut-il devenir public ? », le rapport
[`OPEN_SOURCE_READINESS.md`](../security/OPEN_SOURCE_READINESS.md) de M8, qui
reste valable pour tout ce qu'il a établi.

**M10 n'a rendu aucun dépôt public, n'a créé aucun tag et n'a supprimé aucune
branche.**

---

## Question posée

> Le dépôt ST-IA peut-il devenir public aujourd'hui sans exposer de secret, de
> donnée privée, d'artefact indésirable, de code commercial, de dépendance
> cachée ou de procédure de build non reproductible ?

## Réponse

**Oui.** Aucun blocage technique ne subsiste, et les décisions humaines du gate
M10 ont été prises et appliquées : frontière commerciale acceptée (ADR-012),
métadonnées d'auteur remédiées par réécriture d'historique, et les deux findings
de durcissement corrigés puis requalifiés.

Un point de vigilance subsiste et n'est **pas** une question technique : les
branches Dependabot et les anciennes références de pull request portent encore
l'ancienne adresse dans leur ascendance. Elles doivent être nettoyées avant le
passage en public — voir §métadonnées.

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

### Limite à connaître — ce qui subsiste côté GitHub

Il serait faux d'affirmer que l'ancienne adresse a disparu de GitHub. Trois
résidus sont identifiés :

1. **Les six branches Dependabot** n'ont pas été réécrites. Leurs commits
   propres appartiennent au bot, mais leurs **ancêtres** sont les anciens
   commits et portent donc encore l'ancienne adresse. Elles restent en place :
   la décision humaine était de ne supprimer aucune branche dans cette passe.
2. **Les références de pull request** (`refs/pull/14/head`, `refs/pull/15/head`)
   pointent vers les anciens commits. GitHub conserve ces références même après
   un force-push.
3. **Les anciens objets restent accessibles par SHA direct** sur GitHub tant que
   son ramasse-miettes ne les a pas collectés — un délai que GitHub ne
   garantit pas.

**Conséquence pratique :** la remédiation est complète sur les branches
destinées à la publication (`main` et les branches `feat/m*`), et **incomplète
tant que les branches Dependabot et les anciennes PR existent**. Elles doivent
être fermées et supprimées au gate de nettoyage prévu, **avant** le passage en
public. Le dépôt étant encore privé, aucune de ces références n'est aujourd'hui
visible publiquement.

---|---|---|
| `Romain Bourbon <…@…>` (adresse professionnelle, redacted) | **85** | domaine d'entreprise |
| `dependabot[bot] <…@users.noreply.github.com>` | 6 | bot, adresse noreply |

Exactement deux identités, aucune faute de frappe, aucune adresse périmée : les
métadonnées sont propres. Le point n'est pas leur qualité mais leur **nature**.

Ce qu'il faut savoir avant de décider :

* l'adresse deviendra **définitivement publique** — dans chaque commit, chaque
  clone, chaque miroir et chaque archive ; la retirer plus tard ne la retire pas
  des copies déjà faites ;
* elle rattachait publiquement ce projet personnel à un domaine d'entreprise ;
* elle est **récoltable** par les moissonneurs d'adresses ;
* l'auteur est déjà public par ailleurs (`LICENSE`, identifiant de bundle
  `com.romainbourbon.stia`) : l'exposition supplémentaire porte sur l'**adresse
  et le domaine**, pas sur l'identité.

**Options :**

1. **Accepter** — aucune action. Le plus simple, et défendable si l'adresse est
   assumée publiquement par ailleurs.
2. **Réécrire l'historique** (`git filter-repo`) vers une adresse
   `@users.noreply.github.com`. Change **tous les SHA**, invalide toute
   référence externe existante, et **M10 s'interdit formellement de le faire
   sans autorisation explicite**.
3. **Changer pour les commits futurs uniquement** — l'historique reste tel quel.
   Cohérence partielle, aucun SHA modifié.

Aucune de ces options n'est appliquée par M10. C'est le gate
`GIT_AUTHOR_METADATA_ACCEPTANCE_REQUIRED`.

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
| Obligation de relink LGPL FFmpeg | `LEGAL_REVIEW_RECOMMENDED` | Documentée (STIA-SEC-202), non tranchée. Ne bloque pas le source ; à traiter avant distribution binaire |
| Transcriptions de la voix de l'auteur publiées | LOW | Script de démonstration écrit pour le test, ne nomme personne. Décision humaine |
| Avis RustSec « unmaintained » sur des crates GTK3 | INFORMATIONAL | 17 warnings, **0 vulnérabilité**. Crates Linux non compilées sur macOS. Déjà hors périmètre dans `SECURITY.md` |

---

## Verdict

Aucun blocage technique. Les points ouverts sont des **arbitrages humains**, et
le seul qui mérite une décision consciente avant la bascule est celui des
métadonnées d'auteur.

Voir le [rapport de sécurité M10](../security/M10_COMMUNITY_PUBLIC_SECURITY_REVIEW.md)
pour le détail des findings.
