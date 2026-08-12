# ST-IA — Démarrage rapide

Générer des sous-titres à partir d'une vidéo ou d'un fichier audio, entièrement sur
votre Mac.

## Installation

ST-IA se distribue sous la forme d'une image disque `ST-IA-<version>-macos-arm64.dmg`.

1. **Ouvrez le `.dmg`** téléchargé — une fenêtre s'ouvre avec l'application et un
   raccourci vers `Applications`.
2. **Glissez `ST-IA` sur `Applications`.**
3. **Éjectez l'image disque**, puis lancez ST-IA depuis le Launchpad ou le dossier
   Applications.

> **Aucune release publique n'est encore disponible.** Les builds actuelles ne sont ni
> signées ni notariées par Apple : au premier lancement, macOS refusera d'ouvrir
> l'application. Faites un **clic droit sur `ST-IA.app` → Ouvrir → Ouvrir** pour
> confirmer une fois. En attendant une version signée, vous pouvez aussi
> [construire depuis les sources](BUILDING.md).

## En 6 étapes

1. **Lancez ST-IA.** Un écran de démarrage apparaît brièvement pendant l'initialisation.
2. **Téléchargez le modèle** au premier usage. ST-IA affiche « Modèle requis » et
   attend votre clic — 574 Mo, une seule fois. C'est le seul moment où l'application
   utilise le réseau.
3. **Déposez votre vidéo ou votre audio** dans la fenêtre, ou cliquez pour la
   sélectionner.
4. **Choisissez vos formats** (SRT, TXT, ou les deux) et lancez la génération.
5. **Récupérez vos fichiers.** Ils sont écrits dans un dossier
   `<nom-du-média>-sous-titres/`, créé **à côté de votre média**. Le bouton
   « Ouvrir le dossier » vous y emmène directement.
6. **Importez le SRT dans DaVinci Resolve** : menu **File → Import → Subtitle**,
   choisissez `<nom-du-média>.srt`, puis glissez la piste apparue dans le *Media Pool*
   sur votre timeline, au-dessus de la vidéo. Les timecodes du SRT partent de zéro :
   alignez le début de la piste sur le début du média pour que la synchronisation soit
   correcte.

## Ce que vous devez savoir

**Tout se passe sur votre Mac.** Votre média n'est jamais envoyé nulle part. Après le
téléchargement du modèle, ST-IA fonctionne entièrement hors ligne — vous pouvez couper
le Wi-Fi et vérifier.

**Le modèle n'est téléchargé qu'une fois.** Il est conservé dans
`~/Library/Application Support/com.romainbourbon.stia/` et réutilisé ensuite.

**Formats acceptés** : `.mp4`, `.mov`, `.wav`, `.mp3`, `.m4a`, `.flac`.

**Les fichiers produits.** Vous choisissez `SRT`, `TXT`, ou les deux. Le SRT contient
les sous-titres avec leurs timecodes, destiné au montage ; le TXT contient le texte
seul, sans horodatage. Ils portent le nom de votre média : `IMG_8484.srt`,
`IMG_8484.txt`.

**La transcription est en français.** C'est la seule langue qualifiée à ce stade, et
ST-IA ne produit pas de traduction. La langue de l'*interface* (Système / Français /
English) est un réglage séparé et n'affecte ni la transcription ni le nom des fichiers.

**Comptez à peu près la durée du média.** Une vidéo de 10 minutes prend de l'ordre de
10 minutes. Vous pouvez annuler à tout moment : rien n'est écrit tant que la
transcription n'est pas terminée.

**Configuration requise** : macOS sur Apple Silicon (M1 ou plus récent). Les Mac Intel
ne sont pas supportés.

## Réglages

Icône engrenage, en haut à droite :

* **Thème** — Système / Clair / Sombre (bascule rapide aussi disponible dans l'en-tête) ;
* **Animations** — réduction possible pour l'accessibilité ;
* **Langue de l'interface** — Système / Français / English ;
* **À propos** — version installée et licences des composants tiers.

## En cas de problème

**« Aucune piste audio détectée »** — le fichier ne contient pas d'audio à transcrire.

**« Modèle endommagé »** — le fichier téléchargé ne correspond pas à l'empreinte
attendue. Relancez le téléchargement ; ST-IA n'utilisera jamais un modèle non vérifié.

**« Espace disque insuffisant »** — ST-IA a besoin de la taille de votre média plus
environ 256 Mo de marge.

**macOS refuse d'ouvrir l'application** — les builds actuelles ne sont pas encore
signées. Clic droit sur `ST-IA.app` → **Ouvrir** → confirmer.

Pour construire depuis les sources, voir [`BUILDING.md`](BUILDING.md).
