# Construire ST-IA depuis les sources

## Plateforme supportée

**macOS sur Apple Silicon (arm64) uniquement.**

C'est la seule cible qualifiée. Le code n'a rien de spécifiquement hostile aux autres
plateformes, mais :

* les sidecars committés (`src-tauri/binaries/`) sont des binaires **arm64 macOS** —
  ils ne fonctionneront ni sur Intel, ni sur Windows, ni sur Linux ;
* le backend d'accélération est **Metal** ;
* rien d'autre n'est testé ni construit.

Open source ne veut pas dire multiplateforme. Il n'y a **aucun** build Windows ou Linux
supporté, et aucun n'est prévu à court terme.

---

## Prérequis

| Outil | Version de référence | Installation |
|---|---|---|
| macOS | 26.5 (Apple Silicon) | — |
| Xcode Command Line Tools | — | `xcode-select --install` |
| Node.js | 20.20 LTS | [nodejs.org](https://nodejs.org) ou `brew install node@20` |
| pnpm | 10.34 | `corepack enable pnpm` ou `brew install pnpm` |
| Rust | 1.96 (stable) | [rustup.rs](https://rustup.rs) |

Les versions ci-dessus sont celles utilisées pour qualifier la build. Des versions plus
récentes fonctionnent en principe ; c'est le socle vérifié.

pnpm est requis : le projet livre un `pnpm-lock.yaml`, pas un `package-lock.json`.
Utiliser npm ou yarn produirait un arbre de dépendances différent de celui qui a été
audité.

Aucune dépendance Homebrew n'est nécessaire **au runtime** : les deux sidecars sont
statiques et ne lient que des frameworks système Apple (vérifiable avec `otool -L`).

---

## Construire

```sh
git clone https://github.com/romuhica73/ST-IA.git
cd ST-IA

pnpm install --frozen-lockfile
```

### Développement

```sh
pnpm tauri dev
```

Au premier lancement, l'application demande le téléchargement du modèle Whisper
(574 Mo, une seule fois, sur clic explicite). Il est stocké dans
`~/Library/Application Support/com.romainbourbon.stia/models/`.

### Release empaquetée

```sh
pnpm tauri build
```

Produit :

* `src-tauri/target/release/bundle/macos/ST-IA.app`
* `src-tauri/target/release/bundle/dmg/ST-IA_0.1.0_aarch64.dmg`

**Le résultat n'est ni signé ni notarisé.** Au premier lancement, macOS Gatekeeper le
bloquera : clic droit → **Ouvrir**, puis confirmer. La signature et la notarisation sont
prévues pour la première release publique, pas pour une build locale.

---

## Tests

```sh
pnpm build                        # tsc + vite (échoue sur toute erreur de type)
pnpm test                         # Vitest — logique pure, pas de DOM

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                        # tests unitaires + cohérence de version
```

Le test `version_consistency` vérifie que `package.json`, `Cargo.toml` et
`tauri.conf.json` déclarent la même version. Un test Vitest vérifie la parité stricte
des clés entre les catalogues FR et EN.

---

## Reconstruire les sidecars (rarement nécessaire)

`src-tauri/binaries/` contient deux exécutables committés :

| Binaire | Version | Source | Licence |
|---|---|---|---|
| `ffmpeg-aarch64-apple-darwin` | FFmpeg 9.0 | [ffmpeg.org/releases](https://ffmpeg.org/releases/) | LGPL-2.1 |
| `whisper-cli-aarch64-apple-darwin` | whisper.cpp v1.9.2 | [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT |

Ils sont committés parce que whisper.cpp ne distribue pas de binaire arm64 statique
officiel, et parce qu'un contributeur ne devrait pas avoir à construire FFmpeg pour
lancer l'application. Voir
[ADR-003](architecture/ADR-003-local-transcription-pipeline.md) et
`docs/security/M8_SECURITY_REVIEW.md` (STIA-SEC-011) pour le compromis assumé.

**Vous n'avez pas besoin de les reconstruire pour développer.** Ne le faites que si vous
changez délibérément de version.

### FFmpeg

```sh
scripts/build-ffmpeg-sidecar.sh
```

Télécharge la source officielle, **vérifie son SHA-256**, et configure une build
volontairement minimale : `--disable-everything --disable-network`, un seul protocole
(`file`), 5 demuxers, 9 décodeurs, 1 muxer, 1 encodeur. Pas de composant GPL ni nonfree.
Le script échoue si le binaire produit dépend d'un rpath ou de Homebrew.

Prérequis supplémentaires : `nasm` ou `yasm` (`brew install nasm`), `pkg-config`.

### whisper.cpp

Le clone est *gitignoré* — il n'est pas vendorisé (il embarque des modèles multi-Go).

```sh
git clone https://github.com/ggml-org/whisper.cpp engine/whisper.cpp
git -C engine/whisper.cpp checkout 306c88f4d1286aec1bf96e544632897886af5501
scripts/build-whisper-sidecar.sh
```

Prérequis supplémentaire : `cmake` (`brew install cmake`).

Le script **refuse de construire** si le clone n'est pas exactement au commit épinglé,
et vérifie que la build n'a pas utilisé `-mcpu=native` : cela embarquerait le CPU de la
machine de build (SME/SME2 sur M4) dans un binaire qui planterait sur M1/M2/M3.
Voir [ADR-006](architecture/ADR-006-release-identity-and-data-migration.md).

Après toute reconstruction, vérifiez la portabilité :

```sh
file   src-tauri/binaries/whisper-cli-aarch64-apple-darwin
otool -L src-tauri/binaries/whisper-cli-aarch64-apple-darwin   # frameworks Apple uniquement
shasum -a 256 src-tauri/binaries/*
```

---

## Résolution de problèmes

**`pnpm tauri dev` échoue sur le port 1420.** Le port est fixe et strict (Vite
`strictPort`). Libérez-le : `lsof -ti:1420 | xargs kill`.

**« Modèle requis » alors que le modèle est installé.** L'intégrité est vérifiée par
taille **et** SHA-256 ; un fichier tronqué ou d'une autre variante est signalé
« endommagé ». Supprimez
`~/Library/Application Support/com.romainbourbon.stia/models/` et retéléchargez.

**Gatekeeper bloque `ST-IA.app`.** Attendu — build non signée. Clic droit → **Ouvrir**.

**Le build Rust est très long la première fois.** ~500 crates transitives (Tauri,
WebKit, rustls). Les builds suivantes sont incrémentales.
