#!/usr/bin/env bash
# DEVELOPER-ONLY tool: copies an already-downloaded Whisper model into the
# canonical location ST-IA resolves at runtime (Application Support), so M2
# can be tested before M3 implements real in-app download/verification.
#
# This is NOT a user-facing feature: it never runs automatically, never
# downloads anything itself, and never searches the filesystem for a model —
# it only copies the exact source path you pass it.
#
# Usage: scripts/provision-dev-model.sh /path/to/ggml-large-v3-turbo-q5_0.bin
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_NAME="ggml-large-v3-turbo-q5_0.bin"

if [ $# -ne 1 ]; then
  echo "usage: $0 /path/to/$MODEL_NAME" >&2
  exit 1
fi

SRC="$1"
if [ ! -f "$SRC" ]; then
  echo "error: $SRC does not exist or is not a file" >&2
  exit 1
fi

IDENTIFIER="$(jq -r '.identifier' "$ROOT_DIR/src-tauri/tauri.conf.json")"
DEST_DIR="$HOME/Library/Application Support/$IDENTIFIER/models"
DEST="$DEST_DIR/$MODEL_NAME"

mkdir -p "$DEST_DIR"
cp "$SRC" "$DEST"

echo "==> Copied to $DEST ($(du -h "$DEST" | cut -f1))"
echo "==> This is a dev-only shortcut; M3 will implement real download/verification."
