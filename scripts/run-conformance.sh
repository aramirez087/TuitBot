#!/usr/bin/env bash
# Run MCP conformance tests and generate coverage report.
#
# Usage:
#   bash scripts/run-conformance.sh              # deterministic tests only
#   bash scripts/run-conformance.sh --live        # include live sandbox tests
#   bash scripts/run-conformance.sh --report-only # just regenerate the report
#
# Environment variables for live tests:
#   TUITBOT_TEST_BEARER_TOKEN     - App-only Bearer token
#   TUITBOT_TEST_USER_ID          - Authenticated user's numeric ID
#   TUITBOT_TEST_KNOWN_TWEET_ID   - A tweet ID known to exist
#   TUITBOT_TEST_KNOWN_USERNAME   - A username known to exist

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$REPO_ROOT/docs/generated"
ARTIFACTS_DIR="$REPO_ROOT/roadmap/artifacts"

LIVE=false
REPORT_ONLY=false

for arg in "$@"; do
  case $arg in
    --live) LIVE=true ;;
    --report-only) REPORT_ONLY=true ;;
    *) echo "Unknown argument: $arg"; exit 1 ;;
  esac
done

mkdir -p "$REPORT_DIR" "$ARTIFACTS_DIR"

echo "=== MCP Conformance Harness ==="
echo ""

if [ "$REPORT_ONLY" = true ]; then
  echo "--- Generating coverage report only ---"
  cargo test -p tuitbot-mcp tools::conformance_tests::coverage::generate_coverage_report -- --exact 2>&1 | tail -5
  echo ""
  echo "Reports written to:"
  echo "  $REPORT_DIR/coverage-report.json"
  echo "  $REPORT_DIR/coverage-report.md"
  echo "  $ARTIFACTS_DIR/session-09-coverage-report.json"
  echo "  $ARTIFACTS_DIR/session-09-coverage-report.md"
  exit 0
fi

echo "--- Phase 1: Deterministic conformance tests (mock-based) ---"
echo ""

# Kernel conformance (27 tools)
echo "[1/5] Kernel conformance tests..."
cargo test -p tuitbot-mcp conformance_ -- --quiet 2>&1 | tail -3

# Contract envelope tests
echo "[2/5] Contract envelope tests..."
cargo test -p tuitbot-mcp contract_test -- --quiet 2>&1 | tail -3

# Golden fixture tests
echo "[3/5] Golden fixture tests..."
cargo test -p tuitbot-mcp golden_fixtures -- --quiet 2>&1 | tail -3

# Boundary/profile isolation tests
echo "[4/5] Boundary tests..."
cargo test -p tuitbot-mcp boundary_ -- --quiet 2>&1 | tail -3

# Eval scenarios D-G
echo "[5/5] Eval scenarios..."
cargo test -p tuitbot-mcp eval_session09 -- --quiet 2>&1 | tail -3

echo ""
echo "--- Phase 2: Coverage report generation ---"
echo ""
cargo test -p tuitbot-mcp tools::conformance_tests::coverage::generate_coverage_report -- --exact 2>&1 | tail -3

echo ""
echo "Reports written to:"
echo "  $REPORT_DIR/coverage-report.json"
echo "  $REPORT_DIR/coverage-report.md"

if [ "$LIVE" = true ]; then
  echo ""
  echo "--- Phase 3: Live conformance tests (sandbox credentials) ---"
  echo ""

  if [ -z "${TUITBOT_TEST_BEARER_TOKEN:-}" ]; then
    echo "WARNING: TUITBOT_TEST_BEARER_TOKEN not set. Live tests will skip."
  else
    echo "Bearer token detected. Running live tests..."
  fi

  cargo test -p tuitbot-mcp live -- --ignored 2>&1 | tail -20
  echo ""
  echo "Live report: $ARTIFACTS_DIR/session-09-live-conformance.md"
fi

echo ""
echo "=== Conformance harness complete ==="
