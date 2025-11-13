#!/bin/bash
# Install or update DuckDB on Fedora
# Usage: ./scripts/install-duckdb-fedora.sh [version]
# Default version: latest stable

set -e

VERSION=${1:-"v1.1.3"}  # Default to latest stable
INSTALL_DIR="/usr/local"

echo "Installing DuckDB ${VERSION} to ${INSTALL_DIR}"

# Create temp directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Downloading DuckDB ${VERSION}..."
wget -q "https://github.com/duckdb/duckdb/releases/download/${VERSION}/libduckdb-linux-amd64.zip"

echo "Extracting..."
unzip -q libduckdb-linux-amd64.zip

echo "Installing library and headers..."
sudo install -m 644 libduckdb.so "${INSTALL_DIR}/lib/"
sudo install -m 644 duckdb.h duckdb.hpp "${INSTALL_DIR}/include/"

echo "Updating library cache..."
sudo ldconfig

# Verify installation
if [ -f "${INSTALL_DIR}/lib/libduckdb.so" ] && [ -f "${INSTALL_DIR}/include/duckdb.h" ]; then
    echo "✓ DuckDB ${VERSION} installed successfully!"
    echo "  Library: ${INSTALL_DIR}/lib/libduckdb.so"
    echo "  Headers: ${INSTALL_DIR}/include/duckdb.h"

    # Optionally install CLI
    read -p "Install DuckDB CLI? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        wget -q "https://github.com/duckdb/duckdb/releases/download/${VERSION}/duckdb_cli-linux-amd64.zip"
        unzip -q duckdb_cli-linux-amd64.zip
        sudo install -m 755 duckdb "${INSTALL_DIR}/bin/"
        echo "✓ DuckDB CLI installed: ${INSTALL_DIR}/bin/duckdb"
    fi
else
    echo "✗ Installation failed!"
    exit 1
fi

# Cleanup
cd - > /dev/null
rm -rf "$TMP_DIR"

echo ""
echo "To update DuckDB in the future, run:"
echo "  ./scripts/install-duckdb-fedora.sh v1.2.0"
echo ""
echo "Check latest releases at:"
echo "  https://github.com/duckdb/duckdb/releases"
