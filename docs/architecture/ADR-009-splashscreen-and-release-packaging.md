# ADR-009 — Splashscreen applicatif et packaging de release macOS

Statut : `ACCEPTED` — qualifiée humainement le 2026-08-13, dans sa forme
finale : packaging inchangé, splash **intégré** à la fenêtre principale.

La décision d'origine d'une **fenêtre splash séparée** est `SUPERSEDED` par
la section « Splash intégré » en fin de document.

Gate humain sur l'architecture finale : chrome macOS visible dès le splash,
contour de fenêtre strictement identique du premier frame à l'application,
décalage précédemment observé **disparu**, entrée des contrôles validée,
Reduced Motion validé, et 3–5 relances sans flash blanc ni fenêtre
résiduelle.

> **Mise à jour — splash intégré (2026-08-13).** Le pattern « deux fenêtres
> natives » décrit ci-dessous a été abandonné. Même avec une géométrie
> strictement identique — mesurée, vérifiée sur 20 lancements — le passage
> d'une fenêtre à l'autre restait perceptible : macOS recrée un cadre, et
> l'œil le voit.
>
> Le splash est désormais une **couche à l'intérieur de la fenêtre
> principale**, rendue par le frontend. Il n'existe plus qu'une seule fenêtre
> native pendant tout le cycle de vie : le chrome macOS et ses pastilles sont
> visibles dès la première image, l'interface est montée et disposée derrière
> la couche pendant qu'elle s'affiche, et la fin de l'intro est le retrait
> d'une couche déjà transparente — pas un échange de fenêtres.
>
> Ce qui reste valable dans cette ADR : le packaging de release (§Décision 6),
> l'exigence de CSP, et le principe d'isolation du frontend. Ce qui ne l'est
> plus : la seconde fenêtre, son cycle de vie, sa capability dédiée et le
> handshake `notify_ui_ready` / `notify_splash_finished`, tous supprimés.
>
> Détail de la nouvelle architecture : section « Splash intégré » ci-dessous.

Date : 2026-08-12

Contexte : M9 (Bilingual Outputs, Animated Splashscreen & Release Packaging).

## Contexte

Deux sujets indépendants du moteur de transcription, regroupés ici parce
qu'ils partagent la même finalité : rendre ST-IA présentable en tant que
produit distribuable.

1. Au lancement, ST-IA affichait une fenêtre vide le temps que React monte,
   que les réglages se chargent et que l'état du modèle soit résolu. `App.tsx`
   rend explicitement `<main className="app" />` tant que `modelStatus` vaut
   `null`. Court, mais lu comme un défaut.
2. Aucun mécanisme ne produisait d'artefact utilisateur nommé et vérifiable :
   `pnpm tauri build` laissait un `.dmg` au nom du bundler dans
   `target/release/bundle/`, sans checksum ni contrôle de contenu.

## Décision 1 — Deux vraies fenêtres, pas un overlay

La fenêtre `main` est déclarée `"visible": false` dans `tauri.conf.json`. Une
seconde fenêtre, `splashscreen`, est construite en tout premier dans le hook
`setup`, avant le nettoyage de démarrage et la migration.

Alternative écartée : une `<div>` de recouvrement à l'intérieur de la fenêtre
principale. Elle ne masque rien de ce qui coûte réellement du temps — la
fenêtre est déjà ouverte, la WebView déjà en train de booter — et exige que
React soit monté pour afficher quoi que ce soit, c'est-à-dire précisément
l'attente à couvrir.

La bascule est faite en Rust, dans cet ordre : afficher `main`, puis fermer le
splash. L'inverse laisserait une image sans aucune fenêtre ST-IA à l'écran —
un scintillement, et sur macOS un aller-retour de focus vers une autre
application.

## Décision 2 — Le splash n'a aucune capability

Le label `splashscreen` n'apparaît dans aucun fichier de `capabilities/`. La
fenêtre ne peut donc invoquer aucune commande, écouter aucun événement,
lancer aucun sidecar, lire aucun fichier ni atteindre le réseau. Elle affiche
des ressources locales et rien d'autre.

C'est la conséquence directe du modèle de menace M8, qui traite le frontend
comme hostile : une surface qui n'existe pas n'a pas à être auditée. Un
`invoke` depuis le splash serait simplement refusé.

Le seul handshake est **une** commande, `notify_ui_ready`, appelée depuis la
fenêtre principale — qui détient déjà les capabilities de l'application. Elle
ne transporte qu'un booléen et ne retourne rien.

Trois tests d'intégration (`tests/capability_surface.rs`) verrouillent la
propriété : toute capability doit être explicitement rattachée à des fenêtres
nommées (une capability sans clé `windows` s'applique à *toutes* les
fenêtres), aucune ne doit citer le splash, et la fenêtre principale doit
continuer d'en détenir une — sans quoi un répertoire `capabilities/` vide
suffirait à faire passer les deux premiers tests.

## Décision 3 — Les préférences voyagent dans le fragment de l'URL

Le splash doit respecter le thème et la réduction d'animations choisis en M7
dès sa première image, sans pouvoir lire `settings.json`.

Rust lit les réglages (il le fait déjà) et transmet les deux *préférences*
brutes à la construction de la fenêtre :
`splash.html#theme=system|light|dark&motion=system|on|off`. Le splash applique
ensuite la même règle que le reste de l'application — une valeur explicite
gagne, sinon on interroge le système via `matchMedia`.

**Le fragment n'est pas cosmétique.** La première implémentation utilisait une
query string, et la fenêtre s'ouvrait **blanche** : le résolveur d'assets
embarqués de Tauri compare le chemin de la requête littéralement, donc
`splash.html?theme=light` ne correspond à aucun asset. Le symptôme est
particulièrement traître — la fenêtre existe, `webview.url()` retourne une URL
parfaitement normale, et rien n'est journalisé. Un fragment ne fait jamais
partie de la requête HTTP, donc le document se résout toujours.

C'est ce qui a motivé la sonde de rendu décrite ci-dessous : sans elle, le
défaut n'aurait été découvert qu'au gate humain.

Ce sont des préférences, pas des données utilisateur : aucun chemin, aucun nom
de fichier, aucun contenu de transcription n'atteint cette fenêtre.

La duplication de la logique de résolution entre `features/settings/resolve.ts`
et `splash/resolve.ts` est assumée (bundles distincts, aucun runtime partagé)
et tenue par un test qui compare les deux implémentations sur toutes les
combinaisons.

Côté Rust, `ThemePreference::as_str` / `MotionPreference::as_str` sont
comparées par test à leur forme sérialisée par serde, pour que les chaînes
attendues par le TypeScript ne puissent pas dériver silencieusement.

### Sonde de rendu

Une fenêtre qui s'ouvre n'est pas une fenêtre qui s'affiche, et le splash n'a
aucune capability pour le signaler lui-même. `src/splash/main.ts` positionne
donc `document.title` sur une sentinelle une fois son module exécuté, que Rust
relit sur la fenêtre juste avant de la fermer et journalise.

C'est le seul moyen sans capture d'écran de distinguer « la fenêtre s'est
ouverte et a rendu » de « la fenêtre s'est ouverte blanche » — le mode de
défaillance que produisent une URL d'asset erronée ou une violation de CSP.
Le titre du document (`ST-IA:splash`) est volontairement distinct du titre de
fenêtre posé par Rust (`ST-IA`), pour séparer aussi « le document n'a jamais
chargé » de « le document a chargé mais le module n'a pas tourné ». Un test
vérifie que la sentinelle est identique des deux côtés.

Rien n'est affiché : la fenêtre splash n'a pas de barre de titre.

## Décision 4 — Rien ne bloque, et la bascule est idempotente

Le splash a un plancher d'affichage minimal pour qu'un démarrage rapide se
lise comme une intro voulue et non comme un clignotement de fenêtre :

* **820 ms** en motion normale — juste au-delà de la composition CSS (barres
  0–380 ms, lignes 300–700 ms, mot-symbole 480–800 ms) ;
* **160 ms** en reduced motion — assez pour éviter un artefact visuel, trop
  court pour constituer un délai décoratif, ce que la réduction d'animations
  demande précisément de ne pas imposer.

Rust ne peut pas observer `prefers-reduced-motion` du système ; la fenêtre
principale, elle, le résout déjà pour elle-même et transmet sa valeur avec le
signal de disponibilité.

Aucune attente n'est bloquante : le plancher et le chien de garde de 10 s sont
des timers asynchrones (`tokio::time::sleep`), jamais un `thread::sleep`.

Trois chemins peuvent conclure la phase de splash — le signal de
disponibilité, le chien de garde, et la destruction de la fenêtre splash. Ils
courent vers un unique test-and-set, donc la bascule a lieu exactement une
fois. Le chien de garde et la récupération sur destruction existent pour que
l'application devienne utilisable même si le frontend ne signale jamais rien.

## Décision 5 — La CSP n'est pas touchée

Le splash a été écrit pour entrer dans la politique M8, et non l'inverse :
feuille de style externe, script module externe, aucun style ni script inline,
aucune ressource distante, aucune police téléchargée.

`tests/csp_policy.rs` épingle la politique (pas de `unsafe-inline`, pas de
`unsafe-eval`, `connect-src` strictement limité au pont IPC local) *et*
vérifie que `splash.html` s'y conforme réellement — une violation ne se
manifesterait qu'à l'exécution, par une fenêtre blanche.

## Décision 6 — Un script de packaging qui audite avant de collecter

`scripts/package-release.sh` produit `release-artifacts/` :

```
ST-IA-<version>-macos-arm64.dmg        artefact utilisateur recommandé
ST-IA-<version>-macos-arm64.app.zip    artefact avancé (archive `ditto`)
SHA256SUMS.txt                         vérifié après écriture
```

La version vient de `tauri.conf.json`, source de vérité déjà épinglée par le
test de cohérence M7 ; le script revérifie les trois fichiers, puis compare au
`CFBundleShortVersionString` et au `CFBundleIdentifier` du bundle réellement
construit — une build périmée ne peut donc pas être publiée sous un numéro de
version frais.

L'audit précède la collecte et la refuse en cas d'échec : aucun modèle
Whisper, aucun média de test, aucun `.env`, log, matériel de signature ou
source map dans le bundle ; licences, notices, icône et les deux sidecars
présents ; aucun chemin de la machine de build dans `Info.plist`.

L'archive avancée est produite avec `ditto` plutôt que `tar` : c'est le
mécanisme macOS qui préserve correctement les liens symboliques et le bit
exécutable d'un `.app`.

Le DMG conserve le layout standard du bundler Tauri. Aucune personnalisation
graphique : elle serait un coût de maintenance sans bénéfice utilisateur à ce
stade.

`release-artifacts/` est ignoré par Git — plusieurs centaines de mégaoctets,
non signés, régénérables à la demande.

## Conséquences

* Le démarrage montre une identité ST-IA plutôt qu'une fenêtre vide, sans
  ajouter de dépendance ni élargir la surface d'attaque : le splash pèse
  0,5 ko de JavaScript et 3 ko de CSS, et ne détient aucun privilège.
* Une seconde fenêtre existe désormais dans le cycle de vie de l'application.
  Le prix est la robustesse à écrire explicitement (chien de garde,
  récupération sur destruction, bascule idempotente), ce qui est fait.
* Le packaging est reproductible et vérifié, mais les artefacts restent
  **non signés et non notariés** — `PUBLIC_DISTRIBUTION_SIGNING_PENDING`. Le
  script l'affiche à chaque exécution. Gatekeeper les refusera sur une autre
  machine tant que ce point n'est pas traité (M10).
* Le splash ne couvre pas le cas rare où `migration::run` doit hacher un
  modèle de 574 Mo : `setup` reste synchrone sur ce point, exactement comme
  avant M9. Rendre cette étape asynchrone ferait interroger l'état du modèle
  par le frontend avant la fin de la migration, et afficherait à tort l'écran
  « Modèle requis » à un utilisateur qui met à jour. La régression serait pire
  que le symptôme. Le chemin normal (modèle de production présent) ne coûte
  que deux `is_file()`.

## Références

* [ADR-002 — architecture desktop](ADR-002-desktop-architecture.md) ;
* [ADR-007 — préférences locales et localisation](ADR-007-local-preferences-and-interface-localization.md) ;
* [Revue de sécurité M8](../security/M8_SECURITY_REVIEW.md) ;
* [Delta de sécurité M9](../security/M9_SECURITY_DELTA.md) ;
* [Checklist de release](../release/RELEASE_CHECKLIST.md) ;
* pattern officiel Tauri 2 : <https://v2.tauri.app/learn/splashscreen/>.

---

## Splash intégré (remplace les décisions 1 à 5)

### Une seule fenêtre native

Construite au démarrage dans `window::create`, jamais déclarée dans
`tauri.conf.json`. La raison est précise : la couleur de fond native doit
correspondre au thème résolu **avant** le premier affichage, et une fenêtre
déclarée dans la configuration est créée après le retour de `setup` — trop
tard pour éviter un flash de la mauvaise couleur.

La fenêtre est créée masquée puis affichée quelques instructions plus loin,
dans le même `setup`. Ce n'est pas un affichage en deux temps : c'est ce qui
permet de lire le thème du système (`theme()` exige une fenêtre) et de poser
la bonne couleur de fond avant la première image réellement visible.

### Aucune frame blanche

Trois couches doivent s'accorder, et le font :

1. **la fenêtre native** — `background_color` posé depuis le thème résolu
   (préférence explicite, sinon thème du système) ;
2. **le document** — `html` porte un fond issu d'une requête
   `prefers-color-scheme`, seule chose disponible avant l'exécution de tout
   script, `data-theme` reprenant la main ensuite ;
3. **la couche splash** — opaque, sur `var(--bg)`.

### L'interface est montée derrière l'intro

`App` rend son contenu normalement dès le premier instant ; la couche
`BootSplash` le recouvre. Quand la couche disparaît, il n'y a rien à monter,
rien à disposer et rien à révéler — d'où l'absence de décalage à la
transition.

La couche couvre la webview, qui sur macOS est exactement la zone de contenu :
la barre de titre native est en dehors et reste visible tout du long. Aucun
faux chrome n'est dessiné, ce qui rend l'approche neutre vis-à-vis de la
plateforme — un futur portage Windows garderait le même concept avec le
chrome natif de Windows.

### Cycle de vie

`useBootPhase` : `splash` → `ready`. Le verrou est au niveau du module et non
dans l'état d'un composant, parce que l'intro appartient au processus : React
StrictMode monte deux fois en développement, et tout remontage de `App`
rejouerait sinon six secondes de splash.

La fin de la couche déclenche l'entrée échelonnée de l'interface (~300 ms),
une seule fois par lancement. La couche s'éteint sur `animationend` de sa
propre animation de cycle, avec un minuteur de repli si le compositeur ne le
délivre jamais.

### Neutralité de plateforme

L'approche ne dépend d'aucune particularité de macOS. La couche d'intro
couvre la webview et ne dessine **aucun** chrome : pas de fausse barre de
titre, pas de fausses pastilles, aucune hypothèse sur la position ou
l'existence des contrôles de fenêtre. Le chrome reste celui du système, quel
qu'il soit.

Un futur portage Windows conserverait donc le même concept — chrome natif de
la plateforme + splash interne à la même webview — sans retoucher la couche
d'intro. **ST-IA v0.1.0 reste macOS Apple Silicon uniquement** : c'est une
propriété de l'architecture, pas une annonce de support.

### Surface supprimée

`splash.rs`, `capabilities/splash.json`, `splash.html`, l'entrée Vite dédiée,
`src/splash/*`, les commandes `notify_ui_ready` et `notify_splash_finished`
(du manifeste ACL et de la capability), et `MotionPreference::as_str` qui
n'existait que pour construire le fragment d'URL du splash.

**L'ACL applicative est conservée intégralement.** La disparition de la
seconde fenêtre ne change rien au modèle de menace M8 : la webview reste la
frontière non fiable, et l'autorisation des commandes reste imposée côté
Rust.
