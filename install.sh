#!/bin/bash
set -e

REPO="francescorubbo/trackio-tui"
BINARY="trackio-tui"

# Default to user install
INSTALL_DIR="$HOME/.local/bin"
SUDO=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --system)
            INSTALL_DIR="/usr/local/bin"
            SUDO="sudo"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --system    Install to /usr/local/bin (requires sudo)"
            echo "  --help      Show this help message"
            echo ""
            echo "By default, installs to ~/.local/bin (no sudo required)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64) TARGET="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        echo "Please download manually from https://github.com/$REPO/releases"
        exit 1
        ;;
esac

# Create install directory if needed
if [[ "$SUDO" == "" ]]; then
    mkdir -p "$INSTALL_DIR"
else
    $SUDO mkdir -p "$INSTALL_DIR"
fi

# Download and install
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$BINARY-$TARGET.tar.gz"

echo "Downloading $BINARY for $TARGET..."
curl -sSL "$DOWNLOAD_URL" | tar xz

echo "Installing to $INSTALL_DIR..."
$SUDO mv "$BINARY" "$INSTALL_DIR/"
$SUDO chmod +x "$INSTALL_DIR/$BINARY"

echo "Successfully installed $BINARY to $INSTALL_DIR/$BINARY"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "NOTE: $INSTALL_DIR is not in your PATH."
    echo "Add it with: export PATH=\"$INSTALL_DIR:\$PATH\""
fi

