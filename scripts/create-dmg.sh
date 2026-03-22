#!/bin/bash

set -e

APP_NAME="Rust Redis Desktop"
VERSION="${1:-0.1.0}"
ARCH="${2:-x86_64}"

APP_DIR="${APP_NAME}.app"
DMG_NAME="rust-redis-desktop-${ARCH}.dmg"

if [ ! -d "$APP_DIR" ]; then
    echo "Error: $APP_DIR not found. Run build-app.sh first."
    exit 1
fi

rm -f "$DMG_NAME"

if command -v create-dmg &> /dev/null; then
    CREATE_DMG_ARGS=(
        --volname "$APP_NAME" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "$APP_DIR" 150 190 \
        --hide-extension "$APP_DIR" \
        --app-drop-link 450 190
    )

    if [ -f "Assets/AppIcon.icns" ]; then
        CREATE_DMG_ARGS+=(--volicon "Assets/AppIcon.icns")
    fi

    create-dmg \
        "${CREATE_DMG_ARGS[@]}" \
        "$DMG_NAME" \
        "$APP_DIR" || true
fi

if [ ! -f "$DMG_NAME" ]; then
    echo "Creating DMG with hdiutil..."
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$APP_DIR" \
        -ov -format UDZO \
        "$DMG_NAME"
fi

echo "Created $DMG_NAME"

if [ -n "$APPLE_ID" ] && [ -n "$APPLE_TEAM_ID" ] && [ -n "$APPLE_APP_PASSWORD" ]; then
    echo "Notarizing DMG..."
    xcrun notarytool submit "$DMG_NAME" \
        --apple-id "$APPLE_ID" \
        --team-id "$APPLE_TEAM_ID" \
        --password "$APPLE_APP_PASSWORD" \
        --wait \
        --timeout 10m

    echo "Stapling notarization..."
    xcrun stapler staple "$DMG_NAME"

    echo "DMG notarized and stapled successfully"
else
    echo "Skipping notarization (Apple credentials not set)"
fi

echo "Package ready: $DMG_NAME"
