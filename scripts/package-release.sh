#!/usr/bin/env bash
# Collects the macOS release artifacts for ST-IA into release-artifacts/,
# under deterministic names, with checksums — and audits what it collected.
#
#   scripts/package-release.sh            # package an existing build
#   scripts/package-release.sh --build    # run `pnpm tauri build` first
#
# Produces:
#   release-artifacts/ST-IA-<version>-macos-arm64.dmg
#   release-artifacts/ST-IA-<version>-macos-arm64.app.tar.gz
#   release-artifacts/SHA256SUMS.txt
#
# The version is read from tauri.conf.json — the same source of truth the
# version-consistency test pins package.json and Cargo.toml against. Nothing
# here hardcodes a version number.
#
# NOT signed and NOT notarized: out of scope for M9. The artifacts are
# reproducible locally and suitable for a GitHub release *once signing is
# resolved* — see docs/release/RELEASE_CHECKLIST.md.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OUT_DIR="$ROOT_DIR/release-artifacts"
BUNDLE_DIR="$ROOT_DIR/src-tauri/target/release/bundle"
ARCH="arm64"
PLATFORM="macos"

die() { echo "error: $*" >&2; exit 1; }
step() { echo; echo "==> $*"; }

command -v jq >/dev/null || die "jq is required (brew install jq)"
command -v shasum >/dev/null || die "shasum is required"

[ "$(uname -s)" = "Darwin" ] || die "macOS only — ST-IA ships for Apple Silicon exclusively"
[ "$(uname -m)" = "arm64" ] || die "this script builds the Apple Silicon artifact and must run on arm64"

# --- Version: one source of truth, cross-checked -------------------------

VERSION="$(jq -r '.version' src-tauri/tauri.conf.json)"
[ -n "$VERSION" ] && [ "$VERSION" != "null" ] || die "could not read version from tauri.conf.json"

PACKAGE_VERSION="$(jq -r '.version' package.json)"
CARGO_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' src-tauri/Cargo.toml | head -1)"
[ "$VERSION" = "$PACKAGE_VERSION" ] || die "version mismatch: tauri.conf.json=$VERSION package.json=$PACKAGE_VERSION"
[ "$VERSION" = "$CARGO_VERSION" ] || die "version mismatch: tauri.conf.json=$VERSION Cargo.toml=$CARGO_VERSION"

BASENAME="ST-IA-$VERSION-$PLATFORM-$ARCH"
echo "==> ST-IA $VERSION ($PLATFORM $ARCH)"

# --- Build (opt-in) -------------------------------------------------------

if [ "${1:-}" = "--build" ]; then
  step "Building (pnpm tauri build)"
  pnpm tauri build
elif [ $# -gt 0 ]; then
  die "unknown argument: $1 (expected --build or nothing)"
fi

# --- Locate what the bundler produced ------------------------------------

APP_PATH="$BUNDLE_DIR/macos/ST-IA.app"
[ -d "$APP_PATH" ] || die "no app bundle at $APP_PATH — run: pnpm tauri build"

# Tauri names the DMG ST-IA_<version>_aarch64.dmg; match on the pattern
# rather than the exact string so a bundler rename does not silently produce
# an empty release.
DMG_SRC="$(find "$BUNDLE_DIR/dmg" -maxdepth 1 -name '*.dmg' -print -quit 2>/dev/null || true)"
[ -n "$DMG_SRC" ] || die "no .dmg under $BUNDLE_DIR/dmg — run: pnpm tauri build"

# --- Verify the build matches the version we are naming it with ----------

PLIST_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP_PATH/Contents/Info.plist" 2>/dev/null || true)"
[ "$PLIST_VERSION" = "$VERSION" ] || die "built app reports version '$PLIST_VERSION' but we are packaging '$VERSION' — rebuild"

BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP_PATH/Contents/Info.plist" 2>/dev/null || true)"
EXPECTED_ID="$(jq -r '.identifier' src-tauri/tauri.conf.json)"
[ "$BUNDLE_ID" = "$EXPECTED_ID" ] || die "bundle identifier is '$BUNDLE_ID', expected '$EXPECTED_ID'"

# --- Audit the app bundle before shipping it (§44, §45) ------------------

step "Auditing $APP_PATH"

audit_fail=0
audit() {
  local label="$1"; shift
  if "$@"; then
    echo "    ok   — $label"
  else
    echo "    FAIL — $label" >&2
    audit_fail=1
  fi
}

no_match() { [ -z "$(find "$APP_PATH" \( "$@" \) -print -quit)" ]; }

# The 574 MB Whisper model is downloaded on first use into Application
# Support; it must never be inside the app, the DMG or the repository.
audit "no Whisper model embedded"        no_match -name '*.bin'
audit "no test media embedded"           no_match -name '*.MOV' -o -name '*.mov' -o -name '*.mp4' -o -name '*.wav'
audit "no environment file embedded"     no_match -name '.env' -o -name '.env.*'
audit "no log or crash dump embedded"    no_match -name '*.log' -o -name '*.crash'
audit "no signing material embedded"     no_match -name '*.p12' -o -name '*.pem' -o -name '*.key' -o -name '*.mobileprovision'
audit "no source map embedded"           no_match -name '*.map'

audit "third-party notices present" test -f "$APP_PATH/Contents/Resources/THIRD_PARTY_NOTICES.md"
audit "licences directory present"  test -d "$APP_PATH/Contents/Resources/licenses"
audit "app icon present"            test -f "$APP_PATH/Contents/Resources/icon.icns"
audit "ffmpeg sidecar present"      test -x "$APP_PATH/Contents/MacOS/ffmpeg"
audit "whisper-cli sidecar present" test -x "$APP_PATH/Contents/MacOS/whisper-cli"

# The build machine's absolute paths have no business in a shipped binary.
audit "no developer path in Info.plist" \
  bash -c '! grep -qE "/Users/|/Volumes/Workspace" "'"$APP_PATH"'/Contents/Info.plist"'

[ "$audit_fail" -eq 0 ] || die "artifact audit failed — not packaging"

# --- Collect --------------------------------------------------------------

step "Collecting into release-artifacts/"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cp "$DMG_SRC" "$OUT_DIR/$BASENAME.dmg"
echo "    $BASENAME.dmg"

# Advanced/optional artifact (§39): a plain archive for users who prefer not
# to mount a disk image. `ditto` preserves the bundle's symlinks, resource
# forks and the executable bit, which `tar` alone does not reliably do for a
# macOS .app.
ditto -c -k --sequesterRsrc --keepParent "$APP_PATH" "$OUT_DIR/$BASENAME.app.zip"
echo "    $BASENAME.app.zip"

# --- Checksums ------------------------------------------------------------

step "Generating SHA256SUMS.txt"

(
  cd "$OUT_DIR"
  shasum -a 256 "$BASENAME.dmg" "$BASENAME.app.zip" > SHA256SUMS.txt
  # Verify what we just wrote, rather than trusting it (§42).
  shasum -a 256 --check --status SHA256SUMS.txt || exit 1
)
echo "    verified"

# --- Summary --------------------------------------------------------------

step "Done — release-artifacts/"
( cd "$OUT_DIR" && ls -lh && echo && cat SHA256SUMS.txt )

cat <<EOF

These artifacts are UNSIGNED and NOT NOTARIZED (PUBLIC_DISTRIBUTION_SIGNING_PENDING).
macOS Gatekeeper will refuse them on another machine until that is resolved.
Do not attach them to a public GitHub release as-is.
EOF
