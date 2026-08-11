# ADR-007 — Préférences locales et localisation de l'interface

## Statut

**PROVISIONAL**

L'architecture, l'implémentation et les tests automatisés sont en place. Promotion en `ACCEPTED` après le gate humain (qualification manuelle sur le `.app` empaqueté — persistance, changement de langue à chaud, indépendance UI/transcription).

## Contexte

M6 a livré l'identité visuelle et le système de motion de ST-IA, mais l'application restait figée : une seule langue d'interface (français, codée en dur dans chaque composant), aucun réglage utilisateur persistant, aucune information de version affichée. M7 ajoute la couche « application configurable » sans toucher au moteur de transcription ni au pipeline FFmpeg/whisper.cpp.

## Décision 1 — Une seule source de vérité pour les préférences

Trois préférences indépendantes, chacune avec une option « Système » qui est aussi la valeur par défaut :

```rust
pub struct Settings {
    pub theme: ThemePreference,     // System | Light | Dark
    pub motion: MotionPreference,   // System | On | Off
    pub language: LanguagePreference, // System | Fr | En
}
```

Stockage : un fichier unique `{Application Support}/<bundle-id>/settings.json`, résolu via l'API de chemins Tauri (même famille que `models/` du gestionnaire de modèle M3). Écriture par fichier temporaire puis renommage atomique (`settings.json.tmp` → `settings.json`), donc un crash pendant l'écriture ne peut jamais laisser un fichier à moitié écrit qui échouerait ensuite à la lecture.

**Aucune préférence n'est jamais dans `localStorage`, ni dans une variable Rust distincte, ni dans un troisième mécanisme.** Le frontend n'écrit rien localement de son côté ; il lit/écrit exclusivement via `get_settings`/`save_settings`, deux commandes Tauri qui délèguent au fichier Rust.

## Décision 2 — Validation et fallback

`Settings::parse(raw: &str) -> Settings` ne peut jamais échouer vers l'extérieur : un JSON invalide, un champ manquant, ou une valeur d'enum inconnue retombent tous sur `Settings::default()` **en bloc**, jamais champ par champ. Un fichier partiellement corrompu (ex. `theme` invalide mais `motion`/`language` valides) est donc entièrement rejeté plutôt que partiellement récupéré — plus simple à raisonner, et le fichier se « répare » de lui-même au prochain enregistrement. Testé explicitement : fichier vide, JSON invalide, valeur d'enum inconnue, champs manquants, champs inconnus supplémentaires (tolérés, pour la compatibilité ascendante si une future version ajoute un champ).

## Décision 3 — Résolution thème/motion sans flash perceptible

`main.tsx` résout `data-theme`/`data-motion` **de façon synchrone**, avant le premier rendu React, via `window.matchMedia(...).matches` (disponible immédiatement, sans attente). C'est exactement le cas par défaut (« Système ») pour un premier lancement — donc zéro flash dans le cas courant. `useApplySettings` prend ensuite le relais : il corrige l'attribut une fois les vraies préférences chargées depuis Rust (seul le cas d'une préférence explicite différente du système peut produire un flash sub-perceptible, à la vitesse d'un aller-retour IPC local), et il garde le thème/motion **vivants** — si le thème macOS change pendant que ST-IA est ouverte et que la préférence est « Système », l'interface suit sans redémarrage (`matchMedia(...).addEventListener('change', ...)`), pareil pour le mouvement réduit.

Les jetons CSS sombres (`--bg`, `--fg`, etc.) vivent **à un seul endroit** : `:root[data-theme="dark"]`, pas dans une media query `prefers-color-scheme` séparée. Une media query aurait dupliqué chaque valeur dans deux blocs (système vs forcé), avec un risque réel de divergence dans le temps. Le compromis assumé : pas de repli CSS pur si JS ne s'exécute jamais avant le premier rendu — négligeable en pratique pour une app Tauri locale (bundle déjà local, exécution en quelques millisecondes), vérifié empiriquement en qualification.

Même raisonnement, même compromis, pour `prefers-reduced-motion` — mais avec une nuance : c'est une fonctionnalité d'accessibilité, pas seulement esthétique. Le choix reste le même (source unique via `[data-motion="reduce"]`, résolution synchrone pré-peinture) précisément parce qu'une media query pure ne peut *jamais* exprimer le cas « Off » (forcer le mouvement complet même si le système demande une réduction) sans une bataille de spécificité/`!important` fragile sur un sélecteur générique `*` — la solution à deux mécanismes aurait été strictement pire, pas plus sûre.

## Décision 4 — i18n : bibliothèque et architecture

**i18next 26 + react-i18next 17** (MIT). Justification : mature, TypeScript-friendly, interpolation et pluralisation intégrées (règles CLDR, gère nativement le singulier/pluriel français **et** anglais via les clés `_one`/`_other`), fonctionne entièrement hors ligne avec des ressources embarquées (`resources: { fr: {...}, en: {...} }`) — **aucun backend HTTP, aucune requête réseau, jamais**. Surface d'API réduite à l'essentiel : `useTranslation()` et `t()`.

Catalogues : `src/i18n/locales/{fr,en}.ts`, objets TypeScript imbriqués (pas de fichiers JSON séparés à synchroniser manuellement). Clés structurées par domaine (`drop.title`, `error.codes.audioPreparationFailed.message`, `settings.themeLabel`, …), jamais la phrase française comme clé.

Détection de la langue système : `resolveSystemLanguage(navigator.language)` — `fr` si le système est français, `en` si anglais, **`en` par défaut sinon** (choisi comme repli international plutôt que le français, pour qu'un système ni français ni anglais obtienne quand même une interface qu'une majorité d'utilisateurs peuvent lire).

## Décision 5 — Langue UI ≠ langue de transcription

Ce sont deux réglages strictement indépendants. Le champ auparavant appelé « Langue » sur l'écran média est renommé **« Langue de transcription »** / **« Transcription language »**, et reste câblé uniquement sur `fr` (seule langue qualifiée par le pipeline whisper.cpp, ADR-001/ADR-003 — inchangé). Aucun changement de la langue d'interface ne modifie ce réglage, et réciproquement. Testé fonctionnellement : UI en anglais, transcription française sur un média réel, fonctionne normalement.

## Décision 6 — Le backend garde ses codes, le frontend traduit

Aucun code d'erreur Rust n'est renommé ni traduit côté Rust (`TranscriptionErrorCode`, `MediaErrorCode`, `ModelErrorCode` — inchangés, contrats M4/M5 non cassés). Ce qui change : les composants affichaient auparavant `jobStatus.message`/`modelStatus.message`/`mediaState.message` — la chaîne française brute construite côté Rust — directement dans l'UI. Ces champs `message` **existent toujours sur le fil** (Rust les envoie), mais ne sont plus affichés : chaque composant traduit désormais depuis `code` via `t(\`error.codes.${code}.message\`)` (et l'équivalent pour `mediaCodes`/`modelCodes`), avec le texte français/anglais répliqué fidèlement dans les deux catalogues. `mediaState` a été étendu avec un champ `code` qu'il n'avait pas jusqu'ici (le composant ne recevait que `message`) — changement nécessaire pour atteindre la même couverture FR/EN que les erreurs de transcription et de modèle, pas une extension de périmètre.

## Décision 7 — Mode Rapide/Précis retiré

Le sélecteur « Rapide/Précis » n'avait aucun comportement réel : `mode` était un état local React jamais transmis à `start_transcription`, jamais lu par le backend. Le garder aurait exposé une fonctionnalité qui ment sur ce qu'elle fait. Retiré entièrement de l'écran média (§38) ; le seul comportement qualifié (`large-v3-turbo-q5_0`) devient implicite. `TranscriptionMode`, `BoltIcon`, `TargetIcon` supprimés (code mort). Documenté comme travail futur v0.2 dans la roadmap, non entamé ici.

## Conséquences

* Un utilisateur macOS non francophone peut utiliser ST-IA en anglais dès le premier lancement (résolution automatique), sans jamais avoir à ouvrir les réglages.
* Un changement de thème/langue/motion prend effet immédiatement, sans redémarrage.
* Les traductions sont scellées dans le binaire — pas de dépendance réseau, pas de service de traduction, cohérent avec le principe local-first du projet.
* Coût : deux catalogues à tenir synchronisés manuellement (atténué par le test de parité automatisé) et une nouvelle dépendance (`i18next`/`react-i18next`, ~55 ko minifiés cumulés) dans un bundle qui reste, en valeur absolue, très léger.

## Qualification

Automatisée : 19 tests Vitest (résolution de locale y compris repli, parité stricte des clés FR/EN et absence de valeur vide, résolution thème/motion/langue y compris override forcé contre l'état système, `formatBytes` sensible à la langue) ; 10 tests Rust pour `Settings` (defaults, round-trip JSON, fichier vide/invalide/enum inconnu/champs manquants rejetés en bloc, champs inconnus tolérés) et la persistance fichier (round-trip, atomicité, repli sur fichier corrompu) ; 1 test d'intégration Rust pour la cohérence de version entre `package.json`, `Cargo.toml` et `tauri.conf.json`.

Restant à qualifier manuellement (gate humain, section dédiée du rapport M7) : persistance réelle à travers un redémarrage de l'application, changement de langue à chaud observé visuellement, indépendance UI/transcription sur un média réel, accessibilité clavier des contrôles de réglages, absence de régression du splash/premier écran en mode réduit.
