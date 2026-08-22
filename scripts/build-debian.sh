#!/usr/bin/env bash
set -euo pipefail

# build-debian.sh — Build a .deb package and an AppImage for MixOSC on Debian.
#
# Usage:
#   ./scripts/build-debian.sh [OPTIONS]
#
# Options:
#   -s, --source-dir DIR     Path to mixosc source directory (default: parent of this script)
#   -o, --output-dir DIR     Where to write the .deb and .AppImage files (default: ./dist)
#   -v, --version VERSION    Override package version (default: read from Cargo.toml)
#   -t, --target-dir DIR     Local target directory (useful when source is on NFS)
#   -h, --help               Show this help message
#
# The script installs build dependencies via apt, installs Rust via rustup if missing,
# builds the release binary, produces a .deb package using dpkg-deb, and builds an
# AppImage using linuxdeploy.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$SOURCE_DIR/dist"
OVERRIDE_VERSION=""
TARGET_DIR=""

usage() {
    sed -n '4,18p' "$0" | sed 's/^# //'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -s|--source-dir)
            SOURCE_DIR="$(realpath "$2")"
            shift 2
            ;;
        -o|--output-dir)
            OUTPUT_DIR="$(realpath "$2")"
            shift 2
            ;;
        -v|--version)
            OVERRIDE_VERSION="$2"
            shift 2
            ;;
        -t|--target-dir)
            TARGET_DIR="$(realpath "$2")"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

CARGO_TOML="$SOURCE_DIR/Cargo.toml"
if [[ ! -f "$CARGO_TOML" ]]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML" >&2
    exit 1
fi

# Extract version from Cargo.toml or use override
if [[ -n "$OVERRIDE_VERSION" ]]; then
    PKG_VERSION="$OVERRIDE_VERSION"
else
    PKG_VERSION="$(grep -m1 '^version' "$CARGO_TOML" | sed 's/.*= *"\(.*\)".*/\1/')"
fi

DEB_ARCH="$(dpkg --print-architecture)"
PKG_NAME="mixosc"
DEB_NAME="${PKG_NAME}-${PKG_VERSION}-debian.${DEB_ARCH}.deb"
APPIMAGE_NAME="${PKG_NAME}-${PKG_VERSION}-x86_64.AppImage"

echo "========================================"
echo "Building MixOSC .deb package and AppImage"
echo "Version: $PKG_VERSION"
echo "Architecture: $DEB_ARCH"
echo "Source: $SOURCE_DIR"
echo "Deb output: $OUTPUT_DIR/$DEB_NAME"
echo "AppImage output: $OUTPUT_DIR/$APPIMAGE_NAME"
echo "========================================"

# ---------------------------------------------------------------------------
# 1. Install system build dependencies
# ---------------------------------------------------------------------------
echo ""
echo "[1/7] Installing build dependencies..."
sudo apt-get update
sudo apt-get install -y \
    pkg-config \
    build-essential \
    libxkbcommon-dev \
    libfuse2t64 \
    curl \
    ca-certificates \
    git

# ---------------------------------------------------------------------------
# 2. Install Rust if missing
# ---------------------------------------------------------------------------
echo ""
echo "[2/7] Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    echo "Rust not found. Installing via rustup..."
    export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
    export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$CARGO_HOME/env"
else
    echo "Rust already installed: $(rustc --version)"
fi

# Ensure cargo is in PATH for the rest of the script
if [[ -f "${CARGO_HOME:-$HOME/.cargo}/env" ]]; then
    source "${CARGO_HOME:-$HOME/.cargo}/env"
fi

# ---------------------------------------------------------------------------
# 3. Build release binary
# ---------------------------------------------------------------------------
echo ""
echo "[3/7] Building release binary..."
cd "$SOURCE_DIR"

CARGO_ARGS=("--release")
if [[ -n "$TARGET_DIR" ]]; then
    mkdir -p "$TARGET_DIR"
    CARGO_ARGS+=("--target-dir" "$TARGET_DIR")
    echo "Using local target directory: $TARGET_DIR"
fi

cargo build "${CARGO_ARGS[@]}"

# Determine where binary ended up
if [[ -n "$TARGET_DIR" ]]; then
    BIN_DIR="$TARGET_DIR/release"
else
    BIN_DIR="$SOURCE_DIR/target/release"
fi

# Verify binary exists
if [[ ! -f "$BIN_DIR/mixosc" ]]; then
    echo "Error: Binary '$BIN_DIR/mixosc' not found after build" >&2
    exit 1
fi

echo "Build completed successfully."

# ---------------------------------------------------------------------------
# 4. Prepare Debian package staging area
# ---------------------------------------------------------------------------
echo ""
echo "[4/7] Preparing Debian package structure..."

STAGING_DIR="$(mktemp -d)"
APPDIR_BASE="$(mktemp -d)"
trap "rm -rf '$STAGING_DIR' '$APPDIR_BASE'" EXIT

mkdir -p "$STAGING_DIR/DEBIAN"
mkdir -p "$STAGING_DIR/usr/bin"
mkdir -p "$STAGING_DIR/usr/share/applications"
mkdir -p "$STAGING_DIR/usr/share/icons/hicolor/512x512/apps"
mkdir -p "$STAGING_DIR/usr/share/doc/$PKG_NAME"

# Binary
cp "$BIN_DIR/mixosc" "$STAGING_DIR/usr/bin/"
strip "$STAGING_DIR/usr/bin/mixosc"
chmod 755 "$STAGING_DIR/usr/bin/mixosc"

# Desktop entry
cp "$SOURCE_DIR/assets/desktop/mixosc-linux.desktop" "$STAGING_DIR/usr/share/applications/mixosc.desktop"
chmod 644 "$STAGING_DIR/usr/share/applications/mixosc.desktop"

# Icon
cp "$SOURCE_DIR/assets/images/mixosc.png" "$STAGING_DIR/usr/share/icons/hicolor/512x512/apps/mixosc.png"
chmod 644 "$STAGING_DIR/usr/share/icons/hicolor/512x512/apps/mixosc.png"

# Documentation
cp "$SOURCE_DIR/README.md" "$STAGING_DIR/usr/share/doc/$PKG_NAME/"
cp "$SOURCE_DIR/LICENSE"   "$STAGING_DIR/usr/share/doc/$PKG_NAME/"
gzip -9 -n -c > "$STAGING_DIR/usr/share/doc/$PKG_NAME/changelog.gz" /dev/null 2>/dev/null || true

# DEBIAN/control
cat > "$STAGING_DIR/DEBIAN/control" <<EOF
Package: $PKG_NAME
Version: $PKG_VERSION
Section: sound
Priority: optional
Architecture: $DEB_ARCH
Depends: libxkbcommon0
Maintainer: Maolan Team <maolan@github.io>
Description: OSC mixer control surface for X32 and X-Air
 MixOSC is a Rust GUI application that discovers and controls
 Behringer X32 and X-Air digital mixers over OSC.
EOF

cat > "$STAGING_DIR/DEBIAN/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: MixOSC
Source: https://github.com/maolan/mixosc

Files: *
Copyright: Maolan Team
License: BSD-2-Clause
EOF

# ---------------------------------------------------------------------------
# 5. Build the .deb package
# ---------------------------------------------------------------------------
echo ""
echo "[5/7] Building .deb package..."
mkdir -p "$OUTPUT_DIR"
fakeroot dpkg-deb --build "$STAGING_DIR" "$OUTPUT_DIR/$DEB_NAME"

# ---------------------------------------------------------------------------
# 6. Build the AppImage
# ---------------------------------------------------------------------------
echo ""
echo "[6/7] Building AppImage..."

APPDIR="$APPDIR_BASE/AppDir"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/512x512/apps"

cp "$BIN_DIR/mixosc" "$APPDIR/usr/bin/"

# AppImage desktop entry uses relative Exec/Icon paths
sed 's|^Exec=/usr/bin/mixosc|Exec=mixosc|; s|^Icon=/usr/share/icons/hicolor/512x512/apps/mixosc.png|Icon=mixosc|' \
    "$SOURCE_DIR/assets/desktop/mixosc-linux.desktop" > "$APPDIR/usr/share/applications/mixosc.desktop"

cp "$SOURCE_DIR/assets/images/mixosc.png" "$APPDIR/usr/share/icons/hicolor/512x512/apps/mixosc.png"

# Keep the linuxdeploy helper outside the deliverables directory.
LINUXDEPLOY_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/maolan"
LINUXDEPLOY="$LINUXDEPLOY_CACHE/linuxdeploy-x86_64.AppImage"
mkdir -p "$LINUXDEPLOY_CACHE"
if [[ ! -f "$LINUXDEPLOY" ]]; then
    echo "Downloading linuxdeploy..."
    curl -L -o "$LINUXDEPLOY" "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
    chmod +x "$LINUXDEPLOY"
fi

cd "$APPDIR_BASE"
"$LINUXDEPLOY" --appimage-extract-and-run \
    --appdir "$APPDIR" \
    --desktop-file "$APPDIR/usr/share/applications/mixosc.desktop" \
    --icon-file "$APPDIR/usr/share/icons/hicolor/512x512/apps/mixosc.png" \
    --executable "$APPDIR/usr/bin/mixosc" \
    --output appimage

# linuxdeploy/appimagetool names the file from the .desktop Name field, so
# pick up whatever single AppImage was produced rather than hard-coding the
# basename.
BUILT_APPIMAGE=("$APPDIR_BASE"/*.AppImage)
if [[ ! -f "${BUILT_APPIMAGE[0]}" ]]; then
    echo "Error: No AppImage was produced in $APPDIR_BASE" >&2
    exit 1
fi
mv "${BUILT_APPIMAGE[0]}" "$OUTPUT_DIR/$APPIMAGE_NAME"
chmod +x "$OUTPUT_DIR/$APPIMAGE_NAME"

# ---------------------------------------------------------------------------
# 7. Verify the outputs
# ---------------------------------------------------------------------------
echo ""
echo "[7/7] Verifying outputs..."
dpkg-deb --info "$OUTPUT_DIR/$DEB_NAME"
dpkg-deb --contents "$OUTPUT_DIR/$DEB_NAME"
ls -lh "$OUTPUT_DIR/$APPIMAGE_NAME"

echo ""
echo "========================================"
echo "Packages built successfully:"
echo "  $OUTPUT_DIR/$DEB_NAME"
echo "  $OUTPUT_DIR/$APPIMAGE_NAME"
echo "========================================"
