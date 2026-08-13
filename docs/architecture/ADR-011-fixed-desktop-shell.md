# ADR-011 — Shell desktop fixe et layout interne responsive

Statut : `ACCEPTED` — qualifiée humainement le 2026-08-13.

Gate humain : géométrie constante sur toute la navigation, navigation
Réglages jugée naturelle et sans défilement en usage normal, succès FR+EN
lisible, entrée de lancement et états de boutons validés, Reduced Motion
confirmé clair sans mouvement décoratif.

Date : 2026-08-13

Contexte : M9 — « Fixed Desktop Shell + Responsive Internal Layout + Motion
System ».

> **Cette ADR remplace une première version** qui décrivait la direction
> inverse — *content-aware native window sizing*, où la fenêtre suivait la
> hauteur du contenu. Cette direction a été **abandonnée par décision
> produit**, pas parce qu'elle était défectueuse. Elle fonctionnait ; elle
> donnait simplement à ST-IA le comportement d'une page web dans un cadre,
> et non celui d'une application desktop. Le détail de l'ancienne mécanique
> est conservé plus bas sous « Direction précédente (SUPERSEDED) ».

## Contexte

La fenêtre suivait le contenu : chaque changement d'écran mesurait le DOM et
redimensionnait la fenêtre native. Le résultat était correct mais mouvant —
la fenêtre changeait de forme en naviguant, et sa taille dépendait de l'écran
et du contenu plutôt que d'une intention de design.

## Décision 1 — Le shell est fixe, le contenu est conçu pour lui

**900 × 640 px** devient le viewport de référence. Tous les écrans — accueil,
média sélectionné, traitement, succès, réglages, modèles IA, détails
techniques — sont composés pour cette surface exacte, et la fenêtre ne change
jamais de dimensions pendant une session.

Le raisonnement est inversé par rapport à la version précédente : ce n'est
plus « quelle taille faut-il pour ce contenu ? » mais « comment ce contenu
s'organise-t-il dans cette surface ? ». C'est ce qui permet une composition
maîtrisée — une colonne centrée de 560 px pour les écrans de flux, la largeur
complète pour les réglages à deux colonnes — au lieu d'un empilement vertical
qui s'allonge.

## Décision 2 — Une seule décision de taille, au démarrage

`domain/shell.rs` est une fonction pure : elle prend la zone utilisable du
moniteur et rend la taille de session. Elle est appelée **une fois**, dans
`setup`, avant que la moindre fenêtre n'existe.

* zone suffisante → 900 × 640, exactement ;
* zone insuffisante → réduction bornée, **par axe indépendamment** (un écran
  large mais court garde toute sa largeur et ne perd que de la hauteur) ;
* plancher 640 × 460 — en dessous, ce sont les panneaux internes qui
  défilent, pas le shell qui rétrécit encore ;
* zone non mesurable (pas de moniteur, valeur non finie) → taille cible,
  jamais une valeur dérivée de données inutilisables.

Aucun `ResizeObserver`, aucune mesure de DOM, aucun `set_size` piloté par le
contenu n'existe plus.

## Décision 3 — Une seule fenêtre, donc une seule géométrie

Il n'existe qu'une fenêtre native, construite une fois à la taille de session
et centrée. L'écran de démarrage est une couche à l'intérieur de cette même
fenêtre (voir [ADR-009](ADR-009-splashscreen-and-release-packaging.md),
section « Splash intégré »), et non une seconde fenêtre à aligner sur la
première.

Cette décision est ce qui rend la question de la géométrie sans objet : il n'y
a plus deux cadres à faire correspondre, donc plus rien qui puisse diverger.

*Version initiale de cette décision : le splash était une fenêtre distincte
construite à la même taille, et la fenêtre principale était redimensionnée et
recentrée pendant qu'elle était masquée. Cela produisait une géométrie
identique au pixel — mesurée sur 20 lancements — mais le passage d'un cadre
natif à l'autre restait perceptible. D'où la fenêtre unique.*

## Décision 4 — Fenêtre non redimensionnable

`"resizable": false`. C'est la contrepartie assumée d'un shell fixe : une
fenêtre redimensionnable inviterait à une taille pour laquelle aucun écran
n'est composé.

L'accessibilité n'est pas sacrifiée pour autant — c'est le rôle de la
décision suivante.

## Décision 5 — Le défilement est local, jamais global

`.app` ne défile pas. Deux régions le font, et seulement quand c'est
nécessaire :

* `.app-body` — le conteneur des écrans de flux ;
* `.settings-content` — le panneau de droite des réglages.

En usage normal, à la taille cible, aucune des deux ne défile. Elles
prennent le relais quand le contenu dépasse réellement : texte agrandi,
localisation plus longue, écran exceptionnellement petit, ou les deux
disclosures « détails techniques » ouvertes simultanément.

C'est la réponse à §12 : ouvrir les détails techniques ne redimensionne plus
la fenêtre, le panneau concerné défile.

## Décision 6 — Réglages en navigation desktop

Les sections ne sont plus empilées verticalement. Une colonne de navigation
compacte (176 px) à gauche, une section à la fois à droite :

```
┌──────────────────────────────────────────────┐
│ Réglages                                  ✕  │
├───────────────┬──────────────────────────────┤
│ Général       │                              │
│ Accessibilité │   Contenu de la section      │
│ Modèles IA    │                              │
│ À propos      │                              │
└───────────────┴──────────────────────────────┘
```

Implémenté en `role="tablist"` / `role="tab"` / `role="tabpanel"` avec
tabindex glissant : la relation entre chaque entrée et le panneau qu'elle
révèle est explicite pour les technologies d'assistance (« onglet 2 sur 4 »),
et la navigation ne consomme qu'un seul arrêt de tabulation. Flèches haut/bas
pour se déplacer, Début/Fin pour les extrémités — sans bouclage, qui ferait
perdre sa place dans une liste de navigation.

## Décision 7 — Une grammaire de mouvement unique

Des tokens dans `global.css` plutôt que des animations décidées écran par
écran : durées (`instant` / `fast` / `base` / `emphasis`), courbes
(`standard`, `emphasis`, jamais de ressort ni de rebond), et géométrie
d'interaction (`--press-scale: 0.98`, `--hover-lift: -1px`).

Appliqué de façon homogène : survol = lift d'un pixel, clic = compression,
sélection = check qui apparaît en échelle, erreur de validation = déplacement
de trois pixels une seule fois, changement de section = révélation courte.

L'état métier change **toujours immédiatement** ; l'animation ne fait que le
décrire. Aucune dépendance ajoutée : transitions et keyframes CSS.

## Décision 8 — L'entrée de lancement joue une fois

Au retrait de la couche d'intro, les éléments entrent en cascade sur ~300 ms
(60 ms entre chacun). Ce n'est **pas** une transition de navigation : le
verrou est un booléen au niveau du module, pas un état de composant, donc
revenir à l'accueil, annuler un job ou fermer les réglages ne le rejouent
jamais. La distinction demandée entre *l'application apparaît* et *l'écran
change* est structurelle, pas conventionnelle.

## Décision 9 — Reduced Motion retire le mouvement, pas l'information

Sous `data-motion="reduce"` : plus de lift, de compression, de cascade
d'entrée, de nudge d'erreur, de révélation de section, de check animé.

Restent intacts : couleurs, bordures, états sélectionnés, anneaux de focus,
progression réelle, et tout le texte. La règle globale de durée quasi nulle
de `global.css` laisserait ces animations tourner imperceptiblement au lieu
de ne pas tourner ; elles sont donc annulées explicitement.

## Conséquences

* La fenêtre a une identité stable : même taille à chaque lancement, aucune
  variation pendant l'intro, aucun mouvement en navigation.
* Les réglages gagnent une vraie architecture desktop et perdent leur
  défilement permanent — le problème qui avait déclenché la mission
  précédente.
* Les écrans de flux occupent une colonne centrée de 560 px : la surface est
  plus grande, la mesure de lecture ne l'est pas.
* Sur un écran très petit, la fenêtre est réduite une fois puis les panneaux
  internes défilent. Comportement attendu, pas régression.

## Direction précédente (SUPERSEDED)

L'implémentation *content-aware native window sizing* mesurait `.app` via un
`ResizeObserver` (seuil de 6 px, stabilisation de 120 ms) et appelait une
commande `fit_window` qui bornait la demande à 90 % de la zone utilisable.

Elle a été **entièrement retirée**, pas désactivée :
`useFitWindow.ts`, `windowFit.ts`, `commands/window.rs`, la commande
`fit_window` de l'ACL et de la capability, et leurs tests. La seule chose
conservée est la logique pure de bornage, reformulée dans `domain/shell.rs`
pour le calcul **initial** de sécurité petit écran — le seul endroit où elle
reste pertinente.

`tests/shell_contract.rs` verrouille l'absence de retour en arrière : aucune
commande de redimensionnement enregistrée, aucune permission `set-size`
accordée, et les fichiers supprimés doivent rester absents. Une architecture
morte qui compile encore est celle qui ressuscite.

Un défaut réel avait été trouvé et corrigé dans cette direction (boucle de
rétroaction sur la largeur, causée par la barre de défilement du conteneur
de repli). Il est mentionné pour mémoire ; il n'est **pas** la raison de
l'abandon.

## Réserves

* La réduction petit écran est testée en unitaire sur toutes ses branches,
  mais non vérifiée physiquement sur un moniteur plus petit que la cible.
* `resizable: false` désactive aussi le bouton de zoom macOS. Assumé pour une
  application utilitaire de cette taille ; à revoir si un retour utilisateur
  le conteste.

## Références

* [ADR-009 — splashscreen et packaging](ADR-009-splashscreen-and-release-packaging.md)
* [ADR-010 — traduction anglaise locale](ADR-010-local-english-translation.md)
