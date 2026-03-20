#!/bin/bash

set -e

ARCH="x86_64"
VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
fi

VERSION="${VERSION#v}"

APP_DIR="rust-redis-desktop.AppDir"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APP_DIR/usr/share/metainfo"

cargo build --release

cp target/release/rust-redis-desktop "$APP_DIR/usr/bin/"
chmod +x "$APP_DIR/usr/bin/rust-redis-desktop"

cat > "$APP_DIR/usr/share/applications/rust-redis-desktop.desktop" << 'EOF'
[Desktop Entry]
Name=Rust Redis Desktop
Comment=A Redis desktop manager written in Rust
Exec=rust-redis-desktop
Icon=rust-redis-desktop
Terminal=false
Type=Application
Categories=Development;Database;
StartupWMClass=rust-redis-desktop
EOF

cat > "$APP_DIR/usr/share/metainfo/rust-redis-desktop.metainfo.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>dev.yelog.rust-redis-desktop</id>
  <name>Rust Redis Desktop</name>
  <summary>A Redis desktop manager written in Rust</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <description>
    <p>A modern Redis desktop manager built with Rust, Dioxus, and Freya.</p>
  </description>
  <launchable type="desktop-id">rust-redis-desktop.desktop</launchable>
  <url type="homepage">https://github.com/yelog/rust-redis-desktop</url>
  <provides>
    <binary>rust-redis-desktop</binary>
  </provides>
</component>
EOF

cat > "$APP_DIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/rust-redis-desktop" "$@"
EOF
chmod +x "$APP_DIR/AppRun"

if command -v appimagetool &> /dev/null; then
    ARCH=x86_64 appimagetool "$APP_DIR" "rust-redis-desktop-${ARCH}.AppImage"
    echo "Created rust-redis-desktop-${ARCH}.AppImage"
else
    echo "AppImage created at $APP_DIR"
    echo "To build AppImage, install appimagetool and run: ARCH=x86_64 appimagetool $APP_DIR"
fi

mkdir -p debian/DEBIAN
mkdir -p debian/usr/bin
mkdir -p debian/usr/share/applications
mkdir -p debian/usr/share/doc/rust-redis-desktop

cp target/release/rust-redis-desktop debian/usr/bin/
chmod 755 debian/usr/bin/rust-redis-desktop

cat > debian/DEBIAN/control << EOF
Package: rust-redis-desktop
Version: $VERSION
Section: database
Priority: optional
Architecture: amd64
Maintainer: yelog <yelogeek@gmail.com>
Description: A Redis desktop manager written in Rust
 A modern Redis desktop manager built with Rust, Dioxus, and Freya.
 Features include:
 - Database Management
 - Data Visualization
 - Key Inspection and Editing
 - Command Execution
 - Cross-Platform Support
Homepage: https://github.com/yelog/rust-redis-desktop
EOF

cp "$APP_DIR/usr/share/applications/rust-redis-desktop.desktop" debian/usr/share/applications/

dpkg-deb --build debian "rust-redis-desktop_${VERSION}_amd64.deb"
echo "Created rust-redis-desktop_${VERSION}_amd64.deb"

rm -rf debian

echo "Linux packages created successfully"