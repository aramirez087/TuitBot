#!/usr/bin/env bash
set -euo pipefail

REPO="${TUITBOT_REPO:-aramirez087/TuitBot}"
INSTALL_DIR="${TUITBOT_INSTALL_DIR:-}"
INSTALL_TAG="${TUITBOT_INSTALL_TAG:-}"
RELEASES_API_URL="https://api.github.com/repos/${REPO}/releases?per_page=50"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
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

resolve_download_url() {
  local tag candidate tags

  if [ -n "$INSTALL_TAG" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${INSTALL_TAG}/${ASSET}"
    RELEASE_TAG="$INSTALL_TAG"
    return 0
  fi

  tags="$(curl -fsSL "$RELEASES_API_URL" \
    | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p')"

  if [ -z "$tags" ]; then
    echo "Could not read release tags from GitHub API."
    exit 1
  fi

  while IFS= read -r tag; do
    case "$tag" in
      tuitbot-cli-v*|v*) ;;
      *) continue ;;
    esac

    candidate="https://github.com/${REPO}/releases/download/${tag}/${ASSET}"
    if curl -fsSLI "$candidate" >/dev/null; then
      DOWNLOAD_URL="$candidate"
      RELEASE_TAG="$tag"
      return 0
    fi
  done <<< "$tags"

  echo "No compatible release asset found for: ${ASSET}"
  echo "Manual downloads: https://github.com/${REPO}/releases"
  exit 1
}

resolve_download_url
echo "Installing ${ASSET} from ${RELEASE_TAG}"

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
