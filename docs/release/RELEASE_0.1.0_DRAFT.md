# Release 0.1.0 — contenu préparé (non publiée)

Statut : **préparation uniquement**. Aucun tag n'est créé, aucune release
GitHub n'existe, le repository n'est pas public, et rien n'a été téléversé.

Ce document décrit ce que *contiendrait* la release, pour que la publication
elle-même (M10) n'ait plus de décision à prendre.

## Bloquant avant publication

`PUBLIC_DISTRIBUTION_SIGNING_PENDING` — les artefacts ne sont ni signés
Developer ID ni notarisés. Sur une autre machine, Gatekeeper les refusera
avec « ST-IA est endommagé et ne peut pas être ouvert », ce qui est le pire
message possible pour un premier contact. **Ne pas publier en l'état.**

`FULL_DELTA_REVIEW_PENDING_M10` — le [delta de sécurité M9](../security/M9_SECURITY_DELTA.md)
est ciblé, pas exhaustif.

## Artefacts

Produits par `scripts/package-release.sh` dans `release-artifacts/`
(non commité) :

| Fichier | Rôle |
| --- | --- |
| `ST-IA-0.1.0-macos-arm64.dmg` | **artefact utilisateur recommandé** — image disque avec `ST-IA.app` et le raccourci `Applications` |
| `ST-IA-0.1.0-macos-arm64.app.zip` | artefact avancé — archive `ditto` de l'application seule |
| `SHA256SUMS.txt` | empreintes SHA-256, vérifiées à la génération |

GitHub ajoute automatiquement les archives `Source code (zip/tar.gz)`.

Plateforme : **macOS Apple Silicon (arm64) uniquement**. Pas d'Intel, pas de
Windows, pas de Linux, pas de `.pkg`.

## Notes de version (brouillon)

> **ST-IA 0.1.0** — première version publique.
>
> Générez des sous-titres à partir d'une vidéo ou d'un fichier audio,
> entièrement sur votre Mac. Aucun cloud, aucun compte, aucune donnée envoyée
> nulle part.
>
> * transcription **française** locale (whisper.cpp + Metal sur Apple Silicon) ;
> * sorties **SRT** et **TXT**, prêtes pour DaVinci Resolve ;
> * modèle téléchargé une seule fois, sur action explicite, vérifié par SHA-256 ;
> * interface française et anglaise, thème clair/sombre, réduction d'animations ;
> * annulation à tout moment, sans fichier partiel laissé derrière.
>
> **Installation** — ouvrez le `.dmg`, glissez ST-IA sur `Applications`. Au
> premier lancement, l'application télécharge son modèle de transcription
> (~574 Mo). Ensuite, tout fonctionne hors ligne.
>
> **Configuration requise** — macOS sur Apple Silicon (M1 ou plus récent).
>
> **Limites connues** — la transcription est qualifiée en français uniquement ;
> ST-IA ne traduit pas. Certains noms propres et termes techniques restent
> approximatifs (chantier v0.2).
>
> Vérifiez votre téléchargement avec `shasum -a 256 -c SHA256SUMS.txt`.

## Vérifications avant publication (M10)

* [ ] artefacts signés Developer ID et notarisés, ticket agrafé ;
* [ ] `spctl -a -vvv ST-IA.app` accepte le bundle ;
* [ ] installation vérifiée sur un Mac **autre** que la machine de build ;
* [ ] revue delta de sécurité complète ;
* [ ] [checklist de publication GitHub](GITHUB_PUBLICATION_CHECKLIST.md) passée ;
* [ ] tag `v0.1.0` créé sur le commit de release ;
* [ ] `CHANGELOG.md` : `[Unreleased]` promu en `[0.1.0]` daté.
