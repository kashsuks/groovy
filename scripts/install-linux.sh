#!/usr/bin/env bash
# Installs groovy on Linux: checks for the ALSA runtime lib rodio needs
# and installs it via the system package manager if it's missing, then
# copies the groovy binary onto PATH.
set -euo pipefail

BIN_NAME="groovy"
INSTALL_DIR="${GROOVY_INSTALL_DIR:-$HOME/.local/bin}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

have() { command -v "$1" >/dev/null 2>&1; }

check_alsa() {
    if have ldconfig && ldconfig -p 2>/dev/null | grep -q 'libasound\.so'; then
        return 0
    fi
    if have pkg-config && pkg-config --exists alsa; then
        return 0
    fi
    return 1
}

install_alsa() {
    echo "Required audio library (libasound2) not found. Attempting to install it..."

    if have apt-get; then
        sudo apt-get update
        sudo apt-get install -y libasound2
    elif have dnf; then
        sudo dnf install -y alsa-lib
    elif have yum; then
        sudo yum install -y alsa-lib
    elif have pacman; then
        sudo pacman -Sy --noconfirm alsa-lib
    elif have zypper; then
        sudo zypper install -y alsa
    elif have apk; then
        sudo apk add --no-cache alsa-lib
    else
        echo "Could not detect a supported package manager." >&2
        echo "Please install the ALSA runtime library (e.g. libasound2 / alsa-lib) manually, then re-run this script." >&2
        exit 1
    fi
}

if check_alsa; then
    echo "libasound2 already installed."
else
    install_alsa
    if ! check_alsa; then
        echo "libasound2 installation could not be verified; groovy may fail to run." >&2
    fi
fi

mkdir -p "$INSTALL_DIR"
cp "$SCRIPT_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Installed groovy to $INSTALL_DIR/$BIN_NAME"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "Note: $INSTALL_DIR is not on your PATH. Add it with:" ;
       echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
