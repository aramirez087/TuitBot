#!/usr/bin/env bash
set -euo pipefail

REPO="${TUITBOT_REPO:-aramirez087/TuitBot}"
INSTALL_DIR="${TUITBOT_INSTALL_DIR:-}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64)
        echo "Linux ARM64 builds are not published yet."
        echo "Install from source with cargo for now."
        exit 1
        ;;
      *)
        echo "Unsupported Linux architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      aarch64|arm64) TARGET="aarch64-apple-darwin" ;;
      *)
        echo "Unsupported macOS architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported operating system: $OS"
    exit 1
    ;;
esac

ASSET="tuitbot-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

if [ -z "$INSTALL_DIR" ]; then
  if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
  fi
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

ARCHIVE_PATH="$TMP_DIR/$ASSET"
curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$TMP_DIR/tuitbot" "$INSTALL_DIR/tuitbot"

echo "Installed tuitbot to $INSTALL_DIR/tuitbot"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo
    echo "Add this to your shell profile so 'tuitbot' is on PATH:"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac
