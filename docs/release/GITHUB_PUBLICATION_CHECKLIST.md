# Checklist de publication GitHub (M9)

Ce document **recommande** ; il n'applique rien. M8 n'a modifié aucun paramètre GitHub
du dépôt, délibérément — cela demande une autorisation explicite.

À exécuter au moment de rendre le dépôt public, dans cet ordre.

---

## 1. Avant de basculer en public

- [x] **Licence choisie et committée** — **MIT** (`LICENSE` à la racine, champ `license`
      dans `package.json` et `Cargo.toml`, section dédiée du README). Vérifier après le
      passage en public que GitHub affiche bien « MIT license » sur la page du dépôt.
- [ ] `docs/security/M8_SECURITY_REVIEW.md` relu, aucun finding HIGH/CRITICAL ouvert
- [x] Décision prise sur STIA-SEC-103 (`IMG_8484.MOV` dans la doc) — **conservé**,
      fichier jamais committé, aucune action requise
- [ ] Décision prise sur l'adresse e-mail dans les métadonnées de commits
- [ ] `LEGAL_REVIEW_RECOMMENDED` (LGPL FFmpeg, STIA-SEC-202) traité ou accepté
- [ ] Rejouer `gitleaks git --log-opts="--all"` sur le `main` final → 0 finding
- [ ] Vérifier qu'aucune branche `feat/m*` non fusionnée ne contient autre chose que
      ce qui est dans `main` (elles deviendront publiques elles aussi)

## 2. Activer avant, pas après

Ces réglages doivent être en place **au moment** où le dépôt devient public.

Settings → Code security :

- [ ] **Private vulnerability reporting** — active le canal que `SECURITY.md` indique.
      Sans lui, le lien de `SECURITY.md` et de `ISSUE_TEMPLATE/config.yml` est mort.
- [ ] **Secret scanning** + **Push protection** — bloque un secret *avant* qu'il n'entre
      dans l'historique. C'est la seule protection qui agisse en amont plutôt qu'en
      constat.
- [ ] **Dependabot alerts** + **Dependabot security updates**
- [ ] Vérifier que `.github/dependabot.yml` est bien détecté (npm, cargo, actions)

## 3. Protection de branche

Settings → Rules → Rulesets, sur `main` :

- [ ] Interdire le push direct
- [ ] Exiger une pull request
- [ ] Exiger les status checks : `Frontend (build, types, tests)` et
      `Rust (fmt, clippy, tests)`
- [ ] Exiger que la branche soit à jour avant merge
- [ ] Bloquer les force-push
- [ ] Bloquer la suppression de branche

> Sur un projet à un seul mainteneur, l'auto-review n'apporte rien : les status checks
> sont la protection réelle. N'exigez une review que si un second mainteneur arrive.

## 4. Métadonnées du dépôt

- [ ] Description : « Générateur de sous-titres local pour macOS — audio/vidéo → SRT/TXT,
      sans cloud »
- [ ] Topics : `macos`, `tauri`, `rust`, `whisper`, `subtitles`, `srt`, `local-first`,
      `privacy`, `apple-silicon`, `ffmpeg`
- [ ] Désactiver Wiki et Projects s'ils ne sont pas utilisés
- [ ] Activer Discussions si vous voulez éviter que les questions arrivent en issues
- [ ] Vérifier que le README rend correctement en public (liens relatifs)

## 5. Après le passage en public

- [ ] Vérifier que CI et security.yml passent sur le dépôt public
      (`gitleaks-action` et `rustsec/audit-check` ont besoin de `GITHUB_TOKEN`, fourni
      automatiquement)
- [ ] Vérifier que le lien « Report a vulnerability » de `SECURITY.md` fonctionne
- [ ] Vérifier l'onglet Insights → Dependency graph
- [ ] Confirmer qu'aucune GitHub Action ne s'exécute automatiquement sur les PR de
      contributeurs externes sans approbation (Settings → Actions → *Require approval
      for all external contributors*)

## 6. Release 0.1.0 — hors périmètre de cette checklist

Ne pas taguer avant que tout ce qui précède soit fait. Voir
`docs/release/RELEASE_CHECKLIST.md`.

- [ ] Signature Developer ID + notarisation Apple
- [ ] Publier les SHA-256 du `.dmg` dans les notes de release
- [ ] Décider si les sidecars restent dans Git ou passent en artefacts de release
      (STIA-SEC-106)
