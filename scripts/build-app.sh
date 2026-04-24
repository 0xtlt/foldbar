#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/target/debug/Foldbar.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"

cargo build --manifest-path "$ROOT_DIR/Cargo.toml"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
cp "$ROOT_DIR/target/debug/foldbar" "$MACOS_DIR/foldbar"
cp "$ROOT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

echo "$APP_DIR"
