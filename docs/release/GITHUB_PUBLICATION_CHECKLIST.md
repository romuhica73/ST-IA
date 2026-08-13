# Checklist de publication GitHub

Mise à jour par M10 (*Community Public Release Readiness*), sur la base
`9e55c65`. Remplace la version M9.

Ce document **recommande** ; il n'applique rien. **M10 n'a modifié aucun
paramètre GitHub** — cela demande une autorisation explicite, et c'est le rôle
de M11.

À exécuter au moment de rendre le dépôt public, dans cet ordre.

---

## 1. Avant de basculer en public

### Acquis — vérifiés par M10

- [x] **Licence choisie et committée** — **MIT** (`LICENSE`, champ `license`
      dans `package.json` et `Cargo.toml`). Vérifier après le passage en public
      que GitHub affiche bien « MIT license ».
- [x] **Aucun secret dans l'historique** — `gitleaks` sur `--all --full-history`
      (90 commits) : *no leaks found*. Trois balayages manuels indépendants
      confirment : aucun token, clé privée, certificat ou `.env` n'a jamais
      existé sur aucune référence.
- [x] **Aucun modèle Whisper dans Git** — le plus gros blob de tout
      l'historique fait 3,4 Mo (le sidecar FFmpeg). `.git` pèse 14 Mo.
- [x] **Aucun média privé publié** — `mockups/` et `test-media/` sont ignorés ;
      seuls les échantillons JFK (domaine public) sont suivis.
- [x] **Toutes les branches distantes auditées** — voir
      [`PUBLIC_REPOSITORY_REFS_REVIEW.md`](PUBLIC_REPOSITORY_REFS_REVIEW.md).
      Aucune ne contient de secret ni de contenu inattendu.
- [x] **Build depuis un clone propre qualifiée** — clone `origin` → 
      `pnpm install --frozen-lockfile` → tests → `pnpm tauri build` → `.app` et
      `.dmg`. Aucune dépendance cachée.
- [x] **Frontière commerciale décidée** —
      [ADR-012](../architecture/ADR-012-community-commercial-boundary.md).
      Aucun code de licensing, paiement ou gating dans le dépôt.
- [x] **Décision sur STIA-SEC-103** (`IMG_8484.MOV`) — conservé, fichier jamais
      committé.

### Décisions humaines encore ouvertes

- [x] ✅ **Métadonnées d'auteur — remédiée.** L'historique a été réécrit sur
      décision humaine : toutes les branches destinées à la publication portent
      désormais `studio@romain-bourbon.com`. Arbres, noms, dates, messages et
      topologie sont inchangés. **Reste à faire avant le passage en public** :
      fermer et supprimer les branches et PR Dependabot, dont l'ascendance
      porte encore l'ancienne adresse.
- [x] **Transcriptions de la voix de l'auteur** (`spike/out/fr-*`,
      `spike/out/m9-*`) — conservées. Script de démonstration écrit pour le
      test, ne nomme personne.
- [ ] **`LEGAL_REVIEW_RECOMMENDED`** (LGPL FFmpeg, STIA-SEC-202) — traité ou
      accepté explicitement. Ne bloque pas la publication du **source** ; à
      trancher avant toute distribution binaire officielle.
- [ ] **Branches `feat/m*`** — à supprimer **après** le tag `v0.1.0` et
      **avant** le passage en public (décision humaine M10). Elles ont été
      réécrites et ne portent plus l'ancienne adresse.
- [x] **PR et branches Dependabot** — fermées et supprimées **par Dependabot
      lui-même** lors de la réécriture de la base, les branches étant devenues
      non fusionnables. Aucune suppression manuelle. Dependabot recréera les
      mises à jour encore pertinentes.
- [ ] 🔴 **BLOQUANT — références de pull request.** Les quinze
      `refs/pull/N/head` pointent toujours vers les commits d'avant réécriture,
      et un `git fetch origin <ancien-SHA>` **réussit encore**. L'ancienne
      adresse reste donc récupérable dans le dépôt. Trancher entre : demander
      une purge à GitHub Support, publier depuis un dépôt neuf, ou accepter le
      résidu. Voir [`COMMUNITY_PUBLIC_READINESS.md`](COMMUNITY_PUBLIC_READINESS.md).
      **Ne pas rendre le dépôt public avant cet arbitrage.**
- [ ] Rejouer `gitleaks git --log-opts="--all"` sur le `main` final → 0 finding.

## 2. Activer avant, pas après

Ces réglages doivent être en place **au moment** où le dépôt devient public.

Settings → Code security :

- [ ] **Private vulnerability reporting** — active le canal que `SECURITY.md`
      indique. Sans lui, les liens de `SECURITY.md`, du README et de
      `ISSUE_TEMPLATE/config.yml` pointent vers une 404.
- [ ] **Secret scanning** + **Push protection** — bloque un secret *avant* qu'il
      n'entre dans l'historique. C'est la seule protection en amont plutôt qu'en
      constat. M10 a sorti le scan `gitleaks` de `security.yml` vers
      `secret-scan.yml` pour qu'il tourne sur **toute** PR ; Push Protection
      reste le complément indispensable.
- [ ] **Dependabot alerts** + **Dependabot security updates**
- [ ] Vérifier que `.github/dependabot.yml` est bien détecté (npm, cargo, actions)

## 3. Protection de branche

Settings → Rules → Rulesets, sur `main` :

- [ ] Interdire le push direct
- [ ] Exiger une pull request
- [ ] Exiger les status checks : `Frontend (build, types, tests)`,
      `Rust (fmt, clippy, tests)` et `gitleaks (full history)`
- [ ] Exiger que la branche soit à jour avant merge
- [ ] Bloquer les force-push
- [ ] Bloquer la suppression de branche

> Sur un projet à un seul mainteneur, l'auto-review n'apporte rien : les status
> checks sont la protection réelle. N'exigez une review que si un second
> mainteneur arrive.

## 4. Métadonnées du dépôt

- [ ] Description : « Local-first subtitle generator for macOS — audio/video to
      SRT/TXT, no cloud »
- [ ] Topics : `macos`, `tauri`, `rust`, `whisper`, `subtitles`, `srt`,
      `local-first`, `privacy`, `apple-silicon`, `ffmpeg`
- [ ] Désactiver Wiki et Projects s'ils ne sont pas utilisés
- [ ] Activer Discussions si vous voulez éviter que les questions arrivent en
      issues
- [ ] Vérifier que le README rend correctement en public (liens relatifs)
- [ ] *(optionnel)* Ajouter un `CODE_OF_CONDUCT.md` — absent aujourd'hui.
      GitHub le signalera comme manquant dans les *community standards*. À
      arbitrer : utile si des contributions externes sont attendues, purement
      formel sur un projet solo.

## 5. Après le passage en public

- [ ] Vérifier que `ci.yml`, `security.yml` et `secret-scan.yml` passent sur le
      dépôt public (`gitleaks-action` et `rustsec/audit-check` utilisent le
      `GITHUB_TOKEN` fourni automatiquement)
- [ ] Vérifier que le lien « Report a vulnerability » fonctionne réellement
- [ ] Vérifier l'onglet Insights → Dependency graph
- [ ] Confirmer qu'aucune GitHub Action ne s'exécute automatiquement sur les PR
      de contributeurs externes sans approbation (Settings → Actions →
      *Require approval for all external contributors*)

## 6. Release 0.1.0 — hors périmètre de cette checklist

Ne pas taguer avant que tout ce qui précède soit fait. Voir
[`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md).

- [ ] Signature Developer ID + notarisation Apple —
      **`APPLE_DEVELOPER_ID_NOT_AVAILABLE`** aujourd'hui. Bloque la
      distribution binaire officielle, **pas** la publication du source.
- [ ] Publier les SHA-256 du `.dmg` dans les notes de release
- [ ] Décider si les sidecars restent dans Git ou passent en artefacts de
      release (STIA-SEC-106)
