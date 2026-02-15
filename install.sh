#!/usr/bin/env bash
set -e

# Cyclang installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/lyledean1/cyclang/main/install.sh | bash

REPO="lyledean1/cyclang"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
ROOT_DIR="${ROOT_DIR:-$HOME/.cyclang}"

main() {
    echo "Installing Cyclang..."
    echo

    OS=$(get_os)
    ARCH=$(get_arch)

    if [ -z "$OS" ] || [ -z "$ARCH" ]; then
        echo "Error: Unsupported platform"
        echo "OS: $(uname -s)"
        echo "Architecture: $(uname -m)"
        exit 1
    fi

    PLATFORM="${OS}-${ARCH}"
    echo "Detected platform: $PLATFORM"

    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/cyclang-${PLATFORM}.tar.gz"
    echo "Downloading from: $DOWNLOAD_URL"

    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$TMP_DIR"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO- "$DOWNLOAD_URL" | tar -xz -C "$TMP_DIR"
    else
        echo "Error: Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    mkdir -p "$ROOT_DIR/bin" "$ROOT_DIR/lib"
    mv "$TMP_DIR/bin/cyclang.real" "$ROOT_DIR/bin/cyclang.real"
    if [ -d "$TMP_DIR/lib" ]; then
        rm -rf "$ROOT_DIR/lib"
        mv "$TMP_DIR/lib" "$ROOT_DIR/lib"
    fi

    chmod +x "$ROOT_DIR/bin/cyclang.real"

    install_wrapper

    echo
    echo "✓ Cyclang installed successfully"
    echo "  Binary: $ROOT_DIR/bin/cyclang.real"
    echo "  Wrapper: $INSTALL_DIR/cyclang"
    echo
    echo "Run 'cyclang --help' to get started"
}

install_wrapper() {
    WRAPPER_CONTENT="#!/usr/bin/env bash
set -e
ROOT_DIR=\"${ROOT_DIR}\"
BIN=\"${ROOT_DIR}/bin/cyclang.real\"
LIB=\"${ROOT_DIR}/lib\"
case \"\$(uname -s)\" in
  Darwin*)
    export DYLD_LIBRARY_PATH=\"${LIB}\${DYLD_LIBRARY_PATH:+:\$DYLD_LIBRARY_PATH}\"
    ;;
  *)
    export LD_LIBRARY_PATH=\"${LIB}\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}\"
    ;;
esac
exec \"\$BIN\" \"\$@\"
"

    if [ -w "$INSTALL_DIR" ]; then
        echo "$WRAPPER_CONTENT" > "$INSTALL_DIR/cyclang"
        chmod +x "$INSTALL_DIR/cyclang"
    else
        echo "Installing wrapper to $INSTALL_DIR (requires sudo)..."
        echo "$WRAPPER_CONTENT" | sudo tee "$INSTALL_DIR/cyclang" > /dev/null
        sudo chmod +x "$INSTALL_DIR/cyclang"
    fi
}

get_os() {
    case "$(uname -s)" in
        Darwin*)
            echo "macos"
            ;;
        Linux*)
            echo "linux"
            ;;
        *)
            echo ""
            ;;
    esac
}

get_arch() {
    case "$(uname -m)" in
        x86_64|amd64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            echo ""
            ;;
    esac
}

main "$@"
