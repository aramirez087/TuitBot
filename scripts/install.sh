#!/usr/bin/env bash
# install.sh — TuitBot one-liner installer
#
# WHY THIS FILE EXISTS:
#   Users shouldn't need Rust/cargo to install a pre-built binary. This script
#   detects OS/arch, resolves the latest GitHub Release asset, verifies the
#   SHA256 checksum (from the release's SHA256SUMS file), and installs the
#   binary to ~/.local/bin or /usr/local/bin.
#
# USAGE:
#   curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | bash
#
# ENVIRONMENT OVERRIDES:
#   TUITBOT_REPO        — GitHub owner/repo (default: aramirez087/TuitBot)
#   TUITBOT_INSTALL_DIR — installation directory (default: auto-detected)
#   TUITBOT_INSTALL_TAG — pin a specific release tag (default: latest)
#
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
      x86_64)          TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64)   TARGET="aarch64-unknown-linux-gnu" ;;
      *)
        echo "Unsupported Linux architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)          TARGET="x86_64-apple-darwin" ;;
      aarch64|arm64)   TARGET="aarch64-apple-darwin" ;;
      *)
        echo "Unsupported macOS architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported operating system: $OS" >&2
    echo "Windows users: download from https://github.com/${REPO}/releases" >&2
    exit 1
    ;;
esac

ASSET="tuitbot-${TARGET}.tar.gz"

# Resolve the download URL and tag.
# Walks releases in order and picks the first one that has our target asset.
# WHY: release-plz may create a tag before assets are uploaded (rare race).
resolve_download_url() {
  local tag candidate tags

  if [ -n "$INSTALL_TAG" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${INSTALL_TAG}/${ASSET}"
    CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${INSTALL_TAG}/SHA256SUMS"
    RELEASE_TAG="$INSTALL_TAG"
    return 0
  fi

  tags="$(curl -fsSL "$RELEASES_API_URL" \
    | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p')"

  if [ -z "$tags" ]; then
    echo "Could not read release tags from GitHub API." >&2
    exit 1
  fi

  while IFS= read -r tag; do
    case "$tag" in
      tuitbot-cli-v*|v*) ;;
      *) continue ;;
    esac

    candidate="https://github.com/${REPO}/releases/download/${tag}/${ASSET}"
    if curl -fsSLI "$candidate" >/dev/null 2>&1; then
      DOWNLOAD_URL="$candidate"
      CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${tag}/SHA256SUMS"
      RELEASE_TAG="$tag"
      return 0
    fi
  done <<< "$tags"

  echo "No compatible release asset found for: ${ASSET}" >&2
  echo "Manual downloads: https://github.com/${REPO}/releases" >&2
  exit 1
}

# Verify SHA256 checksum of the downloaded archive.
# Downloads the release's SHA256SUMS file, extracts the expected hash for our
# asset, and compares it against the locally computed hash.
# WHY: ensures the downloaded binary hasn't been corrupted or tampered with.
verify_checksum() {
  local archive_path="$1"
  local tmp_dir="$2"

  local checksums_file="$tmp_dir/SHA256SUMS"
  if ! curl -fsSL "$CHECKSUMS_URL" -o "$checksums_file"; then
    echo "Warning: could not download SHA256SUMS — skipping checksum verification." >&2
    return 0
  fi

  local expected
  expected="$(grep " ${ASSET}$" "$checksums_file" | awk '{print $1}')"

  if [ -z "$expected" ]; then
    echo "Warning: ${ASSET} not found in SHA256SUMS — skipping checksum verification." >&2
    return 0
  fi

  local actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$archive_path" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$archive_path" | awk '{print $1}')"
  else
    echo "Warning: neither sha256sum nor shasum found — skipping checksum verification." >&2
    return 0
  fi

  if [ "$actual" != "$expected" ]; then
    echo "Checksum mismatch for ${ASSET}!" >&2
    echo "  Expected: $expected" >&2
    echo "  Got:      $actual" >&2
    echo "Aborting install — please re-run or file an issue." >&2
    exit 1
  fi

  echo "Checksum OK: $actual"
}

resolve_download_url
echo "Installing tuitbot ${RELEASE_TAG} (${TARGET})..."

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
echo "Downloading ${DOWNLOAD_URL}..."
curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"

verify_checksum "$ARCHIVE_PATH" "$TMP_DIR"

tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$TMP_DIR/tuitbot" "$INSTALL_DIR/tuitbot"

echo ""
echo "✓ Installed tuitbot to $INSTALL_DIR/tuitbot"
echo "  Run 'tuitbot --version' to confirm."
echo ""

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "Add $INSTALL_DIR to your PATH:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
    echo "Tip: add that line to ~/.bashrc, ~/.zshrc, or ~/.profile to make it permanent."
    ;;
esac
