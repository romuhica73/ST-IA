## Ce que fait cette PR

<!-- Le pourquoi, pas la liste des fichiers. -->

## Comment ça a été vérifié

<!-- Commandes lancées, et ce qui a été testé à la main sur le .app si pertinent. -->

- [ ] `pnpm build` et `pnpm test`
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
- [ ] Testé sur l'application empaquetée (si le changement est visible pour l'utilisateur)

## Sécurité

- [ ] Aucune nouvelle sortie réseau, télémétrie ou analytics
- [ ] Aucune nouvelle `#[tauri::command]`, ni nouvel argument à une commande existante
- [ ] Aucun changement de capabilities ni de CSP
- [ ] Aucun nouveau chemin construit, fichier supprimé ou processus lancé

Si une case ci-dessus est décochée, expliquez ici — ce n'est pas bloquant, mais
cela demande une relecture attentive (voir `docs/security/THREAT_MODEL.md`).

> Ne décrivez **jamais** une vulnérabilité dans une PR publique. Voir `SECURITY.md`.
