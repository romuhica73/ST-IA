# Prochaine mission — v0.2 Transcription Quality

Statut : **proposition**, non engagée. À arbitrer après clôture de M9.

Ce document existe pour que la prochaine mission reparte des mesures déjà
faites plutôt que de les refaire. Il ne décide de rien.

## Ce qui est déjà clos et ne doit pas être rouvert

M9 a livré et qualifié : sortie bilingue FR/EN locale, second modèle dédié à
la traduction, splashscreen, packaging de release, transparence des modèles,
progression fiable. Le correctif de contrôle d'accès applicatif (ACL de
commandes) est en place et testé.

Reste hors périmètre de M9, par décision explicite : signature, notarisation,
tag, GitHub Release. C'est **M10**.

## Le sujet principal : la qualité de transcription

Trois défauts sont documentés, mesurés, et non résolus.

### 1. Vocabulaire technique et noms propres

Observé depuis M5, jamais traité. Exemples reproductibles sur le média
qualifié : « Claude Code » → « Cloud Code », « Rust » → « REST »,
« Anthropic » → « Anthropik », « Apple Silicon M4 » → « Nynpus, Apple,
Silicon ou M4 ».

Pistes, par coût croissant :

* **prompt initial** (`--prompt`) avec un vocabulaire de domaine. Peu coûteux,
  effet réel sur Whisper, mais borné à `n_text_ctx/2` tokens et sensible :
  un prompt mal calibré dégrade le reste. À mesurer, pas à supposer.
* **vocabulaire utilisateur** persistant, alimenté depuis l'interface. Suppose
  une UI, un stockage et une politique de portée (par projet ? global ?).
* **post-correction par dictionnaire** sur les sorties. Simple, mais casse la
  propriété actuelle « les fichiers sont la copie exacte de ce que produit le
  moteur » — à peser sérieusement avant de l'abandonner.

### 2. Hallucinations et boucles de décodage

M9 a réduit le phénomène sur la passe de traduction avec `-mc 0`
(11 répétitions → 1, ~15 s de discours restituées). **La classe de défaut
subsiste** : d'autres médias peuvent la déclencher, sur l'une ou l'autre passe.

Pistes :

* évaluer `-mc 0` sur la **passe française** — non fait en M9 pour ne pas
  toucher un comportement qualifié sans preuve d'un problème ;
* VAD (détection d'activité vocale) pour ne pas décoder du silence, principale
  source d'hallucination ;
* seuils `--entropy-thold` / `--logprob-thold` / `--no-speech-thold` —
  `-et 2.8` a été mesuré sans effet, les autres non ;
* `-sns` (suppression des tokens non-verbaux) : mesuré, supprime les cues de
  durée nulle sans casser les boucles. Peut compléter `-mc 0`.

Corpus de test à constituer : le média qualifié seul ne suffit plus.

### 3. Endurance bilingue

Jamais mesurée sur un média long. La transcription seule est qualifiée
jusqu'à 60 min (M5) ; la traduction tourne à ~0,9× le temps réel, donc un
média d'une heure représente environ une heure de traitement supplémentaire.
À mesurer : temps, RAM, stabilité, comportement de l'annulation en fin de
passe longue.

## Sujets secondaires identifiés

* **Segments de durée nulle** — apparaissent encore ponctuellement. Sans
  impact sur la validité SRT ni sur DaVinci, mais inélégants.
* **Choix de la langue source** — seul le français est qualifié. Élargir
  suppose un corpus par langue.
* **Seuil de la mention « certains passages demandent plus de calcul »** —
  fixé à 12 s d'après une mesure sur une machine. À revalider sur du matériel
  plus lent.

## Ce qu'il ne faut pas faire

* traiter la qualité par une correction LLM en aval : hors principe
  local-first et hors périmètre produit ;
* toucher aux paramètres de la passe française sans mesure comparative ;
* remplacer le modèle de transcription — arbitré en M9, coût mesuré à 7,5× ;
* ajouter un service distant de quelque nature que ce soit.

## Références

* [ADR-001](../architecture/ADR-001-transcription-engine.md) — choix du moteur
* [ADR-010](../architecture/ADR-010-local-english-translation.md) — traduction locale, `-mc 0`
* [AI_MODELS.md](../AI_MODELS.md) — limites connues, formulation utilisateur
* [`spike/out/m9-english-repetition/`](../../spike/out/m9-english-repetition/) — variantes mesurées
