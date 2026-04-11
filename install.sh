#!/bin/bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
RESET='\033[0m'

REPO="MathisCrr/skl"
BIN_DIR="/usr/local/bin"
BIN_NAME="skl"

# Fetch latest release tag
VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$VERSION" ]; then
  echo -e "${RED}error: could not fetch latest version${RESET}" >&2
  exit 1
fi

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS/$ARCH" in
  linux/x86_64)  ARTIFACT="skl-linux" ;;
  darwin/x86_64) ARTIFACT="skl-macos" ;;
  darwin/arm64)  ARTIFACT="skl-macos" ;;
  *)
    echo -e "${RED}error: unsupported platform $OS/$ARCH${RESET}" >&2
    echo "install via cargo: cargo install skl" >&2
    exit 1
    ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/$ARTIFACT"

echo "installing skl $VERSION..."
curl -fsSL "$URL" -o "$BIN_NAME"
chmod +x "$BIN_NAME"

if [ -w "$BIN_DIR" ]; then
  mv "$BIN_NAME" "$BIN_DIR/$BIN_NAME"
else
  sudo mv "$BIN_NAME" "$BIN_DIR/$BIN_NAME"
fi

echo -e "${GREEN}skl $VERSION installed to $BIN_DIR/$BIN_NAME${RESET}"
