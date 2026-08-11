# ST-IA — Démarrage rapide

Générer des sous-titres à partir d'une vidéo ou d'un fichier audio, entièrement sur
votre Mac.

## En 6 étapes

1. **Lancez ST-IA.**
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
   puis déposez la piste sur votre timeline.

## Ce que vous devez savoir

**Tout se passe sur votre Mac.** Votre média n'est jamais envoyé nulle part. Après le
téléchargement du modèle, ST-IA fonctionne entièrement hors ligne — vous pouvez couper
le Wi-Fi et vérifier.

**Le modèle n'est téléchargé qu'une fois.** Il est conservé dans
`~/Library/Application Support/com.romainbourbon.stia/` et réutilisé ensuite.

**Formats acceptés** : `.mp4`, `.mov`, `.wav`, `.mp3`, `.m4a`, `.flac`.

**La transcription est en français.** C'est la seule langue qualifiée à ce stade. La
langue de l'*interface* (Système / Français / English) est un réglage séparé et
n'affecte pas la transcription.

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
