#!/usr/bin/env bash
#
# build.sh - Build my-mac-tray as a macOS .app bundle ready for install.
#
# Usage:
#   ./script/build.sh              # debug build
#   ./script/build.sh --release    # release build (optimised)
#   ./script/build.sh --sign       # release + ad-hoc sign for local use
#   ./script/build.sh --dmg        # release + build a .dmg installer
#
set -euo pipefail

# ---- config ---------------------------------------------------------------
APP_NAME="MyMacTray"
BINARY_NAME="my-mac-tray"
BUNDLE_ID="com.frank-lan.my-mac-tray"
BUNDLE_VERSION="0.1.0"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RELEASE_FLAG=""
BUILD_DIR_NAME="debug"
CARGO_FLAGS=""
DO_RELEASE=false
DO_DMG=false
DO_SIGN=false

# ---- parse args -----------------------------------------------------------
if [ $# -eq 0 ]; then
  # default: release + sign + dmg
  DO_RELEASE=true
  DO_SIGN=true
  DO_DMG=true
else
  for arg in "$@"; do
    case "$arg" in
      --release) DO_RELEASE=true ;;
      --dmg)     DO_DMG=true ; DO_RELEASE=true ;;
      --sign)    DO_SIGN=true ; DO_RELEASE=true ;;
      --debug)   ;;  # explicit debug build, no extra flags
      *)
        echo "Unknown option: $arg"
        echo "Usage: $0 [--release] [--sign] [--dmg] [--debug]"
        exit 1
        ;;
    esac
  done
fi

if [ "$DO_RELEASE" = true ]; then
  RELEASE_FLAG="--release"
  BUILD_DIR_NAME="release"
fi

# ---- step 1: cargo build --------------------------------------------------
echo "==> Building $BINARY_NAME (${BUILD_DIR_NAME}) …"
cd "$PROJECT_DIR"
cargo build $RELEASE_FLAG

# Check for cross-compiled target directory first
if [ -x "$PROJECT_DIR/target/aarch64-apple-darwin/$BUILD_DIR_NAME/$BINARY_NAME" ]; then
  BINARY_PATH="$PROJECT_DIR/target/aarch64-apple-darwin/$BUILD_DIR_NAME/$BINARY_NAME"
else
  BINARY_PATH="$PROJECT_DIR/target/$BUILD_DIR_NAME/$BINARY_NAME"
fi

if [ ! -x "$BINARY_PATH" ]; then
  echo "ERROR: Binary not found"
  exit 1
fi

# ---- step 2: create .app bundle ------------------------------------------
APP_BUNDLE="$PROJECT_DIR/target/$BUILD_DIR_NAME/$APP_NAME.app"
echo "==> Creating app bundle at $APP_BUNDLE …"
rm -rf "$APP_BUNDLE"

mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# ---- step 3: Info.plist --------------------------------------------------
cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>$BUNDLE_VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$BUNDLE_VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF

echo "  Info.plist written (LSUIElement = true → no dock icon)"

# ---- step 4: ad-hoc code sign (optional) ----------------------------------
if [ "$DO_SIGN" = true ]; then
  echo "==> Ad-hoc signing $APP_NAME.app …"
  codesign --force --deep --sign - "$APP_BUNDLE"
  echo "  Signed (ad-hoc)."
fi

echo ""
echo "==> Bundle ready: $APP_BUNDLE"
echo "  Run with: open '$APP_BUNDLE'"
echo "  Or drag to /Applications:  cp -R '$APP_BUNDLE' /Applications/"

# ---- step 5: .dmg installer (optional) ------------------------------------
if [ "$DO_DMG" = true ]; then
  DMG_PATH="$PROJECT_DIR/target/$BUILD_DIR_NAME/$APP_NAME-$BUNDLE_VERSION.dmg"
  DMG_STAGING="$PROJECT_DIR/target/$BUILD_DIR_NAME/dmg-staging"

  echo "==> Creating .dmg at $DMG_PATH …"
  rm -rf "$DMG_STAGING"
  mkdir -p "$DMG_STAGING"
  cp -R "$APP_BUNDLE" "$DMG_STAGING/"
  ln -s /Applications "$DMG_STAGING/Applications"

  # Determine the volume icon (AppKit icon) if available
  if [ -f "$PROJECT_DIR/assets/icon.icns" ]; then
    cp "$PROJECT_DIR/assets/icon.icns" "$DMG_STAGING/.VolumeIcon.icns"
  fi

  hdiutil create -volname "$APP_NAME" \
    -srcfolder "$DMG_STAGING" \
    -ov -format UDZO \
    "$DMG_PATH"

  rm -rf "$DMG_STAGING"

  if [ "$DO_SIGN" = true ]; then
    echo "==> Signing .dmg …"
    codesign --force --sign - "$DMG_PATH"
  fi

  echo ""
  echo "==> DMG (size: $(du -sh "$DMG_PATH" | cut -f1)) ready: $DMG_PATH"
fi

echo ""
echo "==> Done."