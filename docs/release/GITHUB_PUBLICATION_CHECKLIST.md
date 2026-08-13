# Checklist de publication GitHub — dépôt Community neuf

Mise à jour par M10 après l'arbitrage du mode de publication. Remplace la
version M9.

Ce document **recommande** ; il n'applique rien. **M10 n'a créé aucun dépôt,
modifié aucun paramètre GitHub, créé aucun tag et supprimé aucune branche.**
C'est le rôle de M11.

---

## 0. Le point de départ : deux dépôts

L'historique des commits a été réécrit en M10 pour retirer une adresse
professionnelle. Un force-push ne supprimant pas les objets, le dépôt actuel
conserve l'ancien historique dans ses `refs/pull/*` — vérifié, pas supposé.

D'où la décision humaine `FRESH_REPOSITORY_ACCEPTED` :

| Dépôt | Rôle | Visibilité |
|---|---|---|
| `romuhica73/ST-IA` (actuel) | **archive de développement** — historique des PR des missions M1 à M10 | **privé, définitivement** |
| **Nouveau dépôt** ST-IA Community | ce qui est publié | privé à la création, **public** après la checklist |

**Ne jamais rendre le dépôt actuel public.** Ne pas demander de purge à GitHub
Support : l'option retenue ne dépend d'aucun tiers.

---

## 1. Créer et amorcer le dépôt neuf

- [ ] Créer le dépôt sur GitHub, **vide** : pas de README, pas de `.gitignore`,
      pas de licence auto-générée — ils entreraient en conflit avec l'historique
      poussé.
- [ ] Le créer **privé**, et ne le passer public qu'après la section 3.
- [ ] Depuis un clone propre du dépôt actuel, positionné sur le `main` réécrit :

```sh
git remote add community git@github.com:<compte>/<nouveau-dépôt>.git
git push community main
```

- [ ] **Ne pousser que `main`.** Les branches `feat/m*` sont des jalons
      historiques : leur place est dans l'archive privée, pas dans un dépôt neuf
      dont l'intérêt est précisément de ne contenir que l'historique propre.
- [ ] Vérifier immédiatement après le push, sur le nouveau dépôt :

```sh
git ls-remote community 'refs/pull/*'          # doit être vide
git log --all --format='%ae %ce' | sort -u     # studio@romain-bourbon.com uniquement
```

- [ ] Mettre à jour l'URL du dépôt dans les fichiers qui la citent —
      `package.json` (`repository`, `homepage`, `bugs`), `src-tauri/Cargo.toml`
      (`repository`), `README.md`, `SECURITY.md`,
      `.github/ISSUE_TEMPLATE/config.yml`, `docs/BUILDING.md`. Le test
      `version_consistency` ne couvre pas ces champs : les vérifier à la main.

## 2. Avant de basculer en public

### Acquis — vérifiés par M10

- [x] **Licence MIT** — `LICENSE` standard non modifié, champ `license`
      cohérent dans `package.json` et `Cargo.toml`.
- [x] **Aucun secret dans l'historique** — `gitleaks` sur l'historique réécrit :
      *no leaks found*. Trois balayages manuels indépendants confirment.
- [x] **Métadonnées d'auteur** — historique réécrit ;
      `studio@romain-bourbon.com` est la seule identité sur toutes les branches.
- [x] **Aucun modèle Whisper dans Git** — plus gros blob de l'historique : 3,4 Mo.
- [x] **Aucun média privé publié** — `mockups/` et `test-media/` ignorés ; seuls
      les échantillons JFK (domaine public) sont suivis.
- [x] **Toutes les références auditées** —
      [`PUBLIC_REPOSITORY_REFS_REVIEW.md`](PUBLIC_REPOSITORY_REFS_REVIEW.md).
- [x] **Build depuis un clone propre qualifiée** — install → tests → `.app` et
      `.dmg`, aucune dépendance cachée.
- [x] **Frontière commerciale décidée** —
      [ADR-012](../architecture/ADR-012-community-commercial-boundary.md)
      `ACCEPTED`. Aucun code de licensing, paiement ou gating.
- [x] **Transcriptions de la voix de l'auteur** — conservées, acceptées.
- [x] **STIA-SEC-103** (`IMG_8484.MOV`) — fichier jamais committé.

### À faire sur le dépôt neuf

- [ ] Rejouer `gitleaks git --log-opts="--all"` sur le nouveau dépôt → 0 finding.
- [ ] Rejouer un clone propre → `pnpm install --frozen-lockfile` →
      `pnpm tauri build`, depuis le nouveau dépôt cette fois.
- [ ] Vérifier que le README rend correctement (liens relatifs, tableaux).
- [ ] `LEGAL_REVIEW_RECOMMENDED` (LGPL FFmpeg, STIA-SEC-202) — réserve
      maintenue. **Ne bloque pas la publication du source** ; à trancher avant
      toute distribution binaire officielle.

## 3. Activer avant le passage en public, pas après

Settings → Code security, sur le **nouveau** dépôt :

- [ ] **Private vulnerability reporting** — sans lui, les liens de
      `SECURITY.md`, du README et de `ISSUE_TEMPLATE/config.yml` pointent vers
      une 404.
- [ ] **Secret scanning** + **Push protection** — la seule protection qui agisse
      *avant* qu'un secret n'entre dans l'historique. M10 a sorti le scan
      `gitleaks` vers `secret-scan.yml` pour qu'il tourne sur **toute** PR ;
      Push Protection reste le complément indispensable.
- [ ] **Dependabot alerts** + **security updates**
- [ ] Vérifier que `.github/dependabot.yml` est détecté (npm, cargo, actions)

## 4. Protection de branche

Settings → Rules → Rulesets, sur `main` :

- [ ] Interdire le push direct ; exiger une pull request
- [ ] Exiger les status checks : `Frontend (build, types, tests)`,
      `Rust (fmt, clippy, tests)` et `gitleaks (full history)`
- [ ] Exiger que la branche soit à jour avant merge
- [ ] Bloquer les force-push et la suppression de branche

> Sur un projet à un seul mainteneur, l'auto-review n'apporte rien : les status
> checks sont la protection réelle.

## 5. Métadonnées du dépôt

- [ ] Description : « Local-first subtitle generator for macOS — audio/video to
      SRT/TXT, no cloud »
- [ ] Topics : `macos`, `tauri`, `rust`, `whisper`, `subtitles`, `srt`,
      `local-first`, `privacy`, `apple-silicon`, `ffmpeg`
- [ ] Désactiver Wiki et Projects s'ils ne servent pas
- [ ] *(optionnel)* `CODE_OF_CONDUCT.md` — absent. GitHub le signalera dans les
      *community standards*. Utile si des contributions externes sont attendues,
      purement formel sur un projet solo.

## 6. Après le passage en public

- [ ] Vérifier que `ci.yml`, `security.yml` et `secret-scan.yml` passent
- [ ] Vérifier que le lien « Report a vulnerability » fonctionne réellement
- [ ] Vérifier Insights → Dependency graph
- [ ] Settings → Actions → *Require approval for all external contributors*

## 7. L'archive privée

- [ ] Confirmer que `romuhica73/ST-IA` est toujours **privé**
- [ ] Y ajouter une note (README ou description) indiquant qu'il s'agit de
      l'archive de développement, et pointant vers le dépôt Community public
- [ ] Les branches `feat/m*` peuvent y rester : c'est leur place. La
      recommandation « supprimer après le tag » ne s'applique plus, ce dépôt
      n'étant pas destiné à être publié.

## 8. Release 0.1.0 — hors périmètre de cette checklist

Ne pas taguer avant que tout ce qui précède soit fait. Voir
[`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md).

- [ ] Tag `v0.1.0` sur le **dépôt Community**
- [ ] Signature Developer ID + notarisation —
      **`APPLE_DEVELOPER_ID_NOT_AVAILABLE`** aujourd'hui. Bloque la distribution
      binaire officielle, **pas** la publication du source.
- [ ] Publier les SHA-256 du `.dmg` dans les notes de release
- [ ] Décider si les sidecars restent dans Git ou passent en artefacts de
      release (STIA-SEC-106)
