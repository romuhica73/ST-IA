# FFmpeg — sidecar tiers et licence

## Version et provenance

* Version : `9.0`
* Source officielle : `https://ffmpeg.org/releases/ffmpeg-9.0.tar.xz`
* SHA-256 (vérifié par `scripts/build-ffmpeg-sidecar.sh`) : `7f607a00dd0d28a729d5a4811205812eef01cf6ef6155025febb6f36a9062d52`

## Configuration de build

Build statique, minimal, arm64 (`aarch64-apple-darwin`), sans GPL ni composants non-free :

```text
--disable-everything
--disable-doc --disable-debug --disable-network --disable-autodetect --disable-programs
--enable-ffmpeg
--enable-protocol=file
--enable-demuxer=mov,mp3,wav,flac,aac
--enable-decoder=aac,mp3,mp3float,pcm_s16le,pcm_s16be,pcm_s24le,pcm_f32le,flac,alac
--enable-parser=aac,mpegaudio,flac
--enable-muxer=wav
--enable-encoder=pcm_s16le
--enable-filter=aresample,aformat,anull
--arch=arm64 --target-os=darwin
--enable-static --disable-shared
--disable-gpl --disable-nonfree --disable-version3
```

Le binaire produit ne sait faire que : démuxer MP4/MOV/M4A (conteneur `mov`), MP3, WAV, FLAC ; décoder AAC, MP3, PCM (plusieurs variantes), FLAC, ALAC ; encoder en PCM 16 bits ; écrire un conteneur WAV. Aucun décodeur/encodeur vidéo, aucun protocole réseau, aucun composant GPL/non-free activé.

Note : la configuration active aussi un petit nombre de filtres vidéo (`crop`, `rotate`, `transpose`, `hflip`, `vflip`, `format`, `null`, `trim`) que `--enable-filter` ne permet pas d'exclure individuellement — ils sont référencés en dur par l'outil `ffmpeg` lui-même (gestion de la métadonnée de rotation automatique) et compilés dès que `avfilter` est activé. Ils ne sont jamais invoqués par ST-IA (aucun flux vidéo n'est jamais décodé, `-vn` retire la vidéo avant tout traitement) et n'affectent pas la licence.

## Licence observée

Le build affiche explicitement :

```text
License: LGPL version 2.1 or later
```

`--disable-gpl`, `--disable-nonfree` et `--disable-version3` sont passés explicitement pour éviter toute activation accidentelle d'un composant sous licence incompatible (ex. x264, qui est GPL). Aucun composant tiers externe (libx264, libmp3lame, libfdk-aac, etc.) n'est lié — uniquement les décodeurs/encodeurs internes de FFmpeg.

## Obligations avant distribution publique

Cette mission ne conclut PAS la conformité légale complète de la distribution. Points à traiter avant une distribution publique (hors périmètre M2) :

* Inclure le texte de la licence LGPL 2.1+ dans l'application distribuée.
* Documenter que le binaire FFmpeg est un composant tiers distinct, avec attribution au projet FFmpeg (`https://ffmpeg.org`).
* LGPL permet la liaison statique dans une app propriétaire à condition de fournir un moyen de remplacer/relier la bibliothèque LGPL (ex. objets `.o` ou possibilité de recompiler) — à confirmer avec un avis juridique avant publication si cela s'applique à notre mode de distribution en binaire unique.
* Revalider cette configuration à chaque changement de version FFmpeg.

Aucune conclusion juridique définitive n'est affirmée ici au-delà de ce qui est objectivement observable (licence annoncée par le build, options passées).

## Build reproductible

Voir `scripts/build-ffmpeg-sidecar.sh`. Le script télécharge la source officielle, vérifie son SHA-256, compile, vérifie l'architecture et l'absence de dépendance dylib non-système (`otool -L`), puis copie le binaire vers `src-tauri/binaries/ffmpeg-aarch64-apple-darwin`.
