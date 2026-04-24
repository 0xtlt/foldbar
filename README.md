# Foldbar

Foldbar is a tiny macOS menu bar utility written in Rust. It reproduces the core
Hidden Bar interaction: a movable separator, a toggle item, and collapse/expand
behavior for menu bar icons placed between them.

## Usage

1. Start Foldbar.
2. Hold `Command` and drag the `|` separator and `‹` toggle item in the menu bar.
3. Put the menu bar icons you want to hide between `|` and `‹`.
4. Click `‹` to collapse the hidden section.
5. Click `›` to expand it again.

The separator has a minimal menu with `Launch at Login` and `Quit Foldbar`.

## Development

Requirements:

- macOS
- Rust
- Xcode command line tools

Build the binary:

```sh
cargo build
```

Build the local app bundle:

```sh
./scripts/build-app.sh
```

Run the local app bundle:

```sh
open dist/Foldbar.app
```

Build the release app bundle:

```sh
./scripts/build-app.sh --release
```

For distribution, sign the app with a Developer ID Application certificate:

```sh
FOLDBAR_CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
  ./scripts/build-app.sh --release
```

Then zip and notarize the app before sharing it:

```sh
ditto -c -k --keepParent dist/Foldbar.app dist/Foldbar.zip
xcrun notarytool submit dist/Foldbar.zip --keychain-profile YOUR_PROFILE --wait
xcrun stapler staple dist/Foldbar.app
```

Release builds create `dist/Foldbar.app` and `dist/foldbar-<arch>.dmg`.
GitHub Releases build, sign, notarize, staple, and upload DMGs for `arm64` and
`x86_64`.

Run checks:

```sh
cargo check
```

## How It Works

Foldbar uses AppKit through `objc2` bindings. It creates two `NSStatusItem`s:

- a variable-width toggle item showing `‹` or `›`
- a separator item showing `|`

macOS already supports `Command`-drag reordering for status items. Foldbar assigns
stable `autosaveName` values so AppKit can persist their positions.

Collapse/expand is implemented by changing the separator item's length. In the
expanded state the separator is short. In the collapsed state the separator is
wide enough to push the items between the separator and toggle out of the
visible menu bar area.

## Current Scope

This is v1 and intentionally small:

- no preferences window
- no global hotkey
- no auto-hide timer
- no App Store packaging
- no special notch or multi-screen behavior beyond using the main screen width
