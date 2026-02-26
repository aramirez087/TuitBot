#!/usr/bin/env bash
# Generate MCP profile manifest JSON artifacts from source.
#
# Usage: bash scripts/generate-mcp-manifests.sh
#
# Produces docs/generated/mcp-manifest-{write,admin,readonly,api-readonly}.json
# Commit these files alongside any tool or profile changes.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$REPO_ROOT/docs/generated"
PROFILES=(write admin readonly api-readonly)

mkdir -p "$OUT_DIR"

echo "Building tuitbot-cli..."
cargo build -p tuitbot-cli --quiet

for profile in "${PROFILES[@]}"; do
  out="$OUT_DIR/mcp-manifest-$profile.json"
  echo "Generating $profile manifest -> $out"
  cargo run -p tuitbot-cli --quiet -- mcp manifest --profile "$profile" > "$out"
done

echo "Done. Generated ${#PROFILES[@]} manifests in $OUT_DIR"
