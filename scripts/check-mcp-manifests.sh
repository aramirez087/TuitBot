#!/usr/bin/env bash
# Verify committed MCP manifest artifacts match current source.
#
# Usage: bash scripts/check-mcp-manifests.sh
#
# Exit codes:
#   0 — all manifests in sync
#   1 — drift detected or orphaned files found
#
# Used in CI (manifest-sync job). No external dependencies beyond POSIX.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMMITTED_DIR="$REPO_ROOT/docs/generated"
PROFILES=(write admin readonly api-readonly utility-readonly utility-write)

TMPDIR_FRESH="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_FRESH"' EXIT

# ── Build and generate fresh manifests ──────────────────────────────

echo "Building tuitbot-cli..."
cargo build -p tuitbot-cli --quiet

for profile in "${PROFILES[@]}"; do
  cargo run -p tuitbot-cli --quiet -- mcp manifest --profile "$profile" \
    > "$TMPDIR_FRESH/mcp-manifest-$profile.json"
done

# ── Compare each profile ────────────────────────────────────────────

FAIL=0

for profile in "${PROFILES[@]}"; do
  committed="$COMMITTED_DIR/mcp-manifest-$profile.json"
  fresh="$TMPDIR_FRESH/mcp-manifest-$profile.json"

  if [ ! -f "$committed" ]; then
    echo "FAIL: $committed does not exist. Run: bash scripts/generate-mcp-manifests.sh"
    FAIL=1
    continue
  fi

  # Strip tuitbot_mcp_version before comparison — it changes on release bumps
  # and is not a drift signal.
  committed_stripped="$(grep -v '^\s*"tuitbot_mcp_version"' "$committed")"
  fresh_stripped="$(grep -v '^\s*"tuitbot_mcp_version"' "$fresh")"

  if ! diff_output="$(diff -u <(echo "$committed_stripped") <(echo "$fresh_stripped"))"; then
    echo "FAIL: $profile manifest has drifted from source."
    echo "$diff_output"
    echo ""
    echo "Fix: bash scripts/generate-mcp-manifests.sh"
    FAIL=1
  else
    tool_count="$(grep -c '"name":' "$fresh" || true)"
    echo "OK: $profile ($tool_count tools)"
  fi
done

# ── Check for orphaned files ────────────────────────────────────────

for f in "$COMMITTED_DIR"/mcp-manifest-*.json; do
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  # Extract profile name: mcp-manifest-<profile>.json
  profile_name="${base#mcp-manifest-}"
  profile_name="${profile_name%.json}"

  found=0
  for p in "${PROFILES[@]}"; do
    if [ "$p" = "$profile_name" ]; then
      found=1
      break
    fi
  done

  if [ "$found" -eq 0 ]; then
    echo "FAIL: orphaned manifest file $base (profile '$profile_name' no longer exists)"
    FAIL=1
  fi
done

# ── Result ──────────────────────────────────────────────────────────

if [ "$FAIL" -ne 0 ]; then
  exit 1
fi

echo "All manifests in sync."
