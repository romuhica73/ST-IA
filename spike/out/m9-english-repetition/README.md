# Répétition anormale en traduction anglaise — investigation et correctif

Média : `IMG_8484.MOV` (3 min, français), modèle `large-v3`, whisper.cpp
v1.9.2, tâche `translate`.

## Symptôme

Avec les arguments d'origine, la sortie anglaise contient 11 répétitions
consécutives d'une même phrase entre 00:01:25,8 et 00:01:30,7, avec des
horodatages de durée nulle.

**Le plus grave n'est pas la répétition elle-même** : pendant ces ~15
secondes, le discours réellement prononcé — « Tout doit être traité localement
sur mon MacBook Pro […] La vidéo ne doit jamais être envoyée vers OpenAI,
Anthropik » — a été **perdu** et remplacé par la phrase répétée.

## Origine

Pas le pipeline ST-IA. Les fichiers publiés sont une copie octet pour octet de
la sortie de whisper.cpp (vérifié en comparant les SHA-256), et ST-IA ne
parse, ne fusionne ni ne réécrit aucun segment.

Le déclencheur est le contexte textuel que whisper.cpp reporte d'une fenêtre
de décodage à la suivante (`--max-context`, par défaut `-1`, non borné) :
lorsqu'une phrase se répète, sa propre répétition devient le contexte qui la
fait se répéter à nouveau.

## Variantes testées

| Arguments | Cues | Plus longue répétition | Cues dupliquées | Cues de durée nulle | Temps |
| --- | --- | --- | --- | --- | --- |
| défaut (`-mc -1`) | 41 | **11** | 12 | 14 | 96,5 s |
| `-et 2.8` | 41 | 11 | 12 | 14 | 93,8 s |
| `-sns` | 42 | 11 | 12 | 0 | 52,0 s |
| **`-mc 0`** | 36 | **1** | **0** | **0** | **38,0 s** |
| `-mc 0 -sns` | 42 | 1 | 0 | 0 | 35,9 s |

`-et 2.8` (seuil d'entropie plus agressif) ne change rien : sortie identique.
`-sns` supprime les cues dégénérées mais pas la boucle.

## Correctif retenu

`-mc 0` sur **la passe de traduction uniquement**.

Ce n'est pas une déduplication a posteriori : la boucle est empêchée, elle
n'est pas masquée. Aucun segment identique n'est supprimé, donc une
répétition légitime du discours reste intacte.

Vérifié : le passage perdu est correctement traduit dans
`large-v3-translate-mc0.srt`, et conforme au français de référence.

Contrepartie : moins de cohérence à longue portée entre fenêtres. Sur
l'échantillon qualifié la sortie reste cohérente de bout en bout, et perdre
quinze secondes de discours est une défaillance bien pire qu'un pronom moins
bien relié.

La passe française n'est pas modifiée : elle ne présente pas ce défaut, et ses
arguments sont ceux qualifiés depuis M2.

## Fichiers

* `large-v3-translate-raw.srt` — sortie d'origine, avec la boucle ;
* `repetition-region.srt` — la zone affectée, isolée ;
* `large-v3-translate-mc0.srt` — sortie après correctif ;
* `command.txt` — la ligne de commande d'origine et ses paramètres.
