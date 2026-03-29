#!/bin/bash

set -e

APP_NAME="Rust Redis Desktop"
BUNDLE_ID="dev.yelog.rust-redis-desktop"
VERSION="${1:-0.1.0}"
BUILD_NUMBER="${2:-1}"
TARGET="${3:-}"
INCLUDE_SPARKLE="${4:-true}"

if [ -z "$TARGET" ]; then
    TARGET=$(rustc -vV | grep host | cut -d' ' -f2)
fi

echo "Building $APP_NAME v$VERSION ($BUILD_NUMBER) for $TARGET"

cargo build --release --target "$TARGET"

APP_DIR="${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
FRAMEWORKS_DIR="${CONTENTS_DIR}/Frameworks"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"
mkdir -p "$FRAMEWORKS_DIR"

cp "target/$TARGET/release/rust-redis-desktop" "$MACOS_DIR/"

sed -e "s/\${VERSION}/$VERSION/" \
    -e "s/\${BUILD_NUMBER}/$BUILD_NUMBER/" \
    Info.plist > "$CONTENTS_DIR/Info.plist"

if [ -f "Assets/AppIcon.icns" ]; then
    cp Assets/AppIcon.icns "$RESOURCES_DIR/"
fi

# 嵌入 Sparkle Framework (用于自动更新)
if [ "$INCLUDE_SPARKLE" = "true" ] && [ -d "Frameworks/Sparkle.framework" ]; then
    echo "嵌入 Sparkle Framework..."
    cp -R "Frameworks/Sparkle.framework" "$FRAMEWORKS_DIR/"
    
    # 确保 Autoupdate.app 存在
    AUTOUUPDATE_APP="$FRAMEWORKS_DIR/Sparkle.framework/Versions/A/Resources/Autoupdate.app"
    if [ -f "$AUTOUUPDATE_APP" ]; then
        echo "Autoupdate.app 已嵌入"
    fi
fi

echo "Created $APP_DIR"

if [ "$(uname)" = "Darwin" ]; then
    if [ -n "$APPLE_SIGNING_IDENTITY" ]; then
        echo "Signing app..."
        codesign --force --deep --sign "$APPLE_SIGNING_IDENTITY" \
            --options runtime \
            --entitlements entitlements.plist \
            --timestamp \
            "$APP_DIR"
        echo "App signed successfully"
    else
        echo "Skipping code signing (APPLE_SIGNING_IDENTITY not set)"
    fi
fi

echo "Build complete: $APP_DIR"