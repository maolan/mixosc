#!/usr/bin/env bash
set -euo pipefail

# build-fedora.sh — Build a .rpm package for MixOSC on Fedora.
#
# Usage:
#   ./scripts/build-fedora.sh [OPTIONS]
#
# Options:
#   -s, --source-dir DIR     Path to mixosc source directory (default: parent of this script)
#   -o, --output-dir DIR     Where to write the .rpm file (default: ./dist)
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

RPM_ARCH="$(uname -m)"
PKG_NAME="mixosc"
RPM_NAME="${PKG_NAME}-${PKG_VERSION}-1.fedora.${RPM_ARCH}.rpm"

echo "========================================"
echo "Building MixOSC .rpm package"
echo "Version: $PKG_VERSION"
echo "Architecture: $RPM_ARCH"
echo "Source: $SOURCE_DIR"
echo "Output: $OUTPUT_DIR/$RPM_NAME"
echo "========================================"

# ---------------------------------------------------------------------------
# 1. Install system build dependencies
# ---------------------------------------------------------------------------
echo ""
echo "[1/5] Installing build dependencies..."
sudo dnf install -y \
    pkgconf-pkg-config \
    gcc \
    gcc-c++ \
    libxkbcommon-devel \
    git \
    rpm-build \
    curl \
    ca-certificates

# ---------------------------------------------------------------------------
# 2. Ensure Rust is installed
# ---------------------------------------------------------------------------
echo ""
echo "[2/5] Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    echo "Rust not found. Installing from distribution packages..."
    sudo dnf install -y rust cargo
else
    echo "Rust already installed: $(rustc --version)"
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
# 4. Prepare RPM package staging area
# ---------------------------------------------------------------------------
echo ""
echo "[4/5] Preparing RPM package structure..."

SPEC_DIR="$(mktemp -d)"
trap "rm -rf '$SPEC_DIR'" EXIT

mkdir -p "$SPEC_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

STAGING_DIR="$SPEC_DIR/staging"
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

# Create tarball for rpmbuild
cd "$STAGING_DIR"
tar czf "$SPEC_DIR/SOURCES/mixosc-files.tar.gz" .

# Generate spec file
cat > "$SPEC_DIR/SPECS/mixosc.spec" <<EOF
Name:           $PKG_NAME
Version:        $PKG_VERSION
Release:        1.fedora
Summary:        OSC mixer control surface for X32 and X-Air
License:        BSD-2-Clause
URL:            https://github.com/maolan/mixosc
Source0:        mixosc-files.tar.gz
BuildArch:      $RPM_ARCH

Requires:       libxkbcommon

%description
MixOSC is a Rust GUI application that discovers and controls
Behringer X32 and X-Air digital mixers over OSC.

%prep
# No source preparation needed for binary build

%build
# No build needed — binary is already built

%install
mkdir -p %{buildroot}
cd %{buildroot}
tar xzf %{SOURCE0}

%files
%defattr(-,root,root,-)
/usr/bin/mixosc
/usr/share/applications/mixosc.desktop
/usr/share/icons/hicolor/512x512/apps/mixosc.png
%doc /usr/share/doc/mixosc/README.md
%license /usr/share/doc/mixosc/LICENSE

%changelog
* Sun May 10 2026 Maolan Team <maolan@github.io> - $PKG_VERSION-1
- Initial RPM package.
EOF

# ---------------------------------------------------------------------------
# 5. Build the .rpm package
# ---------------------------------------------------------------------------
echo ""
echo "[5/5] Building .rpm package..."
cd "$SPEC_DIR"
rpmbuild --define "_topdir $SPEC_DIR" --bb "$SPEC_DIR/SPECS/mixosc.spec"

# Copy result to output directory
mkdir -p "$OUTPUT_DIR"

# rpmbuild expands Release, so find the actual file name
BUILT_RPM="$(ls "$SPEC_DIR/RPMS/$RPM_ARCH/"*.rpm | head -n1)"
cp "$BUILT_RPM" "$OUTPUT_DIR/"

BUILT_RPM_BASENAME="$(basename "$BUILT_RPM")"

echo ""
echo "========================================"
echo "Package built successfully:"
echo "  $OUTPUT_DIR/$BUILT_RPM_BASENAME"
echo "========================================"
