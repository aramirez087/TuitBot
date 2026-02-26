#!/usr/bin/env bash
# run-sessions.sh — Execute a sequence of Claude Code session files autonomously.
#
# Each session gets a FRESH claude -p process (clean context window).
# Claude plans first, then executes, then commits — no human in the loop.
#
# Usage:
#   run-sessions.sh <sessions-dir> [options]
#   run-sessions.sh docs/claude-sessions/x-api-surface-expansion
#   run-sessions.sh docs/claude-sessions/x-api-surface-expansion --start 5 --model opus
#
# The directory must contain:
#   - session-00-*.md  (operator rules — prepended to every session)
#   - session-01-*.md through session-NN-*.md (sequential work sessions)
#
# Each .md file should contain a prompt inside a ```md ... ``` code fence.
#
# Options:
#   --start N       Resume from session N (default: 1)
#   --end N         Stop after session N (default: run all)
#   --dry-run       Show what would run without executing
#   --branch NAME   Create/switch to this git branch before starting
#   --model MODEL   Claude model (default: sonnet). Options: opus, sonnet, haiku
#   --no-commit     Skip auto-commit after each session
#
# Requirements:
#   - `claude` CLI on PATH
#   - Inside a git repository
#   - Session files following the naming convention
#
# Security: Uses --dangerously-skip-permissions for autonomous execution.
# Review your session files before running.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

log()  { echo -e "${BLUE}[epic]${NC} $*"; }
ok()   { echo -e "${GREEN}[epic]${NC} $*"; }
warn() { echo -e "${YELLOW}[epic]${NC} $*"; }
err()  { echo -e "${RED}[epic]${NC} $*" >&2; }
dim()  { echo -e "${DIM}$*${NC}"; }

usage() {
  cat <<'USAGE'
Usage: run-sessions.sh <sessions-dir> [options]

Options:
  --start N       Resume from session N (default: 1)
  --end N         Stop after session N (default: all)
  --dry-run       Preview without executing
  --branch NAME   Git branch to work on
  --model MODEL   Claude model: opus, sonnet (default), haiku
  --no-commit     Skip auto-commit after each session
  -h, --help      Show this help
USAGE
  exit 1
}

# --- Parse arguments ---
SESSIONS_DIR=""
START_FROM=1
END_AT=999
DRY_RUN=false
BRANCH=""
MODEL="sonnet"
AUTO_COMMIT=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    --start)     START_FROM="$2"; shift 2 ;;
    --end)       END_AT="$2"; shift 2 ;;
    --dry-run)   DRY_RUN=true; shift ;;
    --branch)    BRANCH="$2"; shift 2 ;;
    --model)     MODEL="$2"; shift 2 ;;
    --no-commit) AUTO_COMMIT=false; shift ;;
    --help|-h)   usage ;;
    *)
      if [[ -z "$SESSIONS_DIR" ]]; then
        SESSIONS_DIR="$1"; shift
      else
        err "Unknown argument: $1"; usage
      fi
      ;;
  esac
done

if [[ -z "$SESSIONS_DIR" ]]; then
  err "Missing required argument: <sessions-dir>"
  echo ""
  usage
fi

if [[ ! -d "$SESSIONS_DIR" ]]; then
  err "Directory not found: $SESSIONS_DIR"
  exit 1
fi

SESSIONS_DIR="$(cd "$SESSIONS_DIR" && pwd)"

# --- Validate environment ---
if ! command -v claude &>/dev/null; then
  err "claude CLI not found on PATH."
  exit 1
fi

if ! git rev-parse --is-inside-work-tree &>/dev/null; then
  err "Not inside a git repository."
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"

# --- Extract prompt from markdown code fence ---
extract_prompt() {
  local file="$1"
  local content
  content="$(awk '
    /^```md$/ { capture=1; next }
    /^```$/   { if (capture) exit }
    capture   { print }
  ' "$file")"

  # Fallback: if no ```md fence, use the whole file (skip the # Title line)
  if [[ -z "$content" ]]; then
    content="$(tail -n +2 "$file")"
  fi
  echo "$content"
}

# --- Discover session files ---
OPERATOR_FILE=""
SESSION_FILES=()

for f in "$SESSIONS_DIR"/session-*.md; do
  [[ -f "$f" ]] || continue
  fname="$(basename "$f")"
  if [[ "$fname" == session-00-* ]]; then
    OPERATOR_FILE="$f"
  else
    SESSION_FILES+=("$f")
  fi
done

if [[ -z "$OPERATOR_FILE" ]]; then
  err "No session-00-*.md (operator rules) found in $SESSIONS_DIR"
  exit 1
fi

if [[ ${#SESSION_FILES[@]} -eq 0 ]]; then
  err "No session files (01+) found in $SESSIONS_DIR"
  exit 1
fi

# Sort naturally by number
IFS=$'\n' SESSION_FILES=($(printf '%s\n' "${SESSION_FILES[@]}" | sort -V)); unset IFS

TOTAL=${#SESSION_FILES[@]}
OPERATOR_PROMPT="$(extract_prompt "$OPERATOR_FILE")"

if [[ -z "$OPERATOR_PROMPT" ]]; then
  err "Could not extract prompt from $OPERATOR_FILE"
  exit 1
fi

# --- Print banner ---
echo ""
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "${BOLD}Epic Runner${NC}"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "Sessions dir : ${SESSIONS_DIR/#$HOME/~}"
log "Operator file: $(basename "$OPERATOR_FILE")"
LAST_IDX=$(( ${#SESSION_FILES[@]} - 1 ))
log "Sessions     : $TOTAL ($(basename "${SESSION_FILES[0]}" .md) → $(basename "${SESSION_FILES[$LAST_IDX]}" .md))"
log "Range        : $START_FROM → $END_AT"
log "Model        : $MODEL"
log "Auto-commit  : $AUTO_COMMIT"
if [[ -n "$BRANCH" ]]; then
  log "Branch       : $BRANCH"
fi
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# --- Branch setup ---
if [[ -n "$BRANCH" ]]; then
  CURRENT_BRANCH="$(git branch --show-current)"
  if [[ "$CURRENT_BRANCH" != "$BRANCH" ]]; then
    if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
      log "Switching to existing branch: $BRANCH"
      git checkout "$BRANCH"
    else
      log "Creating new branch: $BRANCH"
      git checkout -b "$BRANCH"
    fi
  fi
fi

# --- Collect previous handoff context ---
# If resuming (--start > 1), look for the previous session's handoff doc
# to feed as additional context for continuity.
find_previous_handoff() {
  local session_num="$1"
  local prev=$((session_num - 1))
  local padded
  padded=$(printf "%02d" "$prev")

  # Check common handoff locations
  local candidates=(
    "$REPO_ROOT/docs/roadmap/"*"/session-${padded}-handoff.md"
    "$REPO_ROOT/docs/roadmap/"*"/session-${prev}-handoff.md"
  )

  for candidate in "${candidates[@]}"; do
    if [[ -f "$candidate" ]]; then
      echo "$candidate"
      return
    fi
  done
}

# --- Execute sessions ---
FAILED=0
COMMITS=()

for session_file in "${SESSION_FILES[@]}"; do
  fname="$(basename "$session_file" .md)"

  # Extract session number
  session_num="$(echo "$fname" | grep -oE 'session-([0-9]+)' | grep -oE '[0-9]+' | sed 's/^0*//')"
  session_num="${session_num:-0}"

  if [[ "$session_num" -lt "$START_FROM" ]]; then
    dim "  skip  $fname (before --start $START_FROM)"
    continue
  fi

  if [[ "$session_num" -gt "$END_AT" ]]; then
    dim "  skip  $fname (after --end $END_AT)"
    continue
  fi

  SESSION_PROMPT="$(extract_prompt "$session_file")"
  if [[ -z "$SESSION_PROMPT" ]]; then
    warn "Skipping $fname — could not extract prompt"
    continue
  fi

  FRIENDLY_NAME="$(echo "$fname" | sed 's/^session-[0-9]*-//' | tr '-' ' ')"

  echo ""
  log "┌─────────────────────────────────────────────────────┐"
  log "│ ${BOLD}Session $session_num: $FRIENDLY_NAME${NC}"
  log "└─────────────────────────────────────────────────────┘"

  # Build the handoff context section (for continuity when resuming)
  HANDOFF_SECTION=""
  PREV_HANDOFF="$(find_previous_handoff "$session_num")"
  if [[ -n "$PREV_HANDOFF" ]]; then
    HANDOFF_SECTION="
---

## Previous Session Handoff

The previous session produced this handoff document. Use it for continuity:

$(cat "$PREV_HANDOFF")
"
    log "Found previous handoff: ${PREV_HANDOFF/#$REPO_ROOT\//}"
  fi

  # --- Build the full prompt with plan-then-execute structure ---
  FULL_PROMPT="$(cat <<PROMPT_EOF
You are executing Session $session_num of an automated multi-session epic.
Each session runs in a FRESH context — you have no memory of previous sessions.

=============================================
PHASE 1: OPERATOR RULES (always in effect)
=============================================

$OPERATOR_PROMPT

=============================================
PHASE 2: SESSION INSTRUCTIONS
=============================================

$SESSION_PROMPT
$HANDOFF_SECTION
=============================================
PHASE 3: EXECUTION PROTOCOL
=============================================

You MUST follow this three-step protocol:

### Step 1 — PLAN (do NOT write code yet)

Before making any changes:
1. Read all files referenced in the session instructions above.
2. Read any handoff documents from previous sessions (check docs/roadmap/ for session-*-handoff.md files).
3. Understand the current state of the codebase as it relates to this session's goals.
4. Print a concise plan with:
   - What you will change/create (files, modules, docs)
   - Key design decisions
   - Potential risks or ambiguities and how you'll resolve them
5. Do NOT ask for approval — proceed immediately to Step 2.

### Step 2 — EXECUTE

1. Implement everything described in the session instructions.
2. Make all code changes, create all deliverables, write all documentation.
3. If the session specifies tests or CI checks, run them.
4. If something is ambiguous, make the best decision and document your reasoning in the handoff doc.
5. Never leave TODOs, TBDs, or placeholders — resolve everything.

### Step 3 — FINALIZE

1. Run any relevant CI checks (e.g., cargo fmt, cargo clippy, cargo test) if you changed code.
2. Create the handoff document if the session instructions require one.
3. Stage and commit ALL changes with this exact format:

   git add -A
   git commit -m "feat: Session $session_num — $FRIENDLY_NAME

   <2-3 sentence summary of what was accomplished>

   Co-Authored-By: Claude <noreply@anthropic.com>"

4. Print a brief completion summary listing:
   - Files created/modified
   - Tests run and their results
   - Key decisions made
   - Any warnings or concerns for the next session
PROMPT_EOF
)"

  if $DRY_RUN; then
    warn "[DRY RUN] Would execute session $session_num (${#FULL_PROMPT} chars)"
    dim "  First 3 lines of session prompt:"
    echo "$SESSION_PROMPT" | head -3 | sed 's/^/    /'
    continue
  fi

  # Write prompt to temp file (avoids shell escaping issues)
  PROMPT_FILE="$(mktemp)"
  echo "$FULL_PROMPT" > "$PROMPT_FILE"

  SESSION_LOG="$SESSIONS_DIR/.session-${session_num}.log"

  log "Executing with model=$MODEL ... (log: ${SESSION_LOG/#$HOME/~})"
  START_TIME="$(date +%s)"

  set +e
  claude -p \
    --model "$MODEL" \
    --dangerously-skip-permissions \
    < "$PROMPT_FILE" \
    > "$SESSION_LOG" 2>&1
  EXIT_CODE=$?
  set -e

  rm -f "$PROMPT_FILE"
  END_TIME="$(date +%s)"
  ELAPSED=$(( END_TIME - START_TIME ))
  MINUTES=$(( ELAPSED / 60 ))
  SECONDS_REM=$(( ELAPSED % 60 ))

  if [[ $EXIT_CODE -ne 0 ]]; then
    err "Session $session_num FAILED (exit $EXIT_CODE) after ${MINUTES}m${SECONDS_REM}s"
    err "Check log: $SESSION_LOG"
    # Show last 20 lines of log for quick debugging
    echo ""
    err "Last 20 lines of log:"
    tail -20 "$SESSION_LOG" | sed 's/^/    /'
    FAILED=1
    break
  fi

  ok "Session $session_num completed in ${MINUTES}m${SECONDS_REM}s"

  # --- Auto-commit if Claude didn't already and there are uncommitted changes ---
  if $AUTO_COMMIT; then
    cd "$REPO_ROOT"
    if ! git diff --quiet HEAD 2>/dev/null || [[ -n "$(git ls-files --others --exclude-standard)" ]]; then
      git add -A
      COMMIT_MSG="feat: Session $session_num — $FRIENDLY_NAME

Automated execution of $fname.
See handoff doc and session log for details.

Co-Authored-By: Claude <noreply@anthropic.com>"

      git commit -m "$COMMIT_MSG" 2>/dev/null || true
      COMMIT_HASH="$(git rev-parse --short HEAD)"
      COMMITS+=("$COMMIT_HASH $fname")
      ok "Committed: $COMMIT_HASH"
    else
      log "No uncommitted changes (Claude may have committed already)"
      COMMIT_HASH="$(git rev-parse --short HEAD)"
      COMMITS+=("$COMMIT_HASH $fname (claude-committed)")
    fi
  fi
done

# --- Final summary ---
echo ""
if [[ $FAILED -eq 0 ]]; then
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  ok "${BOLD}Epic completed successfully!${NC}"
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  if [[ ${#COMMITS[@]} -gt 0 ]]; then
    ok ""
    ok "Commits:"
    for c in "${COMMITS[@]}"; do
      ok "  $c"
    done
  fi
  ok ""
  ok "Logs: $SESSIONS_DIR/.session-*.log"
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
  err "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  err "${BOLD}Epic stopped at session $session_num${NC}"
  err "Resume with: run-sessions.sh <dir> --start $session_num"
  err "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 1
fi
