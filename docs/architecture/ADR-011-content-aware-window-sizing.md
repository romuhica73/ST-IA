# ADR-011 — Fenêtre principale adaptée au contenu

Statut : `PROVISIONAL` — passe à `ACCEPTED` après la qualification humaine.

Date : 2026-08-13

Contexte : M9 — correctif « Final Window Fit Polish ».

## Contexte

La fenêtre principale était fixe (720×520), y compris dans le panneau
Réglages, qui compensait par un scroll interne. Avec l'ajout de la section
Modèles IA en M9, ce scroll est devenu systématique même en usage normal —
la fenêtre se comportait comme une page web contrainte, pas comme une
application desktop macOS.

## Décision 1 — Le contenu pilote, le scroll est un filet de sécurité

`.app` (le contenu réel) n'est plus forcé à `100vh`. Il grandit ou rétrécit
avec son contenu naturel ; c'est cette taille naturelle qui est mesurée et
demandée à la fenêtre. `.app-viewport`, qui l'enveloppe, reste fixé à
`100vh` avec défilement — c'est le filet de sécurité pour le seul cas où la
fenêtre a atteint sa taille maximale et que le contenu la dépasse encore.

En usage normal, aucune barre de défilement n'apparaît jamais : la fenêtre a
déjà pris la taille du contenu avant que l'utilisateur ait quoi que ce soit à
faire défiler.

## Décision 2 — Politique de dimensionnement pure, côté Rust

La logique de calcul (`src-tauri/src/domain/window_fit.rs`) ne connaît aucun
type Tauri : elle prend une taille désirée et une zone utilisable, et rend
une taille bornée. Cela permet de tester exactement les règles du protocole
comme de l'arithmétique pure, sans fenêtre réelle ni moniteur simulé.

Règles :

* **désiré < disponible → désiré**, inchangé ;
* **désiré > disponible → plafonné** à 90 % de la zone utilisable du moniteur
  (`Monitor::work_area`, qui exclut déjà la barre de menu et le Dock côté
  macOS — pas une approximation, l'API Tauri l'expose réellement) ;
* **minimum respecté** : 480×400, plancher absolu, jamais la mécanique
  principale ;
* **sécurité prioritaire sur le minimum** : si l'écran est plus petit que le
  plancher, c'est l'écran qui gagne — la fenêtre ne dépasse jamais l'espace
  réellement disponible, quitte à passer sous 480×400 ;
* **largeur essentiellement fixe** : le contenu de ST-IA est mono-colonne et
  ne demande jamais plus de 720 px ; la politique accepte une largeur
  variable pour la sécurité (petit écran) mais rien dans l'interface actuelle
  ne déclenche une croissance horizontale ;
* **NaN/négatif → plancher**, jamais propagé tel quel à un appel natif de
  redimensionnement.

Une fonction séparée, `clamp_position`, garde le coin supérieur gauche de la
fenêtre à l'intérieur de la zone utilisable — utile si l'utilisateur déplace
la fenêtre vers un écran externe plus petit après un agrandissement de
contenu.

## Décision 2 bis — la largeur n'est jamais mesurée, seulement la hauteur

Défaut réel trouvé en testant l'application empaquetée : la largeur
**s'effondrait en boucle**, `720 → 703 → 686 → 669 → …`, jusqu'au plancher
480, en pas de 17 px exactement à chaque appel.

Cause : `.app` mesuré (`offsetWidth`) vit à l'intérieur de `.app-viewport`,
dont la barre de défilement verticale — le filet de sécurité du §7 — prend de
la place horizontale **quand elle est visible**. Le premier appel mesurait
`.app` avant que le contenu réel ne soit monté (état de chargement bref,
hauteur ~56 px), le pli sur le plancher de hauteur (400) déclenchait
l'apparition de la barre de défilement (le contenu réel dépassait alors
légèrement cette hauteur), qui volait ~17 px à la largeur mesurée au tour
suivant — et la fenêtre, redimensionnée à cette largeur plus étroite, gardait
la barre de défilement (elle ne dépend que de la hauteur), donc la mesure
suivante perdait encore 17 px. Une boucle de rétroaction pure.

Correctif : la largeur **n'est plus jamais mesurée depuis le DOM**. Elle est
fixée à la constante `NATURAL_WIDTH` (720, la même valeur qu'avant M9,
celle sur laquelle toute l'interface a été calée visuellement) et transmise
telle quelle à chaque appel. Seule la hauteur provient d'une mesure réelle.
Cela correspond exactement à la décision produit déjà prise en §2/§3 du
protocole (« ne pas agrandir inutilement horizontalement ») — la largeur
n'avait de toute façon jamais besoin de varier ; la mesurer était la seule
erreur.

## Décision 3 — Mesure côté frontend, décision côté backend

`useFitWindow` observe l'élément `.app` avec un `ResizeObserver`, et envoie
sa taille réelle (`offsetWidth`/`offsetHeight`) à la commande `fit_window`
à chaque changement significatif. Le hook ne fait aucun calcul de bornage —
il mesure et transmet ; toute la politique vit dans `window_fit.rs`, seule
source de vérité, testée une fois plutôt que dupliquée en deux langages.

Deux garde-fous empêchent le tremblement :

* **seuil de signification** (6 px) : le bruit de rendu sub-pixel ne
  déclenche jamais un redimensionnement ;
* **fenêtre de stabilisation** (120 ms) : une transition ou l'ouverture d'un
  disclosure passe par des états intermédiaires avant de se stabiliser ; la
  demande de resize part une fois, sur la valeur stabilisée, jamais à chaque
  frame intermédiaire.

## Décision 4 — Pas d'animation de redimensionnement

Tauri ne propose pas de transition native fiable pour `set_size`, et une
fausse animation implémentée à la main (interpolation JS avec des appels
`set_size` répétés) recréerait exactement le problème de tremblement que les
garde-fous ci-dessus évitent. Le redimensionnement est donc **immédiat** —
propre plutôt que fragile, conformément au principe énoncé dans le protocole.

## Décision 5 — Pas de traitement spécial pour le splash

Le splash garde son architecture et sa taille fixes, non modifiées. La
fenêtre principale, elle, se mesure et s'ajuste dès le montage — y compris
pendant qu'elle est encore masquée (`visible: false`). Le cycle du splash
dure 6 secondes ; le premier ajustement de taille se produit largement avant
cette échéance, donc la fenêtre principale a déjà sa taille correcte au
moment de la bascule. Aucune séquence spéciale n'a été nécessaire pour
garantir « pas de petite fenêtre suivie d'un resize visible ».

## Conséquences

* Chaque écran (accueil, sélection, progression, succès, réglages, modèles
  IA, détails techniques ouverts) prend sa taille naturelle sans défilement,
  jusqu'à 90 % de l'écran.
* Un écran très chargé sur un très petit moniteur peut encore défiler — c'est
  le comportement attendu, pas une régression.
* La largeur ne varie plus dans la pratique : elle reste 720 px sur tout
  écran raisonnable, ce qui préserve l'ensemble du travail visuel déjà
  qualifié (M6, M7) sans reprise.
* Le centrage initial (`"center": true`) et le minimum déclaratif
  (`minWidth`/`minHeight` dans `tauri.conf.json`) s'ajoutent au comportement
  dynamique sans dépendre de lui.

## Réserves

* Le repositionnement multi-écran (`clamp_position`) est raisonné et testé
  en unitaire, mais non vérifié physiquement avec plusieurs moniteurs — pas
  de second écran disponible pour cette qualification.
* Le seuil de stabilisation (120 ms) est un choix UX, pas une mesure
  universelle ; à revalider si un état futur introduit une animation plus
  longue avant stabilisation.

## Références

* [ADR-009 — splashscreen et packaging](ADR-009-splashscreen-and-release-packaging.md)
* Tauri 2 `Monitor::work_area` — zone utilisable réelle par plateforme.
