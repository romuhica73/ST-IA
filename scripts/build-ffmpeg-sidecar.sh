#!/usr/bin/env bash
# Builds a standalone, statically-linked, minimal ffmpeg for
# aarch64-apple-darwin from official FFmpeg source, and copies it into
# src-tauri/binaries/ as the Tauri externalBin sidecar.
#
# Scope is intentionally narrow: demux/decode the audio track of
# mp4/mov/wav/mp3/m4a/flac inputs and encode 16kHz mono PCM16 WAV. No
# encoders/decoders beyond that are enabled, no GPL/nonfree components
# (see docs/third-party/FFMPEG.md), no network protocols.
#
# Pinned version: FFmpeg 9.0 (see docs/third-party/FFMPEG.md for source
# and checksum). This is a developer-only build step; the resulting
# binary is committed as a build artifact (see ADR-003).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_DIR="$ROOT_DIR/build-tmp"
FFMPEG_VERSION="9.0"
FFMPEG_TARBALL="ffmpeg-$FFMPEG_VERSION.tar.xz"
FFMPEG_URL="https://ffmpeg.org/releases/$FFMPEG_TARBALL"
FFMPEG_SHA256="7f607a00dd0d28a729d5a4811205812eef01cf6ef6155025febb6f36a9062d52"
SRC_DIR="$WORK_DIR/ffmpeg-$FFMPEG_VERSION"
OUT_DIR="$ROOT_DIR/src-tauri/binaries"
OUT_NAME="ffmpeg-aarch64-apple-darwin"

mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

if [ ! -f "$FFMPEG_TARBALL" ]; then
  echo "==> Downloading $FFMPEG_URL"
  curl -sL --max-time 120 -o "$FFMPEG_TARBALL" "$FFMPEG_URL"
fi

ACTUAL_SHA256="$(shasum -a 256 "$FFMPEG_TARBALL" | cut -d' ' -f1)"
if [ "$ACTUAL_SHA256" != "$FFMPEG_SHA256" ]; then
  echo "error: $FFMPEG_TARBALL checksum mismatch" >&2
  echo "  expected: $FFMPEG_SHA256" >&2
  echo "  actual:   $ACTUAL_SHA256" >&2
  exit 1
fi

if [ ! -d "$SRC_DIR" ]; then
  echo "==> Extracting $FFMPEG_TARBALL"
  tar xf "$FFMPEG_TARBALL"
fi

cd "$SRC_DIR"

echo "==> Configuring FFmpeg $FFMPEG_VERSION (static, minimal, LGPL)"
./configure \
  --disable-everything \
  --disable-doc \
  --disable-debug \
  --disable-network \
  --disable-autodetect \
  --disable-programs \
  --disable-libwebp \
  --enable-ffmpeg \
  --enable-protocol=file \
  --enable-demuxer=mov,mp3,wav,flac,aac \
  --enable-decoder=aac,mp3,mp3float,pcm_s16le,pcm_s16be,pcm_s24le,pcm_f32le,flac,alac \
  --enable-parser=aac,mpegaudio,flac \
  --enable-muxer=wav \
  --enable-encoder=pcm_s16le \
  --enable-filter=aresample,aformat,anull \
  --arch=arm64 \
  --target-os=darwin \
  --enable-static \
  --disable-shared \
  --disable-gpl \
  --disable-nonfree \
  --disable-version3 \
  --pkg-config-flags=--static \
  --prefix="$SRC_DIR/dist"

echo "==> Building (this takes a few minutes)"
make -j"$(sysctl -n hw.ncpu)"

BIN="$SRC_DIR/ffmpeg"

echo "==> Verifying architecture and linkage"
file "$BIN"
lipo -info "$BIN"
echo "--- otool -L ---"
otool -L "$BIN"

if otool -L "$BIN" | tail -n +2 | grep -qiE 'rpath|homebrew|/opt/'; then
  echo "error: ffmpeg has an unexpected dylib dependency (rpath/Homebrew)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
cp "$BIN" "$OUT_DIR/$OUT_NAME"
chmod +x "$OUT_DIR/$OUT_NAME"

echo "==> OK: $OUT_DIR/$OUT_NAME ($(du -h "$OUT_DIR/$OUT_NAME" | cut -f1))"
