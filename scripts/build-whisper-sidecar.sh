#!/usr/bin/env bash
# Builds a standalone, statically-linked whisper-cli for aarch64-apple-darwin
# from the pinned whisper.cpp clone at engine/whisper.cpp, and copies it into
# src-tauri/binaries/ as the Tauri externalBin sidecar.
#
# Pinned version: whisper.cpp v1.9.2, commit 306c88f4d1286aec1bf96e544632897886af5501
# (see docs/architecture/ADR-001-transcription-engine.md and ADR-003).
#
# Requirements: the clone at engine/whisper.cpp (gitignored, see .gitignore)
# checked out at the pinned commit above. This is a developer-only build step;
# the resulting binary is committed as a build artifact (see ADR-003) since
# whisper.cpp has no official prebuilt static arm64 binary distribution.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENGINE_DIR="$ROOT_DIR/engine/whisper.cpp"
BUILD_DIR="$ENGINE_DIR/build-static"
PINNED_SHA="306c88f4d1286aec1bf96e544632897886af5501"
OUT_DIR="$ROOT_DIR/src-tauri/binaries"
OUT_NAME="whisper-cli-aarch64-apple-darwin"

if [ ! -d "$ENGINE_DIR/.git" ]; then
  echo "error: $ENGINE_DIR is not a git checkout. Clone ggml-org/whisper.cpp there first:" >&2
  echo "  git clone https://github.com/ggml-org/whisper.cpp engine/whisper.cpp" >&2
  echo "  git -C engine/whisper.cpp checkout $PINNED_SHA" >&2
  exit 1
fi

ACTUAL_SHA="$(git -C "$ENGINE_DIR" rev-parse HEAD)"
if [ "$ACTUAL_SHA" != "$PINNED_SHA" ]; then
  echo "error: engine/whisper.cpp is at $ACTUAL_SHA, expected pinned $PINNED_SHA" >&2
  echo "  git -C engine/whisper.cpp checkout $PINNED_SHA" >&2
  exit 1
fi

# GGML_NATIVE=OFF is the portability switch, and it is not optional for a
# distributable build. With it ON (ggml's default), ggml asks clang to resolve
# -mcpu=native and bakes the *build machine's* CPU into the binary — on an M4
# that enables SME/SME2, which do not exist on M1/M2/M3 and would fault there.
# With it OFF, ggml emits no -march/-mcpu at all and clang's arm64-apple-darwin
# baseline applies, which still gives NEON/FMA/FP16/DOTPROD on every Apple
# Silicon generation. See ADR-006 for the measured trade-off.
echo "==> Configuring whisper.cpp (static, Metal, arm64, portable) at $ACTUAL_SHA"
rm -rf "$BUILD_DIR"
cmake -B "$BUILD_DIR" -S "$ENGINE_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DGGML_STATIC=ON \
  -DGGML_NATIVE=OFF \
  -DGGML_METAL=ON \
  -DGGML_METAL_EMBED_LIBRARY=ON \
  -DWHISPER_BUILD_EXAMPLES=ON \
  -DWHISPER_BUILD_TESTS=OFF \
  -DWHISPER_BUILD_SERVER=OFF \
  -DCMAKE_OSX_ARCHITECTURES=arm64

echo "==> Building whisper-cli (Release)"
cmake --build "$BUILD_DIR" --config Release --target whisper-cli -j"$(sysctl -n hw.ncpu)"

BIN="$BUILD_DIR/bin/whisper-cli"

echo "==> Verifying no build-machine-specific CPU targeting"
if grep -rqE 'mcpu=native|march=native' "$BUILD_DIR" --include=flags.make --include=link.txt; then
  echo "error: build used -mcpu=native/-march=native; binary would not be portable" >&2
  grep -rnE 'mcpu=native|march=native' "$BUILD_DIR" --include=flags.make --include=link.txt >&2
  exit 1
fi

echo "==> Verifying architecture and linkage"
file "$BIN"
lipo -info "$BIN"
echo "--- otool -L ---"
otool -L "$BIN"

if otool -L "$BIN" | tail -n +2 | grep -qiE 'rpath|homebrew|engine/whisper\.cpp'; then
  echo "error: whisper-cli has an unexpected dylib dependency (rpath/Homebrew/dev clone)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
cp "$BIN" "$OUT_DIR/$OUT_NAME"
chmod +x "$OUT_DIR/$OUT_NAME"

echo "==> OK: $OUT_DIR/$OUT_NAME ($(du -h "$OUT_DIR/$OUT_NAME" | cut -f1))"
