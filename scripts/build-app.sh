#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Foldbar"
BIN_NAME="foldbar"
PROFILE="debug"
ARCH="$(uname -m)"

usage() {
  echo "usage: $0 [--release] [arm64|x86_64]" >&2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      PROFILE="release"
      ;;
    arm64|x86_64)
      ARCH="$1"
      ;;
    *)
      usage
      exit 2
      ;;
  esac
  shift
done

case "$ARCH" in
  arm64)
    TARGET_TRIPLE="aarch64-apple-darwin"
    ;;
  x86_64)
    TARGET_TRIPLE="x86_64-apple-darwin"
    ;;
  *)
    echo "unsupported arch: $ARCH" >&2
    exit 2
    ;;
esac

DIST_DIR="$ROOT_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
DMG_DIR="$DIST_DIR/dmg"
DMG_PATH="$DIST_DIR/$BIN_NAME-$ARCH.dmg"

BUILD_ARGS=(--manifest-path "$ROOT_DIR/Cargo.toml" --target "$TARGET_TRIPLE")
if [[ "$PROFILE" == "release" ]]; then
  BUILD_ARGS=(--release "${BUILD_ARGS[@]}")
fi

echo "Building $APP_NAME for $ARCH ($PROFILE)..."
cargo build "${BUILD_ARGS[@]}"

BINARY="$ROOT_DIR/target/$TARGET_TRIPLE/$PROFILE/$BIN_NAME"

echo "Creating app bundle..."
rm -rf "$APP_DIR" "$DMG_DIR" "$DMG_PATH"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"
cp "$BINARY" "$MACOS_DIR/$BIN_NAME"
cp "$ROOT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"
cp "$ROOT_DIR/assets/Foldbar.icns" "$RESOURCES_DIR/Foldbar.icns"

if [[ -n "${FOLDBAR_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --options runtime --sign "$FOLDBAR_CODESIGN_IDENTITY" "$APP_DIR"
else
  codesign --force --sign - "$APP_DIR"
fi

echo "Creating DMG..."
mkdir -p "$DMG_DIR"
cp -R "$APP_DIR" "$DMG_DIR/"
ln -s /Applications "$DMG_DIR/Applications"
hdiutil create -volname "$APP_NAME" \
  -srcfolder "$DMG_DIR" \
  -ov -format UDZO \
  "$DMG_PATH"
rm -rf "$DMG_DIR"

echo "$APP_DIR"
echo "$DMG_PATH"
