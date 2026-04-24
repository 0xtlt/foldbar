#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="debug"

if [[ "${1:-}" == "--release" ]]; then
  PROFILE="release"
  shift
fi

if [[ $# -gt 0 ]]; then
  echo "usage: $0 [--release]" >&2
  exit 2
fi

APP_DIR="$ROOT_DIR/target/$PROFILE/Foldbar.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
BINARY="$ROOT_DIR/target/$PROFILE/foldbar"

if [[ "$PROFILE" == "release" ]]; then
  cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"
else
  cargo build --manifest-path "$ROOT_DIR/Cargo.toml"
fi

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
cp "$BINARY" "$MACOS_DIR/foldbar"
cp "$ROOT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

if [[ -n "${FOLDBAR_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --options runtime --sign "$FOLDBAR_CODESIGN_IDENTITY" "$APP_DIR"
else
  codesign --force --sign - "$APP_DIR"
fi

echo "$APP_DIR"
