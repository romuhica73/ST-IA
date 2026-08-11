# ADR-006 — Identité de production, portabilité du moteur et migration des données

## Statut

**ACCEPTED**

Le changement d'identifiant, la migration réelle des données et la portabilité du sidecar Whisper sont implémentés, mesurés et qualifiés sur le build empaqueté (voir Qualification).

Note séparée, hors décision architecturale de cette ADR : le gate produit M5 (import du SRT dans DaVinci Resolve) a également été validé par l'utilisateur sur un média réel de 60 minutes — piste de sous-titres créée, timecodes cohérents, synchronisation correcte du début à la fin. Une limite de vocabulaire a été observée sur certains noms propres/termes techniques (ex. « Claude Code ») ; elle ne remet pas en cause cette ADR et relève d'une future mission de qualité de transcription.

## Contexte

M0 à M4 ont livré une application fonctionnelle mais explicitement pré-release :

* l'identifiant de bundle était `com.romainbourbon.st-ia.dev`, marqué **provisoire** dès l'ADR-002 (« devra être revu et confirmé avant tout chantier de signature/notarisation ») ;
* le sidecar `whisper-cli` était compilé avec les réglages par défaut de ggml, c'est-à-dire `GGML_NATIVE=ON`, laissant ouverte la réserve `PORTABILITY_APPLE_SILICON_TO_REQUALIFY_BEFORE_PUBLIC_DISTRIBUTION`.

M5 ferme ces deux points pour produire une Release Candidate macOS Apple Silicon locale, sans ajouter la moindre fonctionnalité produit.

## Décision 1 — Identifiant de production

| | Valeur |
|---|---|
| Ancien identifiant (M0–M4) | `com.romainbourbon.st-ia.dev` |
| Identifiant de production | `com.romainbourbon.stia` |
| Version cible RC | `0.1.0` |

Aucune contrainte Apple préexistante ne s'y oppose : le dépôt ne contient ni entitlements, ni provisioning profile, ni bloc de signature dans `tauri.conf.json`, et aucun App ID Apple n'est supposé exister. Le suffixe `.dev` est retiré parce qu'il désignait une phase, pas un produit.

La version `0.1.0` était déjà cohérente entre `tauri.conf.json`, `package.json` et `Cargo.toml` ; elle n'a pas été dupliquée ailleurs. `tauri.conf.json` reste la source qui alimente l'`Info.plist` généré.

## Décision 2 — Migration des données `.dev` → production

Changer l'identifiant change le répertoire qu'macOS résout pour `Application Support`. Sans intervention, un utilisateur existant se verrait redemander **574 Mo** de modèle déjà téléchargé et déjà vérifié. C'est inacceptable pour une mise à jour.

Au démarrage, avant toute détection du modèle, ST-IA adopte le modèle de l'ancien emplacement :

```text
production a déjà un modèle ?  →  oui  →  ne rien faire (idempotent)
                                  non
                                   ↓
ancien modèle présent ?        →  non  →  ne rien faire (flow Model Manager normal)
                                  oui
                                   ↓
ancien modèle conforme au manifeste (taille + SHA-256) ?
                                  non  →  ne rien faire, laisser le fichier intact
                                  oui
                                   ↓
                          rename atomique  →  vérification destination
                                   ↓
                    purge des répertoires devenus vides
```

Propriétés retenues :

* **Périmètre minimal.** Le seul artefact considéré est le fichier modèle, par son nom exact. Aucun balayage générique d'`Application Support`, aucun fichier étranger déplacé ou supprimé.
* **Déplacement atomique.** Les deux emplacements sont sous `Application Support`, donc sur le même volume : `rename` est atomique. Le fichier est soit à l'ancien chemin, soit au nouveau — jamais perdu, jamais à moitié écrit. Pas de pic disque de 1,1 Go, et surtout **aucune étape de suppression distincte** qui pourrait s'exécuter avant que la destination existe.
* **Repli par copie** si les deux emplacements se trouvaient sur des volumes différents : copie vers le nom temporaire du Model Manager → vérification SHA-256 → promotion atomique → suppression de la source seulement ensuite.
* **Le modèle n'est jamais déclaré `ready` par la migration.** Celle-ci ne fait que placer le fichier ; l'autorité reste la détection du Model Manager (M3), qui recalcule le SHA-256 au démarrage.
* **Un modèle ancien corrompu n'est jamais promu.** Il est laissé en place et l'utilisateur retombe sur le flow « Modèle requis » normal.
* **Purge prudente.** Les répertoires hérités ne sont retirés que via `remove_dir` (non récursif), qui échoue sur un répertoire non vide : tout ce que l'utilisateur aurait pu y déposer est préservé par construction.

## Décision 3 — Portabilité du sidecar Whisper

`GGML_NATIVE=OFF` est ajouté au script de build, et ce n'est pas une option pour un binaire distribué.

Avec le défaut de ggml (`ON`), le CMake d'ggml interroge clang avec `-mcpu=native`, extrait le drapeau résolu et le grave dans le binaire, en y ajoutant les extensions détectées sur la machine de build. Sur la machine de qualification (Apple M4), cela activait **SME et SME2**, absentes des M1/M2/M3 : le binaire aurait levé une instruction illégale sur toute machine Apple Silicon antérieure.

Avec `GGML_NATIVE=OFF`, ggml n'émet **aucun** `-march`/`-mcpu` et la base arm64-apple-darwin de clang s'applique.

Mesures réelles, même modèle, même échantillon français, même machine :

| | Build M4 (`GGML_NATIVE=ON`) | Build RC (`GGML_NATIVE=OFF`) |
|---|---|---|
| Extensions CPU actives | `FP16_VA, MATMUL_INT8, DOTPROD, SME, SME2` | `FP16_VA, DOTPROD` |
| Metal | actif | **actif** (`MTL0`, Apple M4) |
| Temps Whisper | 18 103 ms | 19 877 ms (**+9,8 %**) |
| SRT produit | 4959 o | **4959 o, octet pour octet identique** |
| TXT produit | 3428 o | **3428 o, octet pour octet identique** |

Le coût est d'environ 10 % de temps de transcription, contre un binaire qui fonctionne sur toute la gamme Apple Silicon. Les sorties sont rigoureusement identiques : la portabilité ne dégrade pas la qualité. `-mcpu=native` ne sera pas réintroduit pour récupérer ces secondes.

Le script de build échoue désormais explicitement si `-mcpu=native` ou `-march=native` apparaît dans les fichiers de compilation générés, pour que la régression soit impossible à commettre silencieusement.

whisper.cpp reste épinglé à **v1.9.2**, commit `306c88f4d1286aec1bf96e544632897886af5501`. Aucune montée de version en M5.

## Compatibilité des données

| Scénario | Comportement |
|---|---|
| Utilisateur M4 qui met à jour | Modèle adopté automatiquement, aucun téléchargement |
| Nouvelle installation | Flow « Modèle requis » inchangé (M3) |
| Ancien modèle corrompu | Non migré ; flow « Modèle requis » |
| Deuxième lancement | Aucune action (idempotent) |
| Retour à une build `.dev` | Le modèle a été déplacé ; l'ancienne build le redemanderait. Compromis assumé : le déplacement atomique est plus sûr qu'une copie, et un retour en arrière n'est pas un scénario utilisateur du RC. |

## Stratégie d'échec

Aucune étape de la migration n'est bloquante pour le démarrage. Chaque échec est journalisé côté développeur et laisse l'application dans un état valide :

* répertoire de données non résolu → migration ignorée ;
* `rename` en échec → repli par copie vérifiée ; si la copie échoue aussi, le fichier temporaire est supprimé et l'ancien modèle reste intact ;
* destination inattendue après déplacement → journalisé, aucune purge des répertoires hérités.

Dans tous les cas le pire résultat est « l'utilisateur retombe sur l'écran Modèle requis », jamais une perte de données ni un modèle invalide déclaré prêt.

## Qualification

### Migration réelle (build empaqueté, identité production)

État de départ : modèle valide sous `.dev`, aucun répertoire de production. Protection non destructive : lien physique (hard link) créé sur l'inode du modèle avant l'essai, à coût disque nul.

```text
[st-ia] migration: legacy model hash=394221709c…a7e2 valid=true
[st-ia] migration: moved model to …/com.romainbourbon.stia/models/… (atomic rename)
[st-ia] migration: destination verified (574041195 bytes)
[st-ia] migration: removed empty …/com.romainbourbon.st-ia.dev/models
[st-ia] migration: removed empty …/com.romainbourbon.st-ia.dev
[st-ia] model detect: size=574041195 hash=394221709c…a7e2 valid=true
```

* SHA-256 après migration strictement identique à celui d'avant ;
* **inode inchangé** (`54315811`) : le fichier n'a pas été recopié, ce sont littéralement les mêmes octets ;
* **aucune connexion réseau** pendant toute l'opération (`lsof -p <pid> -a -i` vide) ;
* second lancement : `production model already present, nothing to do` — idempotence confirmée ;
* répertoires hérités vides effectivement purgés.

### Installation propre

Modèle écarté des deux emplacements : `no legacy model found, nothing to do`, état `Missing`, **aucune connexion réseau** tant que l'utilisateur n'a pas cliqué « Télécharger ». Le téléchargement complet lui-même reste couvert par la qualification M3 (ADR-004), le code réseau étant inchangé en M5 — seul l'emplacement de destination a changé, et il est prouvé par la migration ci-dessus.

### Gate humain — DaVinci Resolve

Validé par l'utilisateur sur un média réel de 60 minutes : fichier SRT accepté, piste de sous-titres créée, sous-titres présents sur toute la timeline, timecodes cohérents, synchronisation vérifiée en début, milieu et fin. Limite observée hors périmètre (vocabulaire technique/noms propres), non bloquante pour cette ADR.

### Icône de production

L'icône Tauri par défaut a été remplacée par l'asset ST-IA approuvé (`src/assets/ST-IA_icon.png`, source conservée telle quelle) via le générateur officiel `tauri icon`, qui produit les tailles macOS nécessaires (`32x32`, `128x128`, `128x128@2x`, `icon.icns` 1024 px, `icon.ico`) sans dépendance runtime. Vérifié visuellement sur le `.icns` extrait du `.app` empaqueté et sur celui monté depuis le `.dmg` : dans les deux cas l'icône ST-IA s'affiche, plus aucune trace du logo Tauri par défaut.

## Conséquences

* L'identifiant de production est figé avant tout chantier de signature ; la réserve ouverte par l'ADR-002 est close.
* La réserve de portabilité ouverte par M2/M4 est close et rendue non régressable par un garde-fou dans le script de build.
* Une mise à jour depuis une build M4 ne coûte aucun téléchargement à l'utilisateur.
* Le prix payé est d'environ 10 % de temps de transcription, documenté et assumé.
