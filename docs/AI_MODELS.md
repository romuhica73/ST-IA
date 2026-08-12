# Modèles IA utilisés par ST-IA

Ce document décrit **factuellement** les modèles d'intelligence artificielle
que ST-IA exécute, leur provenance, et où le traitement a lieu.

Ce n'est **pas** une attestation de conformité. ST-IA ne revendique aucune
certification, ni au titre de l'AI Act ni à aucun autre. Ce document existe
parce qu'un utilisateur a le droit de savoir quel modèle traite ses données et
où — pas pour cocher une case réglementaire.

Les valeurs ci-dessous sont la copie d'une source de vérité unique
(`src-tauri/src/domain/model.rs`). L'application les affiche dans
**Réglages → Modèles IA** en les lisant depuis cette même source, et refuse
tout fichier dont la taille et l'empreinte ne correspondent pas exactement.

---

## Vue d'ensemble

| | Transcription | Traduction |
| --- | --- | --- |
| Identifiant | `large-v3-turbo-q5_0` | `large-v3` |
| Fichier | `ggml-large-v3-turbo-q5_0.bin` | `ggml-large-v3.bin` |
| Rôle | transcription française | traduction français → anglais |
| Taille | 574 041 195 o (~547 Mo) | 3 095 033 483 o (~2,9 Gio) |
| Quantisé | oui (`q5_0`) | non |
| Requis | pour tout traitement | seulement si la version anglaise est demandée |
| Inclus dans l'application | **non** | **non** |
| Téléchargé à la demande | oui | oui |
| Exécution | locale | locale |
| Réseau pendant le traitement | **aucun** | **aucun** |

---

## Modèle de transcription

**Identifiant** : `large-v3-turbo-q5_0`
**Fichier** : `ggml-large-v3-turbo-q5_0.bin`
**Taille** : 574 041 195 octets
**SHA-256** : `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2`

**Rôle** : produire la transcription française d'un média audio ou vidéo, sous
forme de fichiers SRT et TXT.

**Origine** : modèle Whisper d'OpenAI, converti au format GGML par le projet
[ggerganov/whisper.cpp](https://github.com/ggerganov/whisper.cpp).

**Source épinglée** : `huggingface.co/ggerganov/whisper.cpp` au commit
`5359861c739e955e79d9a303bcbc70fb988958b1`. Un commit, jamais la branche
`main` — un pointeur de branche peut être redirigé vers un autre fichier.

**Moteur d'exécution** : whisper.cpp v1.9.2, compilé pour Apple Silicon avec
le backend Metal, embarqué comme exécutable séparé dans l'application.

**Choix** : retenu en mission 0B pour sa vitesse (~0,12× le temps réel) à
qualité de transcription française acceptable. Voir
[ADR-001](architecture/ADR-001-transcription-engine.md).

---

## Modèle de traduction

**Identifiant** : `large-v3`
**Fichier** : `ggml-large-v3.bin`
**Taille** : 3 095 033 483 octets
**SHA-256** : `64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2`

**Rôle** : produire une traduction anglaise à partir de l'audio français, sous
forme de fichiers SRT et TXT.

**Origine** et **source épinglée** : identiques au modèle de transcription
(même dépôt, même commit vérifié).

**Moteur d'exécution** : le même binaire whisper.cpp v1.9.2.

**Pourquoi un second modèle** : `large-v3-turbo` est un décodeur distillé
entraîné sur des données de transcription uniquement. Il accepte le drapeau de
traduction sans erreur et **renvoie du français**. Mesuré, avec un témoin :
voir [ADR-008](architecture/ADR-008-bilingual-output-pipeline.md) puis
[ADR-010](architecture/ADR-010-local-english-translation.md).

**Pourquoi non quantisé** : la traduction est l'étape où la dégradation de
quantisation se voit le plus. La décision produit est la qualité d'abord.

---

## Où le traitement a lieu

```
votre média
   └─ FFmpeg (local)      → audio.wav temporaire
        └─ whisper.cpp (local, Metal)
             └─ SRT + TXT à côté de votre média
```

Rien de tout cela ne quitte votre Mac. Aucun appel d'inférence distant
n'existe dans le code : le moteur est un exécutable embarqué, lancé sur votre
machine, et il n'y a aucun point de terminaison d'inférence à appeler.

**La seule connexion réseau du produit** est le téléchargement d'un modèle,
déclenché par un clic explicite. Il envoie une requête GET et ne transmet ni
média, ni transcription, ni nom de fichier. Après ce téléchargement,
l'application fonctionne entièrement hors ligne — coupez le Wi-Fi et vérifiez.

## Intégrité

Un modèle n'est jamais utilisé sur la seule foi de son nom de fichier. Au
téléchargement comme à chaque détection, sa **taille exacte** et son
**SHA-256** doivent correspondre au manifeste épinglé. Le téléchargement passe
par un fichier temporaire, n'est promu au nom définitif qu'après validation, et
est plafonné en taille pendant le flux. Voir
[ADR-004](architecture/ADR-004-model-management.md).

## Ce que produisent les sorties

Les fichiers SRT et TXT sont écrits **tels que whisper.cpp les produit**,
copiés octet pour octet depuis l'espace de travail temporaire. ST-IA ne
réécrit, ne fusionne, ne reformate et ne corrige aucun segment.

## Limites connues

**Vocabulaire technique et noms propres.** Les termes techniques, les noms de
produits et les noms propres sont fréquemment approximés
(« Claude Code » → « Cloud Code », « Rust » → « REST »). Cela concerne la
transcription comme la traduction. Chantier de la version 0.2.

**Répétitions occasionnelles à la traduction.** Le modèle `large-v3` peut
entrer dans une boucle de décodage et répéter la même phrase plusieurs fois
d'affilée, avec des horodatages dégénérés (durée nulle ou quasi nulle).
Observé et reproduit sur le média qualifié : 11 répétitions consécutives entre
00:01:25,8 et 00:01:30,7, et 3 entre 00:00:56,08 et 00:00:56,10. Preuves dans
[`spike/out/m9-english-repetition/`](../spike/out/m9-english-repetition/).

C'est une **limite du modèle**, pas un défaut du pipeline : le phénomène est
présent dans la sortie native de whisper.cpp, et les fichiers publiés en sont
la copie exacte (vérifié par comparaison d'empreintes). Le repli de
température de Whisper — son mécanisme intégré contre ces boucles — est déjà
actif, ST-IA ne le désactive pas.

Aucune déduplication automatique n'est appliquée : supprimer des segments
identiques sans comprendre le contexte risquerait d'effacer des répétitions
légitimes du discours. Le réglage anti-hallucination (VAD, seuils, prompt
initial) appartient à la mission v0.2 Transcription Quality.

**Langue source.** Seul le français est qualifié, en entrée comme point de
départ de la traduction.

**Qualité de traduction** : jugée **utile, pas parfaite**. Le sens général est
préservé et le résultat est exploitable ; certaines tournures restent
littérales.

## Transparence et gouvernance

* **Aucun modèle n'est entraîné, affiné ou modifié** par ST-IA. Les poids sont
  téléchargés tels quels et vérifiés.
* **Aucune donnée utilisateur ne sert à entraîner quoi que ce soit.** ST-IA
  n'a ni compte, ni télémétrie, ni serveur.
* **Aucune décision automatisée** n'est prise sur des personnes. La sortie est
  un texte que l'utilisateur relit et corrige.
* **Le résultat doit être relu.** Une transcription automatique n'est pas une
  transcription certifiée, et ne convient pas telle quelle à un usage légal,
  médical ou officiel.
* **Licences** : le code de ST-IA est sous MIT ; whisper.cpp est sous MIT ;
  FFmpeg est sous LGPL-2.1 et distribué comme exécutable séparé. Les poids
  Whisper sont publiés par OpenAI sous licence MIT. Voir
  [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

## Références

* [ADR-001 — moteur de transcription](architecture/ADR-001-transcription-engine.md)
* [ADR-004 — gestion et intégrité du modèle local](architecture/ADR-004-model-management.md)
* [ADR-008 — mesure sur le modèle turbo](architecture/ADR-008-bilingual-output-pipeline.md)
* [ADR-010 — traduction anglaise locale](architecture/ADR-010-local-english-translation.md)
* [Modèle de menace](security/THREAT_MODEL.md)
