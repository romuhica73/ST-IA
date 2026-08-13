# ST-IA Desktop — stratégie d'extraction depuis Community

Statut : **plan**. Rien de ce document n'est exécuté par M10.

Ce document décrit comment le futur dépôt privé **ST-IA Desktop** devra être
créé à partir de **ST-IA Community**, et comment les deux resteront synchronisés
sans que le code propriétaire ne remonte jamais dans le dépôt public.

La frontière elle-même est décidée dans
[ADR-012](ADR-012-community-commercial-boundary.md). Ce document est la
procédure, pas la décision.

---

## Prérequis

À faire **avant** toute création du dépôt Desktop :

* Community est public et stable ;
* un tag Community existe et fait autorité (`v0.1.0` au minimum) ;
* ADR-012 est `ACCEPTED`.

Extraire depuis une branche mouvante plutôt que depuis un tag rendrait
impossible de dire, plus tard, quelle version de Community une build Desktop
contient.

---

## 1. Créer le dépôt privé depuis un tag

```sh
# Depuis un clone propre de Community, positionné sur un tag publié.
git clone https://github.com/romuhica73/ST-IA.git ST-IA-Desktop
cd ST-IA-Desktop
git checkout v0.1.0
```

Deux options, à trancher au moment venu :

| Option | Historique | Conséquence |
|---|---|---|
| **A — conserver l'historique** | complet | Desktop hérite de tout l'historique Community ; les remontées `merge` sont naturelles |
| **B — repartir d'un commit initial** | tronqué | Desktop démarre propre, mais chaque remontée devient un `cherry-pick` manuel |

**Recommandation : option A.** L'historique Community est destiné à être
public de toute façon ; le conserver rend la synchronisation triviale, alors
que l'option B transforme chaque correctif cœur en travail manuel.

Puis repointer l'origine et conserver Community en amont :

```sh
git remote rename origin upstream
git remote add origin git@github.com:<compte>/ST-IA-Desktop.git   # dépôt PRIVÉ
git push -u origin main
```

Vérifier immédiatement que le dépôt Desktop est bien privé **avant** le premier
push contenant du code propriétaire.

---

## 2. Topologie des remotes

```text
upstream  →  ST-IA Community   (public, MIT)     — lecture seule
origin    →  ST-IA Desktop     (privé)           — lecture / écriture
```

**Règle absolue : `push` vers `upstream` est interdit depuis Desktop.**

Une protection locale simple, à poser dès la création :

```sh
git remote set-url --push upstream DISABLED
```

Toute tentative de push vers `upstream` échoue alors sur une URL invalide,
plutôt que de publier accidentellement du code propriétaire.

---

## 3. Où vit quoi

| Nature du changement | Écrit dans | Remonte vers Desktop | Descend vers Community |
|---|---|---|---|
| Correctif du pipeline, du modèle, de l'UI cœur | **Community** | oui | — |
| Correctif de sécurité du cœur | **Community** | oui | — |
| Installateur signé, notarisation, updater | **Desktop** | — | **jamais** |
| Licensing, comptes, paiement, entitlements | **Desktop** | — | **jamais** |
| Historique, bibliothèque, batch, workflows avancés | **Desktop** | — | **jamais** |

La colonne « descend vers Community » n'a qu'une valeur : **jamais**. C'est ce
qui rend la frontière tenable. Un correctif cœur découvert en travaillant sur
Desktop doit être **réécrit dans Community** puis remonté, et non poussé
directement dans Desktop.

---

## 4. Synchroniser Desktop avec Community

```sh
git fetch upstream --tags
git merge upstream/v0.2.0        # un tag, pas upstream/main
```

Se synchroniser sur des **tags** plutôt que sur `upstream/main` : une build
Desktop doit toujours pouvoir nommer exactement l'upstream qu'elle contient.

Après chaque synchronisation, mettre à jour dans Desktop un fichier
`UPSTREAM.md` :

```markdown
Upstream : ST-IA Community
Tag      : v0.2.0
SHA      : <sha complet>
Synchro  : <date>
```

Sans cette trace, il devient impossible de répondre à « quelle version du cœur
tourne dans Desktop 1.4.0 ? » — question qui se pose au premier avis de
sécurité sur le cœur.

---

## 5. Isoler le code propriétaire

Le code Desktop-only doit être **physiquement séparé**, pas mélangé aux
fichiers venant de Community. Un répertoire dédié, absent de Community :

```text
ST-IA-Desktop/
├── src/            ← vient de Community (éviter d'y écrire)
├── src-tauri/      ← vient de Community (éviter d'y écrire)
└── desktop/        ← Desktop uniquement — n'existe pas dans Community
```

Bénéfice concret : un `git merge upstream/<tag>` ne produit de conflit que là
où Desktop a réellement modifié un fichier cœur. Plus la surface modifiée dans
`src/` et `src-tauri/` est petite, moins la synchronisation coûte cher — et
moins le risque de renvoyer par mégarde du code propriétaire dans une PR
Community est élevé.

---

## 6. Contrôle avant chaque release Desktop

- [ ] `git remote -v` — `upstream` en push est bien désactivé
- [ ] le dépôt Desktop est toujours privé
- [ ] `UPSTREAM.md` reflète le tag Community réellement fusionné
- [ ] aucun fichier de `desktop/` n'est apparu dans une PR Community
- [ ] les notices de licence Community (MIT, FFmpeg LGPL, whisper.cpp MIT) sont
      conservées dans la distribution Desktop — MIT exige la conservation de
      l'avis de copyright, y compris dans un produit commercial

---

## 7. Ce qui n'est pas tranché

Ces points restent ouverts et devront l'être avant la création effective :

* **Nom du compte / de l'organisation** hébergeant Desktop.
* **Option A ou B** pour l'historique (recommandation : A).
* **Modèle de licence Desktop** — propriétaire classique, ou source-available
  pour les clients ? Aucune décision prise.
* **Entité juridique** de vente. Aucune entité n'existe aujourd'hui et le
  dépôt public n'en mentionne aucune.

Ces questions n'ont pas à être résolues pour publier Community. Elles sont
listées ici pour qu'elles ne soient pas oubliées au moment où Desktop démarrera.
