# ADR-012 — Frontière Community / Desktop / Plus

Statut : `ACCEPTED` — validée au gate humain M10, le 2026-08-13.

Date : 2026-08-13

Contexte : M10 — « Community Public Release Readiness ».

## Contexte

ST-IA atteint sa release candidate `0.1.0` dans un dépôt privé. La question
posée à M10 n'est pas « faut-il ouvrir le code ? » — la licence MIT a été
choisie en M8 — mais **où passe la frontière** entre ce qui est publié et ce
qui pourra un jour être vendu.

Cette décision doit être prise **avant** la publication, pas après. Une fois
un fichier publié sous MIT, il est publié sous MIT pour toujours : le retirer
d'un commit ultérieur ne retire ni les clones, ni les forks, ni les droits
déjà accordés par la licence sur cette version. La frontière est donc une
décision irréversible dans un seul sens.

## Décision 1 — Trois produits, deux dépôts

```text
ST-IA Community
GitHub public / MIT
        │
        │ upstream
        ▼
ST-IA Desktop
dépôt privé (n'existe pas encore)
        │
        ├── Desktop Free
        └── Desktop Plus
```

**ST-IA Community** — ce dépôt. Public, MIT, destiné à un public technique
capable de cloner, installer les dépendances de développement et construire
l'application lui-même.

**ST-IA Desktop** — futur dépôt **privé**, distinct. Distribution officielle
prête à installer : installateurs, signature, notarisation, updater, support.
Il n'existe pas encore et M10 ne le crée pas.

**ST-IA Plus** — future couche premium de Desktop. N'a aucune existence dans
Community.

## Décision 2 — Community est une vraie application, pas une démo mutilée

Le cœur Community n'est pas une version d'évaluation. Il contient l'intégralité
de ce qui fait le produit :

* transcription française locale ;
* traduction anglaise locale ;
* sorties SRT et TXT, FR / EN / FR+EN ;
* gestion des modèles locaux, avec vérification d'intégrité ;
* téléchargement sécurisé et épinglé des modèles ;
* progression réelle, annulation, reprise après échec ;
* transparence sur les modèles exécutés ;
* l'interface complète de la v0.1.

Aucune de ces capacités ne sera retirée de Community pour être revendue. Un
utilisateur qui construit depuis les sources obtient l'application, pas un
aperçu.

Ce qui distinguera Desktop n'est pas *la transcription* mais la **commodité de
distribution** (installateur signé, notarisé, mis à jour) et des fonctions de
volume et de flux de travail qui n'existent aujourd'hui nulle part.

## Décision 3 — Aucune fonction propriétaire n'est développée dans le dépôt MIT

C'est la règle centrale de cette ADR, et elle est structurelle plutôt que
contractuelle.

**Anti-pattern explicitement rejeté :**

```text
dépôt public MIT
└── implémentation premium
    └── masquée derrière un drapeau de licence
```

Ce montage ne protège rien. Le code premium serait présent dans un dépôt
publié sous MIT, donc **publié sous MIT** : chacun aurait le droit de l'utiliser,
de le modifier et de le redistribuer, drapeau retiré. Le drapeau ne serait
qu'une politesse, et son contournement serait parfaitement licite.

La conséquence pratique est une discipline de développement :

> Une fonction destinée à Desktop Plus ne doit jamais être écrite dans
> Community « en attendant », même désactivée, même derrière un drapeau, même
> incomplète. Elle est écrite directement dans le dépôt privé.

L'extraction se fait dans un seul sens : Community → Desktop. Jamais l'inverse.

## Décision 4 — Desktop consomme Community comme upstream

Desktop ne sera pas un fork divergent : ce serait deux bases de code à
maintenir. Il conserve Community comme `remote upstream` et remonte les
correctifs cœur.

* les correctifs et améliorations du **cœur** sont écrits dans Community,
  puis remontés dans Desktop ;
* le code **propriétaire** vit uniquement dans Desktop et ne descend jamais ;
* Desktop documente en permanence le tag et le SHA Community qu'il consomme.

La procédure détaillée est dans
[`DESKTOP_UPSTREAM_STRATEGY.md`](DESKTOP_UPSTREAM_STRATEGY.md). Elle n'est pas
exécutée par M10.

## Décision 5 — Le dépôt public ne promet rien de commercial

Le README public décrit ce qui existe. Il ne présente ni prix, ni abonnement,
ni fonction Plus comme disponible. Une mention sobre du type « des
distributions officielles prêtes à l'emploi pourront être proposées séparément »
est acceptable ; une page marketing ne l'est pas.

Raison : un dépôt open source qui annonce des fonctions payantes inexistantes
perd la confiance des deux publics à la fois — les contributeurs y voient un
produit d'appel, les utilisateurs une promesse non tenue.

## Décision 6 — Deux cycles de versions séparés

Community suit SemVer classique : `v0.1.0`, `v0.2.0`, … Desktop aura son
propre cycle, indépendant, et déclare explicitement l'upstream Community qu'il
embarque. Une version Desktop `1.4.0` pourra reposer sur Community `v0.3.2`
sans que les numéros aient à coïncider.

## Conséquences

**Assumées :**

* le cœur du produit est donné, y compris à un concurrent potentiel — c'est le
  prix d'un projet local-first crédible sur la vie privée : une application qui
  affirme ne rien envoyer doit pouvoir être vérifiée ;
* la valeur commerciale se déplace vers ce qui ne se copie pas facilement : la
  distribution signée, la maintenance, le support et les fonctions de volume ;
* la discipline de la décision 3 impose parfois d'écrire une fonction deux
  fois plutôt que de la partager — coût accepté, il achète une frontière
  juridique nette.

**Rejetées :**

* *Open core à drapeau* — juridiquement inopérant (décision 3).
* *Licence source-available non libre* (BSL, PolyForm) — aurait préservé la
  frontière, mais contredit l'engagement MIT pris en M8 et l'argument de
  vérifiabilité.
* *Tout garder privé* — un produit qui affirme ne pas exfiltrer les données
  sans permettre de le vérifier demande une confiance qu'il n'a pas méritée.

## Ce que cette ADR ne fait pas

Elle ne crée pas le dépôt Desktop, n'écrit aucun code de licensing, ne rend
aucun dépôt public et ne crée aucun tag. Elle fixe la frontière **avant** que
la publication ne la rende irréversible.

## Références

* [`docs/COMMUNITY_EDITION.md`](../COMMUNITY_EDITION.md) — présentation publique
* [`DESKTOP_UPSTREAM_STRATEGY.md`](DESKTOP_UPSTREAM_STRATEGY.md) — procédure d'extraction
* [`docs/release/COMMUNITY_PUBLIC_READINESS.md`](../release/COMMUNITY_PUBLIC_READINESS.md)
* [`LICENSE`](../../LICENSE) — MIT
