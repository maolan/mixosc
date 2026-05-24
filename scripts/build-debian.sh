#!/usr/bin/env bash
set -euo pipefail

# build-debian.sh — Build a .deb package for MixOSC on Ubuntu.
#
# Usage:
#   ./scripts/build-debian.sh [OPTIONS]
#
# Options:
#   -s, --source-dir DIR     Path to mixosc source directory (default: parent of this script)
#   -o, --output-dir DIR     Where to write the .deb file (default: ./dist)
#   -v, --version VERSION    Override package version (default: read from Cargo.toml)
#   -t, --target-dir DIR     Local target directory (useful when source is on NFS)
#   -h, --help               Show this help message

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$SOURCE_DIR/dist"
OVERRIDE_VERSION=""
TARGET_DIR=""

usage() {
    sed -n '2,14p' "$0" | sed 's/^# //'
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
DEB_NAME="${PKG_NAME}_${PKG_VERSION}-debian_${DEB_ARCH}.deb"

echo "========================================"
echo "Building MixOSC .deb package"
echo "Version: $PKG_VERSION"
echo "Architecture: $DEB_ARCH"
echo "Source: $SOURCE_DIR"
echo "Output: $OUTPUT_DIR/$DEB_NAME"
echo "========================================"

# ---------------------------------------------------------------------------
# 1. Install system build dependencies
# ---------------------------------------------------------------------------
echo ""
echo "[1/5] Installing build dependencies..."
sudo apt-get update
sudo apt-get install -y \
    pkg-config \
    build-essential \
    libxkbcommon-dev \
    curl \
    ca-certificates \
    git

# ---------------------------------------------------------------------------
# 2. Install Rust if missing
# ---------------------------------------------------------------------------
echo ""
echo "[2/5] Checking Rust toolchain..."
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
echo "[3/5] Building release binary..."
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
echo "[4/5] Preparing Debian package structure..."

STAGING_DIR="$(mktemp -d)"
trap "rm -rf '$STAGING_DIR'" EXIT

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
cp "$SOURCE_DIR/desktop/mixosc-linux.desktop" "$STAGING_DIR/usr/share/applications/mixosc.desktop"
chmod 644 "$STAGING_DIR/usr/share/applications/mixosc.desktop"

# Icon
cp "$SOURCE_DIR/images/mixosc.png" "$STAGING_DIR/usr/share/icons/hicolor/512x512/apps/mixosc.png"
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
echo "[5/5] Building .deb package..."
mkdir -p "$OUTPUT_DIR"
fakeroot dpkg-deb --build "$STAGING_DIR" "$OUTPUT_DIR/$DEB_NAME"

# Verify the package
dpkg-deb --info "$OUTPUT_DIR/$DEB_NAME"
dpkg-deb --contents "$OUTPUT_DIR/$DEB_NAME"

echo ""
echo "========================================"
echo "Package built successfully:"
echo "  $OUTPUT_DIR/$DEB_NAME"
echo "========================================"
