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
4. **Choisissez vos versions et vos formats.**
   * **Versions** — *Français (original)*, *English (traduction)*, ou les deux.
     Le français seul est sélectionné par défaut.
   * **Formats** — SRT, TXT, ou les deux.

   Le bouton indique combien de fichiers seront créés : deux versions × deux
   formats = 4 fichiers.

   La première fois que vous cochez **English**, ST-IA propose de télécharger
   son modèle de traduction (~3,1 Go, une seule fois, traitement 100 % local).
   Rien n'est téléchargé sans votre clic, et le français seul n'en a jamais
   besoin.
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

**Les modèles ne sont téléchargés qu'une fois.** Ils sont conservés dans
`~/Library/Application Support/com.romainbourbon.stia/models/` et réutilisés
ensuite. Celui de traduction n'est récupéré que si vous demandez l'anglais.

**Formats acceptés** : `.mp4`, `.mov`, `.wav`, `.mp3`, `.m4a`, `.flac`.

**Les fichiers produits.** Le SRT contient les sous-titres avec leurs timecodes,
destiné au montage ; le TXT contient le texte seul, sans horodatage. Les noms
dérivent de celui de votre média :

| Ce que vous demandez | Ce que vous obtenez |
| --- | --- |
| Français seul | `IMG_8484.srt`, `IMG_8484.txt` |
| English seul | `IMG_8484.en.srt`, `IMG_8484.en.txt` |
| Les deux | `IMG_8484.fr.srt`, `IMG_8484.fr.txt`, `IMG_8484.en.srt`, `IMG_8484.en.txt` |

Le français ne prend le suffixe `.fr` que s'il y a une version anglaise à côté —
sinon les noms restent ceux que vous connaissez.

**La langue parlée doit être le français.** C'est la seule langue qualifiée à ce
stade, pour la transcription comme pour la traduction. La langue de l'*interface*
(Système / Français / English) est un réglage séparé et n'affecte ni le traitement
ni le nom des fichiers.

**La traduction est plus lente que la transcription.** Elle utilise un modèle
plus grand et plus précis : comptez environ la durée du média, contre un dixième
pour la transcription française seule. Demander les deux versions additionne les
deux temps — le traitement est séquentiel, jamais simultané.

**Tout ou rien.** Si vous annulez, ou si quelque chose échoue, aucun fichier n'est
créé — pas même la moitié déjà calculée. Relancez simplement.

**Vous pouvez annuler à tout moment.** Rien n'est écrit tant que le traitement
n'est pas entièrement terminé.

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

**« Modèle de traduction non installé »** — vous avez demandé la version anglaise
sans avoir téléchargé son modèle. Le bouton d'installation est proposé directement
sur l'écran.

**« Espace disque insuffisant »** — ST-IA a besoin de la taille de votre média plus
environ 256 Mo de marge.

**macOS refuse d'ouvrir l'application** — les builds actuelles ne sont pas encore
signées. Clic droit sur `ST-IA.app` → **Ouvrir** → confirmer.

Pour savoir précisément quels modèles ST-IA exécute, d'où ils viennent et où le
traitement a lieu, voir [`AI_MODELS.md`](AI_MODELS.md) — ou, dans l'application,
**Réglages → Modèles IA**.

Pour construire depuis les sources, voir [`BUILDING.md`](BUILDING.md).
