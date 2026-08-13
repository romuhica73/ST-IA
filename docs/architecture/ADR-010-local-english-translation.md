# ADR-010 — Traduction anglaise locale par un second modèle

Statut : `ACCEPTED` — qualifiée humainement le 2026-08-13.

Gate humain : sortie anglaise seule, français + anglais (4 fichiers groupés
par version), annulation pendant la traduction puis relance complète, et
lecture des passages début / milieu / fin. Traduction jugée **utile**, la
réserve « pas parfaite » de la section Réserves restant explicitement en
vigueur.

Date : 2026-08-12

Contexte : M9 — correctif « Premium Splashscreen + English Translation Model ».

## Contexte

[ADR-008](ADR-008-bilingual-output-pipeline.md) avait conclu au rejet de la
sortie bilingue : le modèle épinglé `large-v3-turbo-q5_0` ignore le drapeau
`-tr` et renvoie du français. Toutes les issues étaient alors hors périmètre.

L'auteur a depuis levé explicitement cette contrainte, avec une priorité
produit claire : **la meilleure qualité de traduction possible**, le poids, la
vitesse et l'espace disque venant ensuite. ADR-008 reste valable dans son
constat technique ; cette ADR remplace sa décision.

## Décision 1 — Deux modèles, pas un modèle remplacé

Le modèle de transcription française reste **`large-v3-turbo-q5_0`**,
inchangé. Remplacer le modèle unique par un `large-v3` complet aurait dégradé
le cas d'usage principal — la transcription française — d'un facteur mesuré
de 7,5× sur le temps de traitement, pour un bénéfice nul sur ce cas.

Un **second** modèle est ajouté, dédié à la traduction :

| | Transcription | Traduction |
| --- | --- | --- |
| Modèle | `ggml-large-v3-turbo-q5_0.bin` | `ggml-large-v3.bin` |
| Taille | 574 041 195 o | 3 095 033 483 o |
| SHA-256 | `394221709c…a7e2` | `64d182b440…d1e2` |
| Quantisé | oui (`q5_0`) | non |
| Requis | pour tout job | uniquement si English est demandé |

Non quantisé délibérément : la décision produit est la qualité d'abord. Un
`large-v3-q5_0` aurait divisé la taille par trois, mais la traduction est
précisément l'étape où la dégradation de quantisation se voit le plus.

Les deux fichiers proviennent du **même commit épinglé** de
`ggerganov/whisper.cpp` sur Hugging Face que celui déjà vérifié en M3/M8 —
pas de `main`, qui est un pointeur de branche déplaçable.

## Décision 2 — Qualification préalable du modèle

Vérifié avant toute intégration, sur cette machine (M4, 16 Go) :

* **provenance** : même dépôt et même commit épinglé que le modèle existant ;
* **taille** : 3 095 033 483 o, conforme au `x-linked-size` annoncé ;
* **SHA-256** : `64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2`,
  identique au `x-linked-etag` du serveur et recalculé localement après
  téléchargement ;
* **chargement Metal** : `use gpu = 1`, `flash attn = 1`, bibliothèque Metal
  embarquée chargée en 0,107 s ;
* **stabilité** : sortie 0, aucun crash ;
* **mémoire** : pic RSS **3,85 Go** — confortable sur 16 Go ;
* **vitesse** : 163,9 s pour 181 s d'audio, soit ~0,9× le temps réel (contre
  ~0,12× pour la transcription turbo) ;
* **résultat** : anglais correct, 41 segments, SRT valide, timecodes
  croissants, aucun résidu de français.

La traduction est donc **réellement réalisable en local** sur la machine
cible. Réserve : un segment de durée nulle a été observé
(`00:00:56,100 --> 00:00:56,100`) ; artefact connu de Whisper, sans impact sur
la validité du SRT, à surveiller.

Preuves : [`spike/out/m9-translation-audit/`](../../spike/out/m9-translation-audit/).

## Décision 3 — Téléchargement à la demande, jamais embarqué

Le modèle de traduction n'est présent ni dans le `.app`, ni dans le `.dmg`,
ni dans Git, ni dans les artefacts de release. Il est téléchargé uniquement
si l'utilisateur sélectionne la version English, avec exactement les mêmes
garanties que le modèle de transcription (M3/M8) : HTTPS strict, URL épinglée,
redirections bornées, plafond de taille en cours de flux, SHA-256 vérifié,
fichier temporaire `.download`, promotion par renommage atomique, aucun
téléchargement silencieux.

L'utilisateur qui ne demande jamais d'anglais ne télécharge jamais 3,1 Go.

## Décision 4 — Vérification paresseuse de ce second modèle

La détection vérifie le SHA-256 du fichier, ce qui coûte ~9 s d'E/S pour
3,1 Go. Le faire à chaque démarrage aurait ralenti tout le monde, y compris
la majorité qui ne demande jamais d'anglais.

Le statut du modèle de traduction n'est donc interrogé **qu'une fois un média
sélectionné** — moment où la réponse devient nécessaire, et où le coût est
masqué par le temps que l'utilisateur passe à choisir ses options.

Défaut trouvé et corrigé au passage : `get_model_status` était une commande
Tauri **synchrone**, donc exécutée sur le thread principal. Le hachage de
3,1 Go y gelait toute l'application pendant ~9 s. Elle est désormais `async`
et délègue à `spawn_blocking`.

## Décision 5 — Passes séquentielles, un seul FFmpeg

```
FFmpeg (une fois)
  └─ audio.wav (une fois, jamais recopié)
       ├─ passe 1 — turbo,     transcribe → transcript-fr.{srt,txt}
       └─ passe 2 — large-v3,  translate  → transcript-en.{srt,txt}
            └─ publication atomique
```

Les passes s'exécutent dans une boucle `for` unique, sans `spawn` ni `join` :
à tout instant le job détient **au plus un enfant `whisper-cli`**, ce qui
préserve littéralement la garantie M4 « un job, un enfant » pour un job
bilingue. Le handle du premier enfant est repris et détruit avant que le
second ne démarre.

Le français passe en premier — pas par symétrie, mais parce que c'est la
passe courte : une annulation pendant la longue traduction a ainsi déjà eu sa
meilleure occasion de se produire plus tôt.

Dans les deux passes, `-l fr` désigne la langue **parlée**. Seul `-tr`
distingue la traduction. Mettre `-l en` reviendrait à affirmer que l'audio est
anglais, ce qu'il n'est pas.

## Décision 5 bis — `-mc 0` sur la passe de traduction

Le modèle entrait en boucle de décodage sur le média qualifié : 11 répétitions
consécutives d'une même phrase, horodatages de durée nulle, et — le plus
grave — **~15 secondes de discours réellement prononcé perdues**, remplacées
par la phrase répétée.

Cause isolée : `--max-context`, non borné par défaut dans whisper.cpp. Chaque
fenêtre de décodage reçoit le texte que la précédente a produit ; quand une
phrase se répète, sa propre répétition devient le contexte qui la fait se
répéter à nouveau.

Variantes mesurées sur le média qualifié, passe de traduction :

| Arguments | Cues | Plus longue répétition | Cues de durée nulle | Temps |
| --- | --- | --- | --- | --- |
| défaut (`-mc -1`) | 41 | **11** | 14 | 96,5 s |
| `-et 2.8` | 41 | 11 | 14 | 93,8 s |
| `-sns` | 42 | 11 | 0 | 52,0 s |
| **`-mc 0`** | 36 | **1** | **0** | **38,0 s** |

`-et 2.8` produit une sortie strictement identique. `-sns` nettoie les cues
dégénérées sans casser la boucle. **`-mc 0` supprime le phénomène**, restitue
le passage perdu (vérifié contre la transcription française de référence) et
divise le temps de traitement par 2,5 — une boucle consomme des étapes de
décodage pour rien.

Ce n'est **pas** une déduplication a posteriori. Le mission brief interdisait
de supprimer automatiquement des segments identiques sans contexte, et c'est
exactement ce qui n'est pas fait ici : la boucle est *empêchée*, aucun segment
n'est retiré, donc une répétition légitime du discours reste intacte.

Contrepartie assumée : moins de cohérence à longue portée entre fenêtres
(pronoms, terminologie). Sur l'échantillon qualifié la sortie reste cohérente
de bout en bout, et perdre quinze secondes de parole est une défaillance bien
pire qu'un pronom moins bien relié.

**Portée strictement limitée à la passe de traduction.** La passe française ne
présente pas ce défaut et ses arguments sont ceux qualifiés depuis M2 ;
les modifier reviendrait à risquer une régression sur le cas d'usage principal
pour corriger un problème qu'il n'a pas. Un test l'impose dans les deux sens.

Le réglage n'élimine pas la classe de défaut : d'autres médias peuvent encore
déclencher une hallucination. Le traitement large (VAD, seuils, prompt initial)
reste le sujet de v0.2.

## Décision 6 — Contrat de nommage

| Sélection | Fichiers |
| --- | --- |
| Français seul | `IMG_8484.srt`, `IMG_8484.txt` |
| English seul | `IMG_8484.en.srt`, `IMG_8484.en.txt` |
| Français + English | `IMG_8484.fr.srt`, `IMG_8484.fr.txt`, `IMG_8484.en.srt`, `IMG_8484.en.txt` |

La version française ne devient `.fr.` que lorsqu'une version anglaise existe
pour la distinguer. Qualifier une version isolée serait du bruit, et casserait
les noms historiques sans contrepartie : un utilisateur qui met à jour et ne
touche à rien obtient exactement les fichiers qu'il obtenait avant.

Un test vérifie qu'aucune combinaison ne produit deux fichiers de même nom.

## Décision 7 — Atomicité stricte, aucun succès partiel

Toutes les passes écrivent dans le workspace temporaire. Le dossier de sortie
est créé **une seule fois**, après la dernière passe. Conséquence directe :
une annulation ou un échec pendant la traduction anglaise ne publie **rien**,
pas même la moitié française déjà calculée.

C'est un choix assumé : un dossier contenant la moitié de ce qui a été demandé
se lit comme un succès et ne l'est pas. Le retry relance le job complet.

## Conséquences

* Un utilisateur qui demande l'anglais télécharge 3,1 Go une fois et paie
  ~0,9× le temps réel sur la passe de traduction. C'est le prix de la qualité
  demandée, et il est explicite dans l'interface avant tout téléchargement.
* Le disque occupé passe de ~574 Mo à ~3,7 Go pour ces utilisateurs.
* La transcription française seule est **strictement inchangée** : même
  modèle, mêmes arguments, mêmes noms de fichiers, même vitesse.
* Deux modèles signifient deux états de gestionnaire, deux téléchargements
  possibles et deux fichiers `.download` à nettoyer au démarrage — tous
  traités.

## Réserves

* La qualité linguistique n'est **pas** déclarée parfaite : elle est déclarée
  utile et compréhensible, sur un échantillon. Le gate humain la qualifie.
* Le vocabulaire technique et les noms propres restent le chantier v0.2 ; la
  traduction hérite des mêmes limites que la transcription sur ce point.
* Endurance : qualifiée sur un échantillon de 3 minutes. Un test long
  bilingue reste à faire.

## Références

* [ADR-001 — moteur de transcription](ADR-001-transcription-engine.md) ;
* [ADR-004 — gestion et intégrité du modèle local](ADR-004-model-management.md) ;
* [ADR-005 — cycle de vie des jobs et annulation](ADR-005-runtime-lifecycle-and-cancellation.md) ;
* [ADR-008 — pipeline bilingue, constat initial](ADR-008-bilingual-output-pipeline.md) ;
* [ADR-009 — splashscreen et packaging](ADR-009-splashscreen-and-release-packaging.md).
