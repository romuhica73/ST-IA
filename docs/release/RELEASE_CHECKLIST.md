# ST-IA — Checklist de release macOS

Checklist opérationnelle à dérouler avant de déclarer une build distribuable. Cocher uniquement ce qui a été **réellement vérifié sur la build candidate**, pas ce qui « devrait » marcher.

Cible actuelle : **0.1.0**, macOS Apple Silicon (arm64), MVP local-first.

---

## 1. Git

- [ ] `git status --short` vide
- [ ] Branche de release à jour avec `origin`
- [ ] `main` non modifiée tant que la PR n'est pas validée
- [ ] Aucun fichier de test/fixture personnel commité (`test-media/private/` est gitignoré)

## 2. Version

- [ ] Version identique dans `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml`
- [ ] Version réellement présente dans le `.app` :
      `/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" ST-IA.app/Contents/Info.plist`

## 3. Bundle ID

- [ ] `CFBundleIdentifier` = `com.romainbourbon.stia` (jamais un identifiant `.dev`)
- [ ] Migration depuis l'ancien identifiant testée si l'utilisateur peut venir d'une build antérieure (ADR-006)

## 4. Sidecars

- [ ] `whisper-cli` reconstruit par `scripts/build-whisper-sidecar.sh` (échoue si `-mcpu=native`)
- [ ] `ffmpeg` reconstruit par `scripts/build-ffmpeg-sidecar.sh` si sa version a changé
- [ ] Pour chacun : `file` → `Mach-O 64-bit executable arm64`
- [ ] Pour chacun : `otool -L` → **aucune** dépendance Homebrew, rpath ou clone de développement
- [ ] Metal actif au runtime (`using MTL0 backend` dans la sortie whisper)
- [ ] Aucune extension CPU spécifique à la machine de build (pas de `SME`/`SME2` si build sur M4)

## 5. Modèle

- [ ] **Absent** du dépôt, du `.app` et du `.dmg` :
      `find ST-IA.app -name "*.bin" -size +100M` → vide
- [ ] URL de téléchargement épinglée à une révision immuable (pas `resolve/main/`)
- [ ] SHA-256 attendu inchangé dans `src-tauri/src/domain/model.rs`
- [ ] Téléchargement déclenché uniquement par action utilisateur explicite

## 6. Tests automatisés

- [ ] `pnpm build`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets` (0 warning)
- [ ] `cargo check`
- [ ] `cargo test`
- [ ] `pnpm tauri build`

## 7. Endurance

- [ ] 5, 15, 30 et 60 minutes traités jusqu'au bout
- [ ] Mémoire stable (pas de croissance continue sur le run 60 min)
- [ ] Workspace temporaire supprimé après chaque run
- [ ] Aucun `ffmpeg`/`whisper-cli` résiduel : `pgrep -fl "ffmpeg|whisper-cli"`
- [ ] Nouveau job possible après le run long

## 8. Validation des sorties

- [ ] `scripts/validate-srt.py` passe sur toutes les sorties d'endurance
- [ ] TXT non vide et cohérent avec le SRT

## 9. DaVinci Resolve (gate humain)

- [ ] SRT importé dans DaVinci Resolve
- [ ] Piste de sous-titres créée, texte visible, timecodes cohérents, synchronisation correcte

## 10. Licences

- [ ] `THIRD_PARTY_NOTICES.md` à jour (versions, provenances, SHA-256)
- [ ] `licenses/` contient les textes réels
- [ ] Notices présentes dans le bundle : `ST-IA.app/Contents/Resources/licenses/`
- [ ] Aucun composant GPL/non-free activé (`ffmpeg -version` → `--disable-gpl --disable-nonfree`)

## 11. Privacy

- [ ] Un seul point d'accès réseau dans le code (`reqwest` dans `install_model`)
- [ ] Aucun `fetch`/XHR/WebSocket côté frontend
- [ ] Aucune télémétrie, aucun analytics
- [ ] Aucune connexion réseau pendant une transcription :
      `lsof -p <pid> -a -i` vide

## 12. Capabilities

- [ ] Pas de shell arbitraire exposé au frontend
- [ ] `shell:allow-execute` borné aux deux sidecars nommés
- [ ] `opener` limité à `allow-reveal-item-in-dir`
- [ ] Aucune permission filesystem globale
- [ ] Aucune permission ajoutée pour le confort de développement

## 13. `.app`

- [ ] Se lance depuis le bundle (pas `pnpm tauri dev`)
- [ ] Icône : asset ST-IA approuvé, **pas** le logo Tauri par défaut
- [ ] Smoke test complet : sélection → transcription → SRT/TXT → Ouvrir le dossier → nouveau fichier
- [ ] Annulation et relance fonctionnelles
- [ ] Apparence correcte en mode clair **et** sombre

## 14. `.dmg`

- [ ] Généré par `pnpm tauri build`
- [ ] S'ouvre et l'application se lance depuis l'image montée

## 15. Signature

- [ ] `security find-identity -v -p codesigning` — identité Developer ID disponible
- [ ] Application signée
- [ ] `codesign --verify --deep --strict` OK
- [ ] Aucun secret dans les logs, les commits ou les rapports

## 16. Notarisation

- [ ] Build soumis à Apple (**avec autorisation explicite de l'utilisateur**)
- [ ] Ticket agrafé (`xcrun stapler staple`)
- [ ] `spctl -a -vvv` OK
- [ ] Test d'ouverture sur une machine n'ayant jamais vu la build

## 17. Checksums

- [ ] SHA-256 du `.dmg` calculé et consigné
- [ ] Nom de fichier conforme : `ST-IA-<version>-macos-arm64.dmg`

## 18. Validation humaine finale

- [ ] Un humain a lancé la build finale et confirmé le comportement
- [ ] Réserves connues explicitement listées dans les notes de version

---

## Statuts de distribution

| Niveau | Condition |
|---|---|
| **Local Release Candidate** | Sections 1 à 14 vertes. Signature/notarisation non requises. |
| **Public Distribution Ready** | Toutes les sections vertes, y compris 15, 16 et 18. |

Ne jamais annoncer « Public Distribution Ready » tant que la signature et la notarisation n'ont pas été réellement effectuées et vérifiées.
