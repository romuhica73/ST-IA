# ST-IA — Open Source Readiness

Date : 2026-08-11
Périmètre : `feat/m8-open-source-security-readiness`
Question : *le dépôt peut-il être rendu public ?*

**Réponse : oui.** Le seul point qui bloquait — l'absence de licence principale — a été
tranché par l'auteur : ST-IA est sous **licence MIT**. Aucun blocker de sécurité ni de
confidentialité ne subsiste.

---

## Checklist

| # | Critère | État | Détail |
|---|---|---|---|
| 1 | Secrets dans HEAD | ✅ | `gitleaks` + grep manuel sur ~30 motifs → 0 |
| 2 | Secrets dans l'historique | ✅ | 43 commits scannés → 0. **Aucun fichier n'a jamais existé sans être dans HEAD** → pas de fichier supprimé où un secret pourrait subsister |
| 3 | Réécriture d'historique | ✅ **non requise** | `HISTORY_REWRITE_REQUIRED` : NON |
| 4 | Données privées / PII | ✅ | Chemins dev nettoyés dans HEAD ; `IMG_8484.MOV` (nom seul, fichier jamais committé) conservé — STIA-SEC-103 **ACCEPTED / NO ACTION REQUIRED** |
| 5 | **Licence principale** | ✅ **MIT** | `LICENSE` à la racine + `package.json` + `Cargo.toml` + README. Voir §Licence |
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
| 27 | Qualification humaine du `.app` | ✅ | Gate A/B/C/D passé sur la build empaquetée — voir `M8_SECURITY_REVIEW.md` §Qualification humaine |
| 28 | Privacy mesurée | ✅ | 0 connexion au repos, en réglages et en transcription ; téléchargement du modèle vers le seul endpoint attendu, 2 Ko sortants |

---

## Licence — MIT

**Décision de l'auteur : ST-IA est distribué sous licence MIT.**
`PROJECT_LICENSE_DECISION_REQUIRED` → **RESOLVED / MIT**.

### Ce qui a été appliqué

| Emplacement | Contenu |
|---|---|
| `LICENSE` | Texte MIT standard, `Copyright (c) 2026 Romain Bourbon`, aucune clause personnalisée |
| `package.json` | `"license": "MIT"` |
| `src-tauri/Cargo.toml` | `license = "MIT"` |
| `README.md` | Section « Licence », avec la distinction explicite code ST-IA / composants tiers |

GitHub détectera `LICENSE` et affichera « MIT license » sur la page du dépôt.

### Ce que MIT couvre — et ce qu'elle ne couvre pas

MIT couvre **le code de ST-IA uniquement** : le backend Rust, le frontend TypeScript,
les scripts et la documentation de ce dépôt.

Les composants tiers distribués avec l'application **conservent chacun leur licence
propre**, inchangée :

| Composant | Licence | Mode de distribution |
|---|---|---|
| FFmpeg 9.0 | **LGPL-2.1** | Exécutable séparé dans le bundle, non lié |
| whisper.cpp v1.9.2 | MIT (la sienne) | Exécutable séparé dans le bundle |
| Modèle Whisper | — | **Non redistribué** — téléchargé depuis Hugging Face par l'utilisateur |

`THIRD_PARTY_NOTICES.md` reste le document distinct qui les recense, et les textes de
licence restent embarqués dans `ST-IA.app/Contents/Resources/licenses/`.
**Rien n'a été relicencié.**

### Compatibilité

MIT est compatible avec les deux composants distribués. whisper.cpp est déjà MIT.
FFmpeg est en LGPL-2.1 mais distribué comme **exécutable séparé**, invoqué par création
de processus et jamais lié : la LGPL n'impose donc pas ses termes au code de ST-IA.

Cela ne clôt pas STIA-SEC-202 : la manière dont FFmpeg est distribué reste un point à
faire confirmer par un tiers qualifié (`LEGAL_REVIEW_RECOMMENDED`). C'est une question
indépendante du choix de licence de ST-IA, et ce n'est pas un défaut de sécurité.

---

## Risques résiduels à l'ouverture

### Sécurité
* Un média piégé exploitant un décodeur FFmpeg — surface minimale (5 demuxers,
  9 décodeurs), processus séparé sans réseau, mais pas de sandbox dédiée.
* Un frontend compromis peut transcrire un média que l'utilisateur possède et en lire
  le texte — **sans canal d'exfiltration** (CSP + FFmpeg sans réseau).

### Privacy
* `IMG_8484.MOV` (nom seul) dans la documentation — STIA-SEC-103, **accepté, clos**.
  Le fichier média n'a jamais été committé ; aucune donnée média personnelle n'est
  présente dans le dépôt ni dans son historique.
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
* Licence principale : **MIT, résolue** (STIA-SEC-201). Plus de blocker.
* Conformité LGPL de FFmpeg : **`LEGAL_REVIEW_RECOMMENDED`**, STIA-SEC-202 — ouvert,
  indépendant du choix de licence, non bloquant pour la sécurité.

### Distribution publique
* Pas de signature ni de notarisation Apple — les utilisateurs verront un
  avertissement Gatekeeper. Documenté dans QUICKSTART et BUILDING. **M9.**
* Aucun tag, aucune release, aucun binaire publié — conforme au mandat M8.

---

## Verdict

**Prêt techniquement, légalement et fonctionnellement.** Aucun secret, aucune
réécriture d'historique nécessaire, aucune vulnérabilité de dépendance, surface IPC
bornée et testée, CSP appliquée et vérifiée sur build empaquetée, licence principale
MIT en place.

La qualification humaine est passée sur le `.app` empaqueté (réglages, parcours
nominal, annulation/retry, téléchargement sécurisé du modèle), et la propriété
local-first n'est plus seulement déduite du code : elle est **mesurée**. Zéro connexion
réseau au repos, en réglages et pendant une transcription ; pour le téléchargement du
modèle, l'endpoint épinglé uniquement, en HTTPS, avec 2 144 octets sortants au total
contre 493 Mo entrants. Les sorties SRT/TXT sont identiques aux tailles qualifiées en
M2/M4 : aucun correctif M8 n'a modifié le produit.

Réserves restantes, aucune n'étant un blocker de sécurité ou de confidentialité :

* `PUBLIC_DISTRIBUTION_SIGNING_PENDING` — signature Developer ID et notarisation
  Apple, prévues pour M9 ;
* `LEGAL_REVIEW_RECOMMENDED` — mode de distribution de FFmpeg sous LGPL
  (STIA-SEC-202) ;
* reproductibilité des sidecars — deux binaires suivis dans Git, non vérifiables
  trivialement par un contributeur ; compromis documenté (STIA-SEC-106), à
  reconsidérer en M9.
