# ST-IA — Open Source Readiness

Date : 2026-08-11
Périmètre : `feat/m8-open-source-security-readiness`
Question : *le dépôt peut-il être rendu public ?*

**Réponse : techniquement oui, légalement pas encore.** Un seul point bloque, et ce
n'est pas un problème de sécurité — c'est une décision qui vous appartient : le projet
n'a pas de licence.

---

## Checklist

| # | Critère | État | Détail |
|---|---|---|---|
| 1 | Secrets dans HEAD | ✅ | `gitleaks` + grep manuel sur ~30 motifs → 0 |
| 2 | Secrets dans l'historique | ✅ | 43 commits scannés → 0. **Aucun fichier n'a jamais existé sans être dans HEAD** → pas de fichier supprimé où un secret pourrait subsister |
| 3 | Réécriture d'historique | ✅ **non requise** | `HISTORY_REWRITE_REQUIRED` : NON |
| 4 | Données privées / PII | ⚠️ mineur | Chemins dev nettoyés dans HEAD ; `IMG_8484.MOV` (nom seul, fichier jamais committé) laissé — décision utilisateur, STIA-SEC-103 |
| 5 | **Licence principale** | ❌ **BLOQUANT** | Aucun fichier `LICENSE`. Voir §Licence |
| 6 | Notices tierces | ✅ | `THIRD_PARTY_NOTICES.md`, `licenses/`, `docs/third-party/FFMPEG.md`, embarqués dans le bundle |
| 7 | Documentation de build | ✅ | `docs/BUILDING.md` — prérequis, versions, tests, rebuild des sidecars, dépannage |
| 8 | Quickstart utilisateur | ✅ | `docs/QUICKSTART.md` |
| 9 | Politique de sécurité | ✅ | `SECURITY.md` — private vulnerability reporting, périmètre, délais |
| 10 | Guide de contribution | ✅ | `CONTRIBUTING.md` |
| 11 | Threat model | ✅ | `docs/security/THREAT_MODEL.md` |
| 12 | CI | ✅ | `.github/workflows/ci.yml` — build, types, Vitest, fmt, clippy, cargo test |
| 13 | CI sécurité | ✅ | `.github/workflows/security.yml` — séparée du gate PR (advisories volatiles) |
| 14 | Dependabot | ✅ | npm + cargo + actions, hebdomadaire, groupé |
| 15 | Dépendances JS | ✅ | `pnpm audit` : 0 vulnérabilité / 165 paquets |
| 16 | Dépendances Rust | ✅ | `cargo audit` : 0 vulnérabilité / 505 crates ; 17 warnings analysés |
| 17 | Supply chain | ✅ | Lockfiles suivis, sources pinnées + checksums, aucun `curl \| sh`, aucune dépendance sur branche Git |
| 18 | CSP | ✅ | Restrictive, sans `unsafe-inline` ni `unsafe-eval`, vérifiée sur build empaquetée |
| 19 | Capabilities | ✅ | Déjà minimales, auditées, inchangées |
| 20 | Réseau / privacy | ✅ | Une seule sortie possible ; 0 socket au repos, vérifié |
| 21 | Sécurité du modèle | ✅ | URL épinglée à un commit immuable, SHA-256 + taille, promotion atomique, plafond de taille |
| 22 | Sidecars | ✅ | Provenance, versions, checksums, `otool -L` propre, invocation sans shell |
| 23 | `.gitignore` | ✅ | Renforcé ; **0 fichier suivi masqué** (prouvé) |
| 24 | Templates GitHub | ✅ | PR + issue + lien vulnérabilité |
| 25 | Paramètres GitHub | ⏸️ recommandés | Non appliqués par M8 (hors mandat) — `docs/release/GITHUB_PUBLICATION_CHECKLIST.md` |
| 26 | Signature / notarisation | ⏸️ M9 | Hors périmètre |

---

## Licence — décision requise

**C'est le seul point qui empêche de déclarer le dépôt open source.**

Un dépôt public sans fichier `LICENSE` reste sous **copyright exclusif**. Le code est
visible, mais personne n'a le droit de l'utiliser, de le modifier ou de le
redistribuer. GitHub l'affichera comme « No license ».

M8 **n'a pas choisi** de licence, délibérément. Voici les faits, sans recommandation.

### Contraintes qui s'appliquent à ST-IA

* **Le code de ST-IA lui-même** (Rust + TypeScript) est original : vous êtes libre de
  le licencier comme vous voulez.
* **FFmpeg (LGPL-2.1)** est distribué comme *exécutable séparé*, pas lié. La LGPL
  n'impose donc pas sa licence au code de ST-IA. Voir STIA-SEC-202 —
  `LEGAL_REVIEW_RECOMMENDED` avant une distribution publique large.
* **whisper.cpp (MIT)** est également un exécutable séparé, et MIT est compatible avec
  tout.
* **Le modèle Whisper** n'est pas redistribué : il est téléchargé depuis Hugging Face
  par l'utilisateur.

Aucune de ces dépendances ne vous **force** la main.

### Options courantes — implications factuelles

| Licence | Ce qu'elle permet à un tiers | Ce qu'elle exige en retour |
|---|---|---|
| **MIT** | Tout, y compris un produit commercial fermé | Conserver la notice de copyright |
| **Apache-2.0** | Idem MIT | Idem, + notice des modifications ; inclut une **concession explicite de brevets** et une clause de représailles |
| **GPL-3.0** | Utiliser, modifier, redistribuer | Toute redistribution, modifiée ou non, doit être sous GPL-3.0 avec les sources |
| **AGPL-3.0** | Idem GPL-3.0 | Idem, **plus** : fournir les sources aussi en cas d'usage réseau |
| Aucune | Rien légalement | — |

Quelques éléments factuels, sans prendre parti :

* MIT et Apache-2.0 maximisent l'adoption et n'empêchent pas une reprise commerciale
  fermée.
* GPL-3.0 empêche cette reprise fermée, au prix d'une adoption plus faible et d'une
  incompatibilité avec certains écosystèmes.
* AGPL-3.0 n'apporte rien de plus ici : ST-IA est une application de bureau locale,
  sans usage réseau, donc la clause distinctive de l'AGPL ne se déclencherait jamais.
* Apache-2.0 est la seule des quatre à traiter explicitement des brevets.

**Aucune ne peut être choisie à votre place.** Dites laquelle et j'ajoute le `LICENSE`,
le champ `license` de `package.json`, celui de `Cargo.toml`, et l'en-tête du README dans
un commit dédié.

---

## Risques résiduels à l'ouverture

### Sécurité
* Un média piégé exploitant un décodeur FFmpeg — surface minimale (5 demuxers,
  9 décodeurs), processus séparé sans réseau, mais pas de sandbox dédiée.
* Un frontend compromis peut transcrire un média que l'utilisateur possède et en lire
  le texte — **sans canal d'exfiltration** (CSP + FFmpeg sans réseau).

### Privacy
* `IMG_8484.MOV` (nom seul) dans la documentation — STIA-SEC-103.
* L'adresse e-mail de l'auteur est dans les métadonnées des commits. C'est le
  fonctionnement normal de Git et elle est **déjà publique** via les 6 PR fusionnées de
  ce dépôt. Si vous ne le souhaitez pas, cela demande une réécriture d'historique —
  aucune n'est nécessaire pour la sécurité.
* Chemin de développeur dans le binaire FFmpeg distribué — STIA-SEC-104.

### Supply chain
* Deux sidecars binaires suivis dans Git, non vérifiables trivialement par un
  contributeur — STIA-SEC-106. Compromis assumé et documenté.
* 17 warnings RustSec `unmaintained`, aucun atteignable sur macOS.

### Légal / licence
* **Pas de licence principale** — bloquant, STIA-SEC-201.
* Conformité LGPL de FFmpeg : `LEGAL_REVIEW_RECOMMENDED`, STIA-SEC-202.

### Distribution publique
* Pas de signature ni de notarisation Apple — les utilisateurs verront un
  avertissement Gatekeeper. Documenté dans QUICKSTART et BUILDING. **M9.**
* Aucun tag, aucune release, aucun binaire publié — conforme au mandat M8.

---

## Verdict

**Prêt techniquement.** Aucun secret, aucune réécriture d'historique nécessaire, aucune
vulnérabilité de dépendance, surface IPC bornée et testée, CSP appliquée et vérifiée sur
build empaquetée, propriété local-first prouvée empiriquement.

**Pas encore déclarable open source** tant que la licence principale n'est pas choisie.

La qualification produit interactive sur le `.app` (Réglages, FR/EN, thème,
transcription réelle, SRT/TXT, Finder, annulation, retry) reste un **gate humain** :
elle demande un humain devant l'application.
