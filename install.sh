#!/bin/bash
set -e

REPO="DreamCats/logid"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="logid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)  os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *)      error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac

    echo "${arch}-${os}"
}

# Get latest version from GitHub (using redirect to avoid API rate limits)
get_latest_version() {
    # Method 1: Use GitHub redirect (no API rate limit)
    local redirect_url
    redirect_url=$(curl -sI "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -i "^location:" | sed 's/.*tag\/\([^[:space:]]*\).*/\1/' | tr -d '\r')

    if [ -n "$redirect_url" ]; then
        echo "$redirect_url"
        return
    fi

    # Method 2: Fallback to API
    local response
    response=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null)

    if echo "$response" | grep -q "tag_name"; then
        echo "$response" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
        return
    fi

    # Method 3: Try tags API
    response=$(curl -sL "https://api.github.com/repos/${REPO}/tags" 2>/dev/null)
    if echo "$response" | grep -q '"name":'; then
        echo "$response" | grep '"name":' | head -1 | sed -E 's/.*"([^"]+)".*/\1/'
        return
    fi
}

# Download and install
install() {
    local platform version download_url tmp_dir

    platform=$(detect_platform)
    info "Detected platform: $platform"

    version=$(get_latest_version)
    if [ -z "$version" ]; then
        error "Failed to get latest version. Please check your network connection."
    fi
    info "Latest version: $version"

    download_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}.tar.gz"
    info "Downloading from: $download_url"

    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download and extract
    if ! curl -fsSL "$download_url" | tar xz -C "$tmp_dir" 2>/dev/null; then
        error "Failed to download or extract. URL: $download_url"
    fi

    # Create install directory if not exists
    mkdir -p "$INSTALL_DIR"

    # Install binary
    if [ -f "$tmp_dir/$BINARY_NAME" ]; then
        mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/"
    else
        # Try to find binary in subdirectory
        local found_binary
        found_binary=$(find "$tmp_dir" -name "$BINARY_NAME" -type f 2>/dev/null | head -1)
        if [ -n "$found_binary" ]; then
            mv "$found_binary" "$INSTALL_DIR/"
        else
            error "Binary not found in archive"
        fi
    fi

    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    info "Installed to: $INSTALL_DIR/$BINARY_NAME"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    info "Installation complete! Run 'logid --version' to verify."
}

install
