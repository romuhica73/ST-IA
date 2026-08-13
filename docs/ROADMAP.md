# Roadmap

## M0 — Technical spike

Statut : `DONE`

* whisper.cpp ;
* Apple Silicon arm64 ;
* Metal ;
* FFmpeg ;
* SRT ;
* TXT.

## M0B — Qualification française

Statut : `DONE`

Objectifs :

* vrai média français ;
* comparaison modèles ;
* segmentation SRT ;
* performances ;
* choix modèle MVP.

Résultat : modèle MVP retenu `large-v3-turbo-q5_0` (voir [ADR-001](architecture/ADR-001-transcription-engine.md#11-qualification-française-mission-0b)). Qualification basée sur un échantillon unique (~3 min) — à élargir avant M5.

## M1 — Desktop shell

Statut : `DONE`

Objectifs :

* Tauri 2 ;
* React ;
* TypeScript ;
* fenêtre unique ;
* drag & drop ;
* sélection média ;
* validation.

Résultat : shell Tauri 2 fonctionnel avec sélection native et validation Rust (`inspect_media`). Interface alignée sur les mockups validés (Mission 1B) et vérifiée manuellement par l'utilisateur.

## M2 — Pipeline intégré

Statut : `DONE`

Objectifs :

* Rust orchestration ;
* FFmpeg sidecar ;
* whisper.cpp sidecar ;
* traitement end-to-end ;
* SRT/TXT.

Résultat : pipeline local prouvé de bout en bout depuis l'application (voir [ADR-003](architecture/ADR-003-local-transcription-pipeline.md)). Sidecars FFmpeg 9.0 et whisper-cli statiques (aucune dépendance Homebrew/dev clone, vérifié `otool -L`), modèle `large-v3-turbo-q5_0` résolu via Application Support, SRT/TXT réels générés pour `IMG_8484.MOV`. Mode « Précis » et téléchargement de modèle restent hors périmètre (M3).

## M3 — Model manager

Statut : `DONE`

Objectifs :

* modèle absent ;
* téléchargement explicite ;
* progression ;
* SHA-256 ;
* stockage Application Support ;
* fonctionnement offline.

Résultat : gestionnaire de modèle prouvé de bout en bout, mode développement et `.app` empaqueté (voir [ADR-004](architecture/ADR-004-model-management.md)). Téléchargement réel depuis Hugging Face, SHA-256 vérifié exactement (`394221709c...`), promotion atomique, modèle corrompu détecté et rejeté, transcription réelle après installation, fonctionnement hors ligne prouvé (aucune requête réseau pendant le pipeline).

## M4 — Robustesse

Statut : `DONE`

Objectifs :

* annulation ;
* fermeture application ;
* processus enfants ;
* temporaires ;
* fichier invalide ;
* absence audio ;
* espace disque ;
* modèle corrompu.

Résultat : cycle de vie des jobs prouvé de bout en bout sur le `.app` empaqueté (voir [ADR-005](architecture/ADR-005-runtime-lifecycle-and-cancellation.md), `ACCEPTED`). Emplacement de job unique avec revendication atomique (double lancement refusé côté Rust), handles FFmpeg/whisper-cli détenus et tuables, annulation réelle d'un Whisper en cours, retry sans redémarrage, fermeture applicative tuant l'enfant et supprimant le workspace (`shutdown: killed child pid …` observé), récupération au démarrage bornée aux répertoires ST-IA, média invalide traité en erreur métier sans jamais lancer Whisper, garde d'espace disque. Réserve : l'annulation pendant l'étape FFmpeg n'a pas pu être observée (étape trop brève sur les médias qualifiés) — même chemin de code que Whisper.

## M5 — MVP macOS

Statut : `DONE`

Objectifs :

* UX finale ;
* build Release ;
* `.app` ;
* tests médias longs ;
* validation DaVinci Resolve.

Avancement : identité de production `com.romainbourbon.stia` adoptée et migration des données depuis l'ancien identifiant `.dev` prouvée sans retéléchargement (voir [ADR-006](architecture/ADR-006-release-identity-and-data-migration.md), `ACCEPTED`). Réserve de portabilité Apple Silicon close : `whisper-cli` reconstruit avec `GGML_NATIVE=OFF`, sans `-mcpu=native` ni extension SME propre à la machine de build, Metal conservé, sorties octet pour octet identiques. Endurance 5/15/30/60 min mesurée, icône de production intégrée (asset ST-IA approuvé, plus de logo Tauri par défaut), `.app` et `.dmg` produits, notices de licence embarquées. Gate DaVinci Resolve validé par l'utilisateur (piste de sous-titres, timecodes, synchronisation OK sur un média réel de 60 min) — limite de vocabulaire observée sur certains noms propres/termes techniques, reportée en future mission qualité (voir ci-dessous). Smoke test GUI final validé par l'utilisateur sur le `.app` empaqueté (icône ST-IA incluse) : parcours complet média → SRT/TXT → Ouvrir le dossier/Finder, et annulation → retour propre → retry sans redémarrage → transcription complète, tous deux confirmés OK.

Résultat : release candidate locale macOS 0.1.0 qualifiée de bout en bout — pipeline, gestionnaire de modèle, cycle de vie/annulation, identité de production, portabilité Apple Silicon, endurance jusqu'à 60 minutes, gate DaVinci Resolve et smoke test GUI. Non signée/notariée (hors périmètre local RC, voir [checklist de release](release/RELEASE_CHECKLIST.md)).

### Future — qualité de transcription (vocabulaire)

Non planifiée, non commencée. À étudier dans une mission ultérieure :

* vocabulaire contextuel / noms propres ;
* terminologie technique ;
* évaluer le biasing par prompt initial de whisper.cpp.

Pas d'implémentation, pas de nouveau modèle, pas de champ UI dans cette entrée.

## M6 — Visual Polish & Motion

Statut : `DONE`

Objectif : améliorer l'identité visuelle et le ressenti de ST-IA sans modifier son architecture fonctionnelle.

Périmètre :

* hiérarchie visuelle ;
* micro-interactions ;
* transitions ;
* feedback de drag & drop ;
* progression ;
* états succès/erreur ;
* cohérence light/dark ;
* motion accessible ;
* réduction des animations si macOS `prefers-reduced-motion`.

Avancement : identité visuelle dérivée de l'icône ST-IA (teal-cyan `#0c7180`/`#2dd4bf` échantillonné sur le dégradé réel de l'icône, calmé pour un usage UI — pas de neon, pas de gradient dans le chrome), remplaçant le bleu générique précédent. Fond graphite/anthracite en dark mode (`#1a1c20`) plutôt qu'un simple inversé. Tokens formalisés (rayons, ombre unique très subtile, échelle de motion `100–320ms`) réutilisés dans tous les écrans. Icônes glyphe (`✓`, `!`, `♪`, `✨`) remplacées par des SVG cohérents avec la famille existante. Zone de dépôt avec retour drag-over renforcé (échelle + halo, sobre). Boutons dotés d'états hover/active/focus-visible complets. Écran de progression : marqueurs d'étape animés à l'achèvement, barre indéterminée plus calme. Écran de succès : check en scale-in, liste des fichiers en apparition échelonnée. Écrans d'erreur passés d'un remplissage rouge plein à une icône teintée (rouge réservé à l'icône, jamais un bloc plein) — nettement moins anxiogène. `prefers-reduced-motion` respecté : coupe globale des durées/animations, avec un cas particulier traité explicitement (la barre de progression indéterminée passe à un état statique plutôt que d'être figée à mi-course par la règle générale). Toutes les corrections de contraste AA de M5 revérifiées et conservées, plus deux nouveaux tokens texte-sûrs (`--success-text` clair/sombre) pour le badge SRT qui ne les avait pas. Correction de microcopie : « Déposez votre vidéo ici » → « Déposez votre média ici » (incohérent avec le support audio existant).

Réserve : la validation par capture d'écran automatisée (§28) n'a pas pu être produite — trois tentatives via automatisation d'accessibilité macOS ont chacune capturé incidemment une fenêtre ou un panneau étranger à ST-IA exposant des noms de fichiers/dossiers personnels ; chaque capture a été supprimée immédiatement sans être exploitée. Validation visuelle confirmée par l'utilisateur via le gate GUI humain.

Validé humainement, convergé sur `main`.

## M7 — Settings, i18n, About & Versioning

Statut : `DONE`

Objectif : ajouter la couche « application configurable » — réglages persistants, interface bilingue FR/EN, écran À propos avec version réelle — sans toucher au moteur de transcription ni au pipeline.

Périmètre :

* réglages (thème, motion, langue) persistés localement (`Application Support/settings.json`), jamais dans `localStorage` ;
* thème Système / Clair / Sombre, suit macOS en direct ;
* réduction des animations Système / Activé / Désactivé, cohérente avec le système de motion M6 ;
* internationalisation complète (i18next/react-i18next), catalogues FR/EN avec parité de clés testée automatiquement ;
* langue de l'interface strictement indépendante de la langue de transcription (le champ média renommé « Langue de transcription » / « Transcription language ») ;
* écran À propos avec version réelle (`app.package_info().version`, jamais codée en dur) ;
* vérification automatisée de cohérence de version entre `package.json`, `Cargo.toml`, `tauri.conf.json` ;
* sélecteur Rapide/Précis retiré (aucun comportement réel derrière, voir ADR-007 décision 7) — reporté en v0.2.

Correctif final : icône Settings remplacée (se lisait comme un soleil, pas des réglages), action rapide de thème ajoutée dans la barre supérieure (Système → Clair → Sombre → Système, même préférence que Réglages → Apparence, aucune deuxième source de vérité), espacement de l'écran À propos amélioré (groupes séparés, licences tierces en rangées structurées, aucun nouveau contenu). Qualifié humainement en deux passes : gate fonctionnel (Settings, thème, langue, persistance, indépendance UI/transcription, accessibilité, À propos, parcours complet) puis gate visuel du correctif (icône, action rapide, espacement About) — tous confirmés.

Voir [ADR-007](architecture/ADR-007-local-preferences-and-interface-localization.md) (`ACCEPTED`) pour le détail des décisions.

## M8 — Open Source & Security Readiness (DONE)

Audit complet mené avant toute modification de code (findings first). Résultats dans
[`docs/security/M8_SECURITY_REVIEW.md`](security/M8_SECURITY_REVIEW.md) et
[`docs/security/OPEN_SOURCE_READINESS.md`](security/OPEN_SOURCE_READINESS.md).

Couvert : threat model, audit du HEAD, audit exhaustif de l'historique Git, secrets,
PII, `.gitignore`, surface des commandes Tauri, capabilities, validation des entrées
IPC, sécurité du système de fichiers et du nettoyage, réglages, injection frontend,
CSP, réseau/privacy, sécurité du modèle, sidecars, dépendances JS et Rust, supply
chain, licences, fichiers open source, CI et Dependabot.

Résultats : **0 secret** dans le dépôt et dans les 43 commits de l'historique
(`gitleaks` + extraction manuelle des 265 blobs texte) — **aucune réécriture
d'historique nécessaire**. **0 vulnérabilité** de dépendance (npm et Rust).
3 findings MEDIUM, 2 LOW et 1 HARDENING corrigés : validation du chemin média à la
frontière IPC, CSP restrictive appliquée et vérifiée sur build empaquetée, suppression
du paramètre de chemin de `open_output_folder`, plafond de taille au téléchargement du
modèle, refus des liens symboliques au nettoyage, client HTTPS strict. 11 tests
supplémentaires (78 tests Rust au total), dont des tests adversariaux (traversée,
liens symboliques, noms de fichiers hostiles, chaînes d'URL).

**Licence principale : MIT**, décidée par l'auteur et appliquée (`LICENSE`,
`package.json`, `Cargo.toml`, README). Les licences tierces sont inchangées : FFmpeg
reste LGPL-2.1, whisper.cpp reste sous sa propre MIT, et `THIRD_PARTY_NOTICES.md` reste
un document distinct. Rien n'a été relicencié.

Gate humain passé sur le `.app` empaqueté : réglages et bascule FR/EN, parcours média
complet jusqu'aux SRT/TXT et au Finder, annulation pendant Whisper puis retry sans
redémarrage, et téléchargement sécurisé du modèle mesuré de bout en bout (endpoint
épinglé, HTTPS, SHA-256 conforme, promotion après validation seulement, aucun
`.download` résiduel, 2 Ko sortants contre 493 Mo entrants). Aucun comportement anormal
lié à la CSP. Les SRT/TXT produits après correctifs sont identiques aux tailles
qualifiées en M2/M4.

Réserves restantes, aucune n'étant un blocker de sécurité ou de confidentialité :
signature/notarisation Apple (M9), revue juridique recommandée sur le mode de
distribution de FFmpeg sous LGPL, et reproductibilité des sidecars binaires suivis dans
Git (compromis documenté). Les paramètres GitHub restent recommandés et non appliqués —
voir [`docs/release/GITHUB_PUBLICATION_CHECKLIST.md`](release/GITHUB_PUBLICATION_CHECKLIST.md).

## M9 — Bilingual Outputs, Splashscreen & Release Packaging

Statut : `DONE` — qualifiée humainement le 2026-08-13, prête à converger.

### Sortie bilingue française + anglaise

L'audit préalable a d'abord établi que le modèle épinglé `large-v3-turbo-q5_0`
**ne sait pas traduire** : il accepte `-tr` sans erreur et renvoie du français,
en blocs de 30 s. Un témoin `ggml-small`, mêmes binaire et arguments, produit
de l'anglais correct — le blocage est le modèle, pas notre build
([ADR-008](architecture/ADR-008-bilingual-output-pipeline.md)).

L'auteur a levé la contrainte « pas de second modèle », avec pour priorité la
meilleure qualité possible. Un modèle **dédié à la traduction** a donc été
ajouté, sans toucher au modèle de transcription
([ADR-010](architecture/ADR-010-local-english-translation.md)) :

| | Transcription | Traduction |
| --- | --- | --- |
| Modèle | `large-v3-turbo-q5_0` (inchangé) | `large-v3`, non-turbo, non quantisé |
| Taille | 574 Mo | 3,1 Go |
| Téléchargé | au premier usage | uniquement si English est demandé |
| Vitesse mesurée | ~0,12× le temps réel | ~0,9× le temps réel |
| Pic RAM mesuré | — | 3,85 Go |

Pipeline : FFmpeg **une fois**, puis les passes Whisper **séquentiellement**
dans une boucle unique — au plus un enfant `whisper-cli` à tout instant, la
garantie M4 tenant littéralement pour un job bilingue. Publication atomique :
une annulation ou un échec pendant la traduction ne publie **rien**, pas même
la moitié française déjà calculée.

Nommage : `IMG_8484.srt` pour le français seul (noms historiques préservés),
`IMG_8484.en.srt` pour l'anglais seul, `IMG_8484.fr.srt` + `IMG_8484.en.srt`
pour les deux.

UX : cartes sélectionnables construites sur de vraies cases à cocher (focus,
clavier et rôle ARIA natifs), bouton indiquant le nombre réel de fichiers,
étape « Traduction anglaise » visible dans la progression avec sa progression
Whisper réelle, résultats groupés par version. Catalogues FR/EN à parité.

### Splashscreen

Cycle de **6 s** piloté par la fin visuelle de l'animation et non par un
minuteur parallèle : fondu d'entrée 1 s, composition
stable 3 s, fondu de sortie 2 s, puis coupe franche vers la fenêtre principale
(qui n'a aucun fondu propre). La bascule n'a lieu que lorsque *deux* signaux
sont arrivés — l'animation est terminée, et le frontend sait quel écran
afficher — de sorte que l'animation n'est jamais tronquée et que la fenêtre
principale n'apparaît jamais derrière un splash visible.

Reduced Motion conserve **la même durée** : seules les décorations
disparaissent (waveform, tracés, translations). C'est une demande de retirer
le mouvement, pas d'imposer une version pressée du produit.

Deux défauts trouvés par l'instrumentation, tous deux corrigés : les
préférences passées en query string ne résolvaient aucun asset (fenêtre
blanche, URL pourtant normale), et une animation lancée pendant que la fenêtre
était encore masquée pouvait ne jamais émettre `animationend` (1 échec sur 20,
rattrapé par le chien de garde). 20/20 lancements conformes après correction.

### Packaging

`scripts/package-release.sh` produit `release-artifacts/` (DMG, archive `.app`,
`SHA256SUMS.txt` vérifié), audite le bundle avant de collecter et refuse de
packager en cas d'échec. Aucun des deux modèles n'est embarqué : le `.app`
pèse 22 Mo.

### Correctif de sécurité

L'implémentation a révélé que **les commandes applicatives n'étaient soumises
à aucun ACL** : dans Tauri 2, les capabilities ne gouvernent par défaut que
les commandes de plugins, si bien que la fenêtre splash pouvait appeler
`start_transcription` ou `install_model`. Corrigé par un manifeste ACL
applicatif (`build.rs`) et deux capabilities distinctes ; le splash n'en
détient plus qu'**une seule**. Voir le
[delta de sécurité M9](security/M9_SECURITY_DELTA.md).

### Qualification

**Automatisée** — 20/20 lancements empaquetés du démarrage même-fenêtre
(10 par mode de motion : une seule fenêtre native, géométrie identique
pendant et après l'intro, aucun processus résiduel), 5/5 cas de démarrage
robuste, **0 socket réseau** au repos comme pendant les deux passes Whisper,
DMG monté et audité. 71 tests frontend et 155 tests Rust (33 et 79 avant
cette mission), `fmt` et `clippy -D warnings` propres.

**Humaine** — qualifiée en plusieurs passes successives par l'auteur :

* transcription française, traduction anglaise, français + anglais
  (4 fichiers groupés par version), annulation pendant la traduction puis
  relance complète sans redémarrage ;
* progression jugée fiable et lisible, y compris quand le pourcentage
  n'avance pas ;
* transparence des modèles IA jugée claire et suffisante ;
* shell fixe, navigation Réglages, entrée de lancement, états de boutons,
  erreurs de validation et Reduced Motion validés ;
* démarrage même-fenêtre : chrome macOS présent dès l'intro, géométrie
  strictement constante, décalage précédemment observé **disparu**, relances
  sans flash blanc ni fenêtre résiduelle.

Deux directions UI ont été abandonnées en cours de mission après mesure, et
sont documentées comme décisions et non comme défauts (voir « Architecture UI
finale »).

### Architecture UI finale

Deux directions successives ont été abandonnées après mesure, et la
documentation les conserve comme telles plutôt que comme des bugs :

* **fenêtre redimensionnée par le contenu** → remplacée par un **shell fixe**
  900 × 640, chaque écran étant composé pour cette surface
  ([ADR-011](architecture/ADR-011-fixed-desktop-shell.md)) ;
* **fenêtre splash séparée** → remplacée par un **splash intégré** à la
  fenêtre principale. Même avec une géométrie strictement identique, vérifiée
  sur 20 lancements, le passage d'une fenêtre native à l'autre restait
  perceptible ([ADR-009](architecture/ADR-009-splashscreen-and-release-packaging.md),
  section « Splash intégré »).

L'application n'ouvre plus qu'**une seule fenêtre native** pour tout son cycle
de vie : le chrome macOS est visible dès la première image, l'interface est
montée derrière la couche d'intro pendant qu'elle s'affiche, et la fin de
l'intro est le retrait d'une couche déjà transparente.

Réglages refondus en navigation desktop (colonne de sections + panneau),
grammaire de mouvement unique (survol, pression, sélection, erreur, entrée de
lancement), Reduced Motion retirant le mouvement sans retirer l'information.

### `V0.1_UI_FEATURE_FREEZE`

L'interface de la v0.1.0 est **gelée**. Aucune nouvelle fonctionnalité ni
refonte UI avant la release, hors bug bloquant.

La suite est exclusivement :

1. merge de M9 ;
2. durcissement de release / revue delta complète ;
3. signature Developer ID et notarisation Apple ;
4. artefacts d'installation ;
5. publication du dépôt ;
6. release v0.1.0.

## M10 — Signature, notarisation et publication (non commencée)

Signature Developer ID, notarisation Apple, revue delta complète
(`FULL_DELTA_REVIEW_PENDING_M10`), publication GitHub. Dépend de la
disponibilité d'une identité de signature et d'une décision explicite de
rendre le repository public.

## Post-MVP

Liste indicative, non engageante :

* diarisation ;
* chapitres YouTube ;
* résumés ;
* export formats additionnels ;
* support Windows.
