# ADR-008 — Pipeline de sortie bilingue (français + anglais)

Statut : `REJECTED (blocked by the pinned model)` — décidé en M9, à réexaminer en v0.2.

Date : 2026-08-12

Contexte : M9 (Bilingual Outputs, Animated Splashscreen & Release Packaging).

## Contexte

M9 devait permettre à un même job utilisateur de produire, à partir d'un seul
média source, la transcription française originale **et** une traduction
anglaise locale, avec publication atomique des fichiers sélectionnés.

L'architecture visée était :

```
FFmpeg (une fois)
  └─ audio.wav (une fois)
       ├─ Whisper passe 1 — transcribe, langue fr   → sorties FR
       └─ Whisper passe 2 — translate → anglais     → sorties EN
            └─ publication atomique (tout ou rien)
```

Les deux passes devaient être **séquentielles** : jamais deux `whisper-cli`
simultanés, un seul emplacement de job (garantie M4 conservée).

Le périmètre M9 imposait trois contraintes non négociables :

* ne pas mettre à jour whisper.cpp (v1.9.2 épinglé) ;
* ne pas changer de modèle (`ggml-large-v3-turbo-q5_0`) ;
* ne pas introduire un second moteur, ni aucune traduction distante.

## Audit préalable (exigé avant toute implémentation)

Conformément au protocole M9, la capacité de traduction a été vérifiée sur le
binaire réellement embarqué **avant** d'écrire la moindre ligne de pipeline.

### Le drapeau existe et est correctement transmis

`whisper-cli --help` (build épinglé, `whisper.cpp version: 1.9.2`) :

```
-tr,  --translate  [false] translate from source language to english
```

whisper.cpp confirme lui-même avoir accepté la tâche, dans son propre journal :

```
main: processing '…/fr_sample_16k_mono.wav' (2899264 samples, 181.2 sec),
      4 threads, 1 processors, 5 beams + best of 5,
      lang = fr, task = translate, timestamps = 1 ...
```

Le modèle se charge sans erreur ni avertissement :

```
whisper_model_load: n_vocab = 51866
whisper_model_load: type    = 5 (large v3)
```

### Le modèle épinglé ne réalise pas la tâche de traduction

Sur le fixture français qualifié (`spike/samples/fr_sample_16k_mono.wav`,
181 s), avec le binaire embarqué et la ligne de commande de production :

| Modèle | Arguments | Langue produite | Segments SRT |
| --- | --- | --- | --- |
| `ggml-large-v3-turbo-q5_0` (épinglé) | `-l fr` | français (attendu) | 88, timecodes réels |
| `ggml-large-v3-turbo-q5_0` (épinglé) | `-l fr -tr` | **français** (échec) | **14, blocs de 30 s exactement** |
| `ggml-small` (témoin, non-turbo) | `-l fr -tr` | **anglais correct** | 36, timecodes réels |

Extrait de la passe `-tr` sur le modèle épinglé — la sortie reste française :

```
[00:00:00.000 --> 00:00:30.000]  Bonjour à toutes et à tous. Aujourd'hui, je
voudrais vous parler d'un sujet très concret : comment l'intelligence
artificielle commence à s'intégrer dans notre quotidien…
```

Le même appel, seul le modèle changeant (témoin `ggml-small`) :

```
[00:00:00.000 --> 00:00:11.000]  Hello everyone, today I would like to talk to
you about a very concrete topic, how artificial intelligence begins to be
integrated in our daily life…
```

Variantes testées sur le modèle épinglé, toutes reproduisant la sortie
française : `-l auto -tr`, et `-l fr -tr -nf` (désactivation du repli de
température). Aucune combinaison d'arguments n'a produit d'anglais.

La dégénérescence en blocs de 30 secondes exactement — la fenêtre d'inférence
brute de Whisper, sans re-segmentation — est le symptôme caractéristique d'un
décodeur qui ne dispose pas de la tâche demandée : le modèle retombe sur la
transcription et perd la segmentation par énoncé.

Preuves conservées dans [`spike/out/m9-translation-audit/`](../../spike/out/m9-translation-audit/) :
sorties SRT des trois runs et journaux `stderr` (chemins développeur
remplacés, conformément à la politique M8).

### Cause

`large-v3-turbo` est un décodeur distillé (4 couches au lieu de 32) entraîné
par OpenAI sur des données de **transcription uniquement** ; la tâche de
traduction a été explicitement exclue de son entraînement. Le jeton de tâche
`<|translate|>` reste présent dans le vocabulaire — d'où l'absence d'erreur —
mais le modèle n'a pas appris à s'y conformer.

Ce n'est donc ni un défaut de notre build, ni un problème d'arguments, ni un
problème de quantification `q5_0` : c'est une propriété du modèle retenu en
M0B pour sa vitesse de transcription.

## Décision

**La sortie bilingue n'est pas implémentée en M9.**

Aucun pipeline à deux passes, aucune sélection de langue de sortie, aucun
renommage `.fr.` / `.en.`, aucune clé i18n associée n'a été introduite. Le
comportement de transcription française reste strictement celui qualifié en
M8 — M9 n'a apporté aucune modification au pipeline de transcription.

Les seules voies techniques vers une traduction anglaise sortent toutes du
périmètre M9 explicitement :

| Voie | Statut |
| --- | --- |
| Mettre à jour whisper.cpp | interdit (§4) — et sans effet : le blocage est le modèle |
| Changer le modèle unique | interdit (§4, §62) |
| Ajouter un second modèle multilingue non-turbo | interdit (§62) |
| Second moteur de traduction local | interdit (§4, §62) |
| API de traduction distante (DeepL, OpenAI, Anthropic) | interdit (§4, §62) et contraire au principe local-first |

Le protocole M9 §4 prévoyait exactement ce cas : « Si la traduction vers
l'anglais ne fonctionne pas correctement avec notre build actuel : STOP sur
cette partie. Ne pas bricoler une traduction cloud. » C'est la règle
appliquée. La décision de descope a été confirmée par l'auteur.

## Conséquences

* ST-IA 0.1.0 produit des sous-titres **français uniquement**, comme en M8.
  Le contrat de nommage historique (`IMG_8484.srt`, `IMG_8484.txt`) est
  inchangé, et aucune documentation utilisateur n'annonce de sortie anglaise.
* Aucune dette n'est introduite : plutôt qu'un pipeline multi-passes livré
  puis désactivé, aucune architecture spéculative n'a été écrite. Le pipeline
  reste mono-passe, c'est-à-dire exactement ce que le produit sait faire.
* La garantie « un seul job, un seul enfant `whisper-cli` » (M4) n'est pas
  remise en cause, puisque le nombre de passes reste à un.
* Le reste de M9 — splashscreen animé et packaging de release — a été livré
  intégralement ; ces sujets sont indépendants du moteur.

## Réexamen en v0.2

La traduction anglaise est reportée à la mission **v0.2 Transcription
Quality**, qui traitera de toute façon la question du modèle (vocabulaire,
noms propres, termes techniques — limites déjà relevées en M5).

Options à arbitrer à ce moment, avec leur coût réel :

1. **Second modèle dédié à la traduction** (`large-v3` complet, ~1,1 Go
   quantisé) téléchargé à la demande, uniquement si l'utilisateur active la
   sortie anglaise. Coût : un second téléchargement, un second gestionnaire
   de modèle, une empreinte disque doublée. C'est l'option la plus probable.
2. **Remplacer le modèle unique** par un `large-v3` non-turbo, qui sait
   transcrire *et* traduire. Coût : transcription française sensiblement plus
   lente — régression directe sur le cas d'usage principal. Peu souhaitable.
3. **Renoncer durablement** à la traduction et assumer ST-IA comme un outil
   de sous-titrage français. Option légitime si la demande ne se confirme pas.

Aucune de ces options n'est décidée ici ; elles sont documentées pour que le
réexamen reparte de l'audit ci-dessus plutôt que de le refaire.

## Références

* [ADR-001 — moteur de transcription](ADR-001-transcription-engine.md) (choix
  du modèle en M0B) ;
* [ADR-003 — pipeline de transcription local](ADR-003-local-transcription-pipeline.md) ;
* [ADR-005 — cycle de vie et annulation](ADR-005-runtime-lifecycle-and-cancellation.md) ;
* preuves : [`spike/out/m9-translation-audit/`](../../spike/out/m9-translation-audit/).
