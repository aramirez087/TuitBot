#!/usr/bin/env bash
# run-sessions.sh — DAG-aware epic runner. Executes a directed acyclic graph
# of session prompts, fanning out parallel waves into per-session worktrees
# and iteratively merging them into a coordinator trunk branch.
#
# Two-pass execution per session:
#   1. PLAN pass    — read-only exploration, writes an implementation plan file
#   2. EXECUTE pass — implements from the plan, commits changes
#
# Sessions can declare their DAG edges via YAML frontmatter (see epic-dag.py).
# Sessions without frontmatter implicitly form a linear chain — back-compat
# with pre-DAG epics.
#
# Layout:
#   .epic-worktrees/<repo>/
#     epic--<name>/                ← trunk worktree (branch: epic/<name>)
#     epic--<name>--s02-auth-api/  ← per-session worktree, ephemeral
#     epic--<name>--s03-email/     ← (parallel sibling)
#
# Usage:
#   run-sessions.sh <sessions-dir> [options]
#
# Options:
#   --start N            Resume from session N (default: 1)
#   --end N              Stop after session N (default: run all)
#   --max-parallel N     Max concurrent sessions per wave (default: 4)
#   --timeout MINS       Session timeout in minutes (default: 0 = disabled)
#   --retry N            Retry attempts per failed session (default: 0 = disabled)
#   --strict             Fail when sibling sessions declare overlapping `touches` globs
#   --sequential         Force one-session-per-wave (treats DAG as a linear chain)
#   --show-dag           Print the planned waves and exit
#   --dry-run            Preview without executing
#   --branch NAME        Trunk branch (default: epic/<name>)
#   --base BRANCH        Base branch for trunk (default: repo default)
#   --model MODEL        Model name (passed through to the CLI; e.g. opus, sonnet, haiku for Claude)
#   --cli CMD            Force CLI: opencode or claude (default: auto-detect)
#   --no-commit          Skip auto-commit fallback
#   --no-pr              Skip auto-PR creation
#   --skip-plan          Single-pass mode (no separate plan phase)
#   --no-worktree        Run trunk in CWD (forces --max-parallel 1)
#   --keep-worktree      Retain trunk worktree on success
#   --keep-session-worktrees  Retain per-session worktrees on success
#   -h, --help           Show this help

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

log()  { echo -e "${BLUE}[epic]${NC} $*"; }
ok()   { echo -e "${GREEN}[epic]${NC} $*"; }
warn() { echo -e "${YELLOW}[epic]${NC} $*"; }
err()  { echo -e "${RED}[epic]${NC} $*" >&2; }
dim()  { echo -e "${DIM}$*${NC}"; }

usage() {
  sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

# --- Parse arguments ---
SESSIONS_DIR=""
START_FROM=1
END_AT=999
MAX_PARALLEL=4
STRICT=false
SEQUENTIAL=false
SHOW_DAG=false
DRY_RUN=false
BRANCH=""
BASE_BRANCH=""
MODEL="sonnet"
AUTO_COMMIT=true
AUTO_PR=true
SKIP_PLAN=false
USE_WORKTREE=true
KEEP_WORKTREE=false
KEEP_SESSION_WORKTREES=false
CLI_OVERRIDE=""
TIMEOUT=0
RETRY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --start)                    START_FROM="$2"; shift 2 ;;
    --end)                      END_AT="$2"; shift 2 ;;
    --max-parallel)             MAX_PARALLEL="$2"; shift 2 ;;
    --strict)                   STRICT=true; shift ;;
    --sequential)               SEQUENTIAL=true; shift ;;
    --show-dag)                 SHOW_DAG=true; shift ;;
    --dry-run)                  DRY_RUN=true; shift ;;
    --branch)                   BRANCH="$2"; shift 2 ;;
    --base)                     BASE_BRANCH="$2"; shift 2 ;;
    --model)                    MODEL="$2"; shift 2 ;;
    --cli)                      CLI_OVERRIDE="$2"; shift 2 ;;
    --timeout)                  TIMEOUT="$2"; shift 2 ;;
    --retry)                    RETRY="$2"; shift 2 ;;
    --no-commit)                AUTO_COMMIT=false; shift ;;
    --no-pr)                    AUTO_PR=false; shift ;;
    --skip-plan)                SKIP_PLAN=true; shift ;;
    --no-worktree)              USE_WORKTREE=false; shift ;;
    --keep-worktree)            KEEP_WORKTREE=true; shift ;;
    --keep-session-worktrees)   KEEP_SESSION_WORKTREES=true; shift ;;
    --help|-h)                  usage ;;
    *)
      if [[ -z "$SESSIONS_DIR" ]]; then SESSIONS_DIR="$1"; shift
      else err "Unknown argument: $1"; usage; fi
      ;;
  esac
done

# Validate numeric arguments
for _arg_name in "START_FROM" "END_AT" "MAX_PARALLEL" "TIMEOUT" "RETRY"; do
  _val="${!_arg_name}"
  if ! [[ "$_val" =~ ^[0-9]+$ ]]; then
    err "$_arg_name must be a non-negative integer (got '$_val')"
    exit 1
  fi
done

if [[ -z "$SESSIONS_DIR" ]]; then
  err "Missing required argument: <sessions-dir>"
  usage
fi
if [[ ! -d "$SESSIONS_DIR" ]]; then
  err "Directory not found: $SESSIONS_DIR"
  exit 1
fi
SESSIONS_DIR="$(cd "$SESSIONS_DIR" && pwd)"

# Get repo root early for config file loading
if ! git rev-parse --is-inside-work-tree &>/dev/null; then
  err "Not inside a git repository."
  exit 1
fi
REPO_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd)"

# --- Load .epic-config.json configuration ---
CONFIG_FILE="$REPO_ROOT/.epic-config.json"
if [[ -f "$CONFIG_FILE" ]]; then
  log "Loading configuration from ${CONFIG_FILE/#$HOME/~}"
  
  # Extract config values using bash regex (Bash 3.2+ compatible)
  if config_content="$(cat "$CONFIG_FILE" 2>/dev/null)"; then
    # Only override defaults if CLI didn't specify the value
    if [[ "$TIMEOUT" -eq 0 && "$config_content" =~ \"timeout\"[[:space:]]*:[[:space:]]*([0-9]+) ]]; then
      TIMEOUT="${BASH_REMATCH[1]}"
    fi
    if [[ "$RETRY" -eq 0 && "$config_content" =~ \"retry\"[[:space:]]*:[[:space:]]*([0-9]+) ]]; then
      RETRY="${BASH_REMATCH[1]}"
    fi
    if [[ -z "$CLI_OVERRIDE" && "$config_content" =~ \"cli\"[[:space:]]*:[[:space:]]*\"([^\"]+)\" ]]; then
      CLI_OVERRIDE="${BASH_REMATCH[1]}"
    fi
    if [[ "$MODEL" == "sonnet" && "$config_content" =~ \"model\"[[:space:]]*:[[:space:]]*\"([^\"]+)\" ]]; then
      MODEL="${BASH_REMATCH[1]}"
    fi
    if [[ "$MAX_PARALLEL" -eq 4 && "$config_content" =~ \"maxParallel\"[[:space:]]*:[[:space:]]*([0-9]+) ]]; then
      MAX_PARALLEL="${BASH_REMATCH[1]}"
    fi
    if [[ "$AUTO_COMMIT" == "true" && "$config_content" =~ \"autoCommit\"[[:space:]]*:[[:space:]]*false ]]; then
      AUTO_COMMIT=false
    fi
    if [[ "$AUTO_PR" == "true" && "$config_content" =~ \"autoPr\"[[:space:]]*:[[:space:]]*false ]]; then
      AUTO_PR=false
    fi
    if [[ "$SKIP_PLAN" == "false" && "$config_content" =~ \"skipPlan\"[[:space:]]*:[[:space:]]*true ]]; then
      SKIP_PLAN=true
    fi
    if [[ "$KEEP_WORKTREE" == "false" && "$config_content" =~ \"keepWorktree\"[[:space:]]*:[[:space:]]*true ]]; then
      KEEP_WORKTREE=true
    fi
  else
    warn "Could not read configuration file: $CONFIG_FILE"
  fi
fi

# --- Detect CLI ---
# Prefer --cli override, then the invoking tool's env var, then PATH fallback.
# OPENCODE_SESSION_ID is set by OpenCode; CLAUDECODE is set by Claude Code.
CLI_CMD=""
if [[ -n "$CLI_OVERRIDE" ]]; then
  CLI_CMD="$CLI_OVERRIDE"
  if ! command -v "$CLI_CMD" &>/dev/null; then
    err "Forced CLI '$CLI_CMD' not found on PATH."
    exit 1
  fi
elif [[ -n "${OPENCODE_SESSION_ID:-}" ]]; then
  CLI_CMD=opencode
elif [[ -n "${CLAUDECODE:-}" ]]; then
  CLI_CMD=claude
fi
if [[ -z "$CLI_CMD" ]]; then
  if command -v opencode &>/dev/null; then
    CLI_CMD=opencode
  elif command -v claude &>/dev/null; then
    CLI_CMD=claude
  else
    err "Neither opencode nor claude CLI found on PATH. Use --cli to force one."
    exit 1
  fi
fi
log "Using CLI: $CLI_CMD"

# --- Map model shorthand to CLI-specific model IDs ---
# Claude accepts bare names (sonnet, opus, haiku).  OpenCode needs fully
# qualified IDs in provider/model format (opencode/claude-sonnet-4).
# If the user passed a slash-containing ID (e.g. opencode/glm-5.1) we
# leave it untouched — it's already a full ID.
map_model_shorthand() {
  local model="$1"
  local cli="$2"
  if [[ "$cli" == "opencode" && "$model" != */* ]]; then
    case "$model" in
      sonnet)   echo "opencode/claude-sonnet-4" ;;
      sonnet4)  echo "opencode/claude-sonnet-4" ;;
      opus)     echo "opencode/claude-opus-4-7" ;;
      haiku)    echo "opencode/claude-haiku-4-5" ;;
      gpt5)     echo "opencode/gpt-5" ;;
      gpt5nano) echo "opencode/gpt-5-nano" ;;
      gemini)   echo "opencode/gemini-3-flash" ;;
      glm)      echo "opencode/glm-5.1" ;;
      *)        log "Model '$model' is not a known OpenCode shorthand — using as-is"; echo "$model" ;;
    esac
  else
    echo "$model"
  fi
}

MODEL="$(map_model_shorthand "$MODEL" "$CLI_CMD")"

# Resolve python interpreter. On Windows, `python3` is often a Microsoft Store
# stub that satisfies `command -v` but errors on actual invocation, so we
# probe with --version instead of trusting PATH lookup alone.
PYTHON_CMD=""
if command -v python3 &>/dev/null && python3 --version &>/dev/null; then
  PYTHON_CMD=python3
elif command -v python &>/dev/null && python --version &>/dev/null; then
  PYTHON_CMD=python
else
  err "python3 (or python) not found on PATH (required for DAG scheduler)."
  exit 1
fi
# Force UTF-8 stdio so the DAG renderer's box-drawing characters don't crash
# on Windows consoles that default to cp1252.
export PYTHONIOENCODING=utf-8

# Resolve plugin root — works in both Claude Code (CLAUDE_PLUGIN_ROOT) and
# OpenCode (no env var; derive from script location).
if [[ -z "${CLAUDE_PLUGIN_ROOT:-}" ]]; then
  # Derive plugin root from this script's location (works when invoked directly
  # or via `bash run-sessions.sh` from the commands/ directory).
  _SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  CLAUDE_PLUGIN_ROOT="$(cd "$_SCRIPT_DIR/.." && pwd)"
  unset _SCRIPT_DIR
fi

# `git rev-parse --show-toplevel` on Git Bash for Windows returns Windows-style
# (`C:/foo/bar`), while `pwd` and SESSIONS_DIR are MSYS-style (`/c/foo/bar`).
# Mixing styles silently breaks the `${SESSIONS_DIR#$REPO_ROOT/}` prefix strip
# downstream and yields malformed paths like `//c/foo/...`. Normalize REPO_ROOT
# through `cd … && pwd` so every path uses the same style.
REPO_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd)"
ORIG_REPO_ROOT="$REPO_ROOT"
ORIG_SESSIONS_DIR="$SESSIONS_DIR"
EPIC_NAME_SLUG="$(basename "$SESSIONS_DIR")"

if [[ -z "$BRANCH" ]]; then
  BRANCH="epic/${EPIC_NAME_SLUG}"
fi
BRANCH_SANITIZED="$(echo "$BRANCH" | tr '/' '--')"

# --no-worktree forces sequential — can't safely parallelize in one CWD.
if ! $USE_WORKTREE && [[ "$MAX_PARALLEL" -gt 1 ]]; then
  warn "--no-worktree forces --max-parallel 1"
  MAX_PARALLEL=1
fi
if $SEQUENTIAL; then
  MAX_PARALLEL=1
fi

DAG_SCRIPT="$(dirname "$0")/epic-dag.py"
PROGRESS_SCRIPT="$(dirname "$0")/epic-progress.py"
UI_SCRIPT="$(dirname "$0")/epic-ui.py"

# --- Build the DAG plan ---
DAG_TMP="$(mktemp)"
trap 'rm -f "$DAG_TMP"' EXIT

DAG_ARGS=()
$STRICT && DAG_ARGS+=(--strict)

if [[ ${#DAG_ARGS[@]} -gt 0 ]]; then
  "$PYTHON_CMD" "$DAG_SCRIPT" "$SESSIONS_DIR" --bash "${DAG_ARGS[@]}" > "$DAG_TMP" || {
    err "DAG validation failed"; exit 1; }
else
  "$PYTHON_CMD" "$DAG_SCRIPT" "$SESSIONS_DIR" --bash > "$DAG_TMP" || {
    err "DAG validation failed"; exit 1; }
fi

OPERATOR_PATH=""
ANY_FRONTMATTER="false"
declare -a WAVE_IDS              # WAVE_IDS[$wave_num]="1 2 3"
declare -a SESSION_FILE_BASENAME # by id
declare -a SESSION_SLUG_BY_ID    # by id
declare -a SESSION_DEPS_BY_ID    # by id ("a,b,c" or "-")
declare -a SESSION_PARALLEL      # by id ("1"/"0")
declare -a SESSION_WAVE_OF       # by id
declare -a SESSION_MODEL_BY_ID   # by id
declare -a SESSION_CLI_BY_ID     # by id

WAVE_COUNT=0

while IFS= read -r line; do
  set -- $line
  case "$1" in
    META)
      case "$2" in
        operator_path=*) OPERATOR_PATH="${2#*=}" ;;
        any_frontmatter=*) ANY_FRONTMATTER="${2#*=}" ;;
        wave_count=*) WAVE_COUNT="${2#*=}" ;;
      esac
      ;;
    WAVE)
      # WAVE <num> <size>
      :
      ;;
    SESSION)
      # SESSION <wave> <id> <file> <deps> <slug> <parallel> [<model> <cli>]
      sw="$2"; sid="$3"; sfile="$4"; sdeps="$5"; sslug="$6"; sparallel="$7"
      smodel="${8:-}"; scli="${9:-}"
      WAVE_IDS[$sw]="${WAVE_IDS[$sw]:-} $sid"
      SESSION_FILE_BASENAME[$sid]="$sfile"
      SESSION_SLUG_BY_ID[$sid]="$sslug"
      SESSION_DEPS_BY_ID[$sid]="$sdeps"
      SESSION_PARALLEL[$sid]="$sparallel"
      SESSION_WAVE_OF[$sid]="$sw"
      SESSION_MODEL_BY_ID[$sid]="$smodel"
      SESSION_CLI_BY_ID[$sid]="$scli"
      ;;
  esac
done < "$DAG_TMP"

if [[ -z "$OPERATOR_PATH" || "$WAVE_COUNT" -eq 0 ]]; then
  err "DAG plan is empty or malformed"
  exit 1
fi

# --- Extract prompt from markdown code fence ---
extract_prompt() {
  local file="$1" content
  content="$(awk '
    /^```md$/ { capture=1; next }
    /^```$/   { if (capture) exit }
    capture   { print }
  ' "$file")"
  if [[ -z "$content" ]]; then
    # Strip frontmatter and the title line, return the rest.
    content="$(awk '
      BEGIN { in_fm=0; fm_done=0 }
      NR==1 && /^---$/ { in_fm=1; next }
      in_fm && /^---$/ { in_fm=0; fm_done=1; next }
      in_fm { next }
      fm_done && /^# / { fm_done=0; next }
      { print }
    ' "$file")"
  fi
  echo "$content"
}

OPERATOR_PROMPT="$(extract_prompt "$OPERATOR_PATH")"
if [[ -z "$OPERATOR_PROMPT" ]]; then
  err "Could not extract operator prompt from $OPERATOR_PATH"
  exit 1
fi

# --- Show DAG and exit if requested ---
if $SHOW_DAG; then
  "$PYTHON_CMD" "$DAG_SCRIPT" "$SESSIONS_DIR" --show
  exit 0
fi

# --- Dry-run: print plan and exit BEFORE any side effects (worktree, branches) ---
if $DRY_RUN; then
  log "${BOLD}DRY RUN${NC} — no worktrees, no branches, no commits will be created."
  "$PYTHON_CMD" "$DAG_SCRIPT" "$SESSIONS_DIR" --show
  echo ""
  for (( wn=1; wn<=WAVE_COUNT; wn++ )); do
    ids="${WAVE_IDS[$wn]:-}"
    ids="$(echo "$ids" | xargs)"
    [[ -z "$ids" ]] && continue
    IFS=' ' read -ra IDS_ARR <<< "$ids"
    in_range=()
    for sid in "${IDS_ARR[@]}"; do
      [[ "$sid" -lt "$START_FROM" ]] && continue
      [[ "$sid" -gt "$END_AT" ]]    && continue
      in_range+=("$sid")
    done
    [[ ${#in_range[@]} -eq 0 ]] && continue
    if [[ ${#in_range[@]} -gt 1 ]]; then
      log "Wave $wn: ${#in_range[@]} sessions in parallel (max $MAX_PARALLEL)"
    else
      log "Wave $wn: 1 session (serial)"
    fi
    for sid in "${in_range[@]}"; do
      dim "  [DRY] would run session $(printf '%02d' "$sid") — ${SESSION_SLUG_BY_ID[$sid]} (deps: ${SESSION_DEPS_BY_ID[$sid]})"
    done
  done
  echo ""
  ok "Dry-run complete. Re-run without --dry-run to execute."
  exit 0
fi

# --- Trunk worktree setup ---
WORKTREE_BASE="$(dirname "$ORIG_REPO_ROOT")/.epic-worktrees/$(basename "$ORIG_REPO_ROOT")"
TRUNK_WORKTREE_DIR=""

# Clean up stale session worktrees from previous runs
if [[ -d "$WORKTREE_BASE" ]]; then
  log "Scanning for stale session worktrees..."
  
  # Get list of current session IDs from DAG
  current_session_ids=""
  while IFS= read -r line; do
    set -- $line
    case "$1" in
      SESSION) current_session_ids="$current_session_ids $3" ;;
    esac
  done < "$DAG_TMP"
  
  cleaned_count=0
  for stale_wt in "$WORKTREE_BASE"/*--s[0-9][0-9]-*; do
    [[ ! -d "$stale_wt" ]] && continue
    
    # Extract session ID from worktree name
    if [[ "$(basename "$stale_wt")" =~ --s([0-9][0-9])- ]]; then
      stale_id="${BASH_REMATCH[1]}"
      # Remove leading zero for comparison
      stale_id="$((10#$stale_id))"
      
      # Check if this session ID is in current DAG
      if [[ ! " $current_session_ids " =~ " $stale_id " ]]; then
        log "  ↻ removing stale worktree for session $stale_id: $(basename "$stale_wt")"
        git worktree remove "$stale_wt" --force 2>/dev/null || rm -rf "$stale_wt"
        cleaned_count=$((cleaned_count + 1))
      fi
    fi
  done
  
  [[ $cleaned_count -gt 0 ]] && log "Cleaned $cleaned_count stale session worktrees"
fi

if $USE_WORKTREE; then
  if [[ -z "$BASE_BRANCH" ]]; then
    BASE_BRANCH="$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')" || true
    [[ -z "$BASE_BRANCH" ]] && BASE_BRANCH="main"
  fi

  TRUNK_WORKTREE_DIR="$WORKTREE_BASE/$BRANCH_SANITIZED"
  if [[ -d "$TRUNK_WORKTREE_DIR" ]]; then
    log "Reusing trunk worktree: $TRUNK_WORKTREE_DIR"
  else
    mkdir -p "$WORKTREE_BASE"
    if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
      log "Creating trunk worktree on existing branch: $BRANCH"
      git worktree add "$TRUNK_WORKTREE_DIR" "$BRANCH"
    else
      log "Creating trunk worktree with new branch: $BRANCH (base: $BASE_BRANCH)"
      git worktree add -b "$BRANCH" "$TRUNK_WORKTREE_DIR" "$BASE_BRANCH"
    fi
  fi

  cd "$TRUNK_WORKTREE_DIR"
  REPO_ROOT="$TRUNK_WORKTREE_DIR"

  # Mirror sessions dir into trunk if absent (uncommitted-source case).
  TRUNK_SESSIONS_REL="${ORIG_SESSIONS_DIR#$ORIG_REPO_ROOT/}"
  TRUNK_SESSIONS_DIR="$TRUNK_WORKTREE_DIR/$TRUNK_SESSIONS_REL"
  if [[ ! -d "$TRUNK_SESSIONS_DIR" ]] || [[ -z "$(ls "$TRUNK_SESSIONS_DIR"/session-*.md 2>/dev/null)" ]]; then
    log "Syncing session files into trunk worktree..."
    mkdir -p "$TRUNK_SESSIONS_DIR"
    cp "$ORIG_SESSIONS_DIR"/session-*.md "$TRUNK_SESSIONS_DIR/" 2>/dev/null || true
  fi

  # Bootstrap commit so per-session worktrees inherit the session files.
  if ! git diff --quiet HEAD -- "$TRUNK_SESSIONS_DIR" 2>/dev/null \
     || [[ -n "$(git ls-files --others --exclude-standard "$TRUNK_SESSIONS_DIR")" ]]; then
    git add "$TRUNK_SESSIONS_DIR"
    git commit -q -m "chore: bootstrap epic session prompts

Sync session prompt files onto the epic trunk so per-session worktrees
inherit them when fanning out parallel waves.

Co-Authored-By: AI <noreply@ai>" || true
    log "Bootstrapped session prompts onto trunk"
  fi

  ok "Trunk worktree: $TRUNK_WORKTREE_DIR"
else
  CURRENT_BRANCH="$(git branch --show-current)"
  if [[ "$CURRENT_BRANCH" != "$BRANCH" ]]; then
    if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
      git checkout "$BRANCH"
    else
      git checkout -b "$BRANCH"
    fi
  fi
  TRUNK_WORKTREE_DIR="$REPO_ROOT"
  TRUNK_SESSIONS_DIR="$SESSIONS_DIR"
fi

# --- Run mode ---
if $SKIP_PLAN; then EXEC_MODE="execute-only"; else EXEC_MODE="plan+execute"; fi

# --- Banner ---
echo ""
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "${BOLD}Epic Runner — DAG mode${NC}"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "Sessions dir : ${TRUNK_SESSIONS_DIR/#$HOME/~}"
log "Operator     : $(basename "$OPERATOR_PATH")"
log "Waves        : $WAVE_COUNT"
log "Range        : $START_FROM → $END_AT"
log "Model        : $MODEL"
log "Mode         : $EXEC_MODE"
log "Max parallel : $MAX_PARALLEL"
[[ "$TIMEOUT" -gt 0 ]] && log "Timeout      : ${TIMEOUT}m per session"
[[ "$RETRY" -gt 0 ]] && log "Retry        : up to $RETRY attempts per session"
log "Frontmatter  : $ANY_FRONTMATTER"
log "Branch       : $BRANCH (trunk)"
[[ -n "$TRUNK_WORKTREE_DIR" && "$TRUNK_WORKTREE_DIR" != "$ORIG_REPO_ROOT" ]] \
  && log "Trunk wt     : ${TRUNK_WORKTREE_DIR/#$HOME/~}"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
"$PYTHON_CMD" "$DAG_SCRIPT" "$SESSIONS_DIR" --show
echo ""

# --- Live UI setup ---
STATUS_FILE="${TRUNK_SESSIONS_DIR}/.epic-status.json"
DAG_PLAN_FILE="${TRUNK_SESSIONS_DIR}/.epic-dag-plan.bash"
UI_PID=""
LIVE_UI=false

# Copy the dag plan to a stable path (DAG_TMP is deleted on EXIT)
cp "$DAG_TMP" "$DAG_PLAN_FILE"

# Initialize shared status JSON (all sessions start as "pending")
"$PYTHON_CMD" - "$STATUS_FILE" "$EPIC_NAME_SLUG" "$DAG_PLAN_FILE" <<'PYEOF_INIT'
import sys, json, time
sf, epic, pf = sys.argv[1], sys.argv[2], sys.argv[3]
sessions = {}
with open(pf, encoding='utf-8', errors='replace') as f:
    for line in f:
        line = line.strip()
        if not line.startswith('SESSION '): continue
        parts = line.split(None, 6)
        wn, sid, _, _, slug = parts[1], parts[2], parts[3], parts[4], parts[5]
        sessions[sid] = {
            'status': 'pending', 'slug': slug, 'wave': int(wn),
            'title': slug.replace('-', ' ').title(),
            'step': 0, 'tool': '', 'target': '', 'elapsed': 0.0,
        }
with open(sf, 'w') as f:
    json.dump({'epic': epic, 'started_at': time.time(), 'sessions': sessions}, f, indent=2)
PYEOF_INIT

# Launch live dashboard only when stderr is a real TTY (cursor-up redraws work).
# In Claude Code / piped contexts isatty() is false — shell's existing colored
# output handles progress there; no background watcher needed.
if [[ -f "$UI_SCRIPT" && -t 2 ]]; then
  "$PYTHON_CMD" "$UI_SCRIPT" \
    --plan-bash "$DAG_PLAN_FILE" \
    --status-file "$STATUS_FILE" \
    --epic "$EPIC_NAME_SLUG" &
  UI_PID=$!
  LIVE_UI=true
  sleep 0.3   # brief pause so initial panel renders before wave messages
fi

# --- Helpers ---

format_elapsed() {
  local s=$1
  printf "%dm%02ds" $((s / 60)) $((s % 60))
}

# Look up a session id's handoff file under docs/roadmap/<epic>/
# Search the current epic's roadmap dir first (intra-epic dependency),
# then fall back to all epic dirs (cross-epic handoff).
find_handoff_for() {
  local pid="$1" padded
  padded="$(printf "%02d" "$pid")"
  for cand in "$REPO_ROOT/docs/roadmap/$EPIC_NAME_SLUG/session-${padded}-handoff.md"; do
    [[ -f "$cand" ]] && { echo "$cand"; return; }
  done
  for cand in "$REPO_ROOT/docs/roadmap/"*"/session-${padded}-handoff.md"; do
    [[ -f "$cand" ]] && { echo "$cand"; return; }
  done
}

# Build the multi-parent handoff section for a session
build_handoff_section() {
  local deps_csv="$1"
  local out="" pid path
  [[ "$deps_csv" == "-" || -z "$deps_csv" ]] && return
  out="
---

## Previous Session Handoffs

These handoff documents come from your DAG parents. Treat their file paths as
the only memory of prior work — you have no other recollection.
"
  IFS=',' read -ra parents <<< "$deps_csv"
  for pid in "${parents[@]}"; do
    path="$(find_handoff_for "$pid")"
    if [[ -n "$path" ]]; then
      out+="
### From session $(printf '%02d' "$pid")  →  ${path/#$REPO_ROOT\//}

$(cat "$path")
"
    fi
  done
  echo "$out"
}

# Run a single AI-CLI pipe-invocation with optional progress stream.
# Args: <prompt_file> <log_file> <phase_label> <quiet:true|false> [session_id]
DISALLOWED_TOOLS="EnterPlanMode,ExitPlanMode,AskUserQuestion,EnterWorktree"

run_cli() {
  local prompt_file="$1" log_file="$2" phase_label="$3" quiet="${4:-false}" sid="${5:-0}"

  # Check for session-specific overrides
  local session_model="$MODEL"
  local session_cli="$CLI_CMD"
  if [[ "$sid" -gt 0 && -n "${SESSION_MODEL_BY_ID[$sid]:-}" ]]; then
    local raw_model="${SESSION_MODEL_BY_ID[$sid]}"
    session_model="$(map_model_shorthand "$raw_model" "$session_cli")"
  fi
  if [[ "$sid" -gt 0 && -n "${SESSION_CLI_BY_ID[$sid]:-}" ]]; then
    session_cli="${SESSION_CLI_BY_ID[$sid]}"
    # Re-map model if CLI changed (e.g. session overrides cli from opencode to claude
    # or vice versa — model shorthand depends on which CLI is in use)
    if [[ "$sid" -gt 0 && -n "${SESSION_MODEL_BY_ID[$sid]:-}" ]]; then
      local raw_model2="${SESSION_MODEL_BY_ID[$sid]}"
      session_model="$(map_model_shorthand "$raw_model2" "$session_cli")"
    fi
  fi

  # Build progress args (status file updates work even in quiet/parallel mode)
  local progress_args=(--log "$log_file" --phase "$phase_label")
  if [[ -n "${STATUS_FILE:-}" && -f "${STATUS_FILE:-}" && "$sid" -gt 0 ]]; then
    progress_args+=(--session-id "$sid" --status-file "$STATUS_FILE")
  fi

  # Build timeout wrapper if enabled  
  local timeout_cmd=()
  if [[ "$TIMEOUT" -gt 0 ]]; then
    if command -v timeout >/dev/null 2>&1; then
      timeout_cmd=(timeout $((TIMEOUT * 60)))
    else
      warn "timeout command not available, ignoring --timeout setting"
    fi
  fi

  # Build the model flag using session-specific model
  local model_flag=()
  if [[ "$session_cli" == "opencode" ]]; then
    if [[ -n "$session_model" ]]; then model_flag=(-m "$session_model"); fi
  else
    model_flag=(--model "$session_model")
  fi

  set +e
  if [[ -f "$PROGRESS_SCRIPT" ]]; then
    local stderr_tmp="${log_file%.log}-stderr.tmp"
    if [[ "$session_cli" == "claude" ]]; then
      # Claude Code with timeout wrapper
      if [[ "$quiet" == "false" ]]; then
        ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u CLAUDECODE claude -p \
          "${model_flag[@]}" \
          --dangerously-skip-permissions \
          --disallowedTools "$DISALLOWED_TOOLS" \
          --output-format stream-json \
          --verbose \
          < "$prompt_file" \
          2>"$stderr_tmp" \
          | "$PYTHON_CMD" "$PROGRESS_SCRIPT" "${progress_args[@]}"
      else
        ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u CLAUDECODE claude -p \
          "${model_flag[@]}" \
          --dangerously-skip-permissions \
          --disallowedTools "$DISALLOWED_TOOLS" \
          --output-format stream-json \
          --verbose \
          < "$prompt_file" \
          2>"$stderr_tmp" \
          | "$PYTHON_CMD" "$PROGRESS_SCRIPT" "${progress_args[@]}" 2>/dev/null
      fi
      local exit_code=${PIPESTATUS[0]}
      cat "$stderr_tmp" >> "$log_file" 2>/dev/null || true
      rm -f "$stderr_tmp"
    else
      # OpenCode with timeout wrapper
      if [[ "$quiet" == "false" ]]; then
        ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u OPENCODE_SESSION_ID opencode run "${model_flag[@]}" \
          --format json \
          --dangerously-skip-permissions \
          < "$prompt_file" \
          2>"$stderr_tmp" \
          | "$PYTHON_CMD" "$PROGRESS_SCRIPT" "${progress_args[@]}"
      else
        ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u OPENCODE_SESSION_ID opencode run "${model_flag[@]}" \
          --format json \
          --dangerously-skip-permissions \
          < "$prompt_file" \
          2>"$stderr_tmp" \
          | "$PYTHON_CMD" "$PROGRESS_SCRIPT" "${progress_args[@]}" 2>/dev/null
      fi
      local exit_code=${PIPESTATUS[0]}
      cat "$stderr_tmp" >> "$log_file" 2>/dev/null || true
      rm -f "$stderr_tmp"
    fi
  else
    # No progress script available
    if [[ "$session_cli" == "claude" ]]; then
      ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u CLAUDECODE claude -p \
        "${model_flag[@]}" \
        --dangerously-skip-permissions \
        --disallowedTools "$DISALLOWED_TOOLS" \
        < "$prompt_file" > "$log_file" 2>&1
      local exit_code=$?
    else
      ${timeout_cmd[@]+"${timeout_cmd[@]}"} env -u OPENCODE_SESSION_ID opencode run "${model_flag[@]}" \
        --format json \
        --dangerously-skip-permissions \
        < "$prompt_file" > "$log_file" 2>&1
      local exit_code=$?
    fi
  fi
  
  # Check for timeout
  if [[ $exit_code -eq 124 ]]; then
    echo "ERROR: Session timed out after ${TIMEOUT}m" >> "$log_file"
  fi
  
  set -e
  return $exit_code
}

# Backward-compat alias
run_claude() { run_cli "$@"; }

# Atomically update one session's entry in the shared status JSON.
# Args: <status_file> <session_id> <new_status> [elapsed_seconds]
mark_session() {
  local sf="${1:-}" sid="$2" st="$3" elapsed="${4:-0}"
  [[ -z "$sf" || ! -f "$sf" ]] && return 0
  "$PYTHON_CMD" - "$sf" "$sid" "$st" "$elapsed" <<'PYEOF'
import sys, json, os, time
sf, sid, st = sys.argv[1], sys.argv[2], sys.argv[3]
elapsed = float(sys.argv[4]) if len(sys.argv) > 4 else 0
lock = sf + '.lock'
for _ in range(200):
    try:
        fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        os.close(fd); break
    except (FileExistsError, OSError):
        time.sleep(0.05)
else:
    sys.exit(0)
try:
    with open(sf, encoding='utf-8') as f: data = json.load(f)
    entry = data['sessions'].setdefault(sid, {})
    entry['status'] = st
    if st == 'running': entry['started_at'] = time.time()
    if elapsed > 0: entry['elapsed'] = elapsed
    tmp = sf + '.tmp'
    with open(tmp, 'w', encoding='utf-8') as f: json.dump(data, f, indent=2)
    os.replace(tmp, sf)
finally:
    try: os.unlink(lock)
    except: pass
PYEOF
}

# ---------------------------------------------------------------------------
# run_one_session — execute one session inside its dedicated worktree.
#
# Args (positional):
#   1: session id (int)
#   2: per-session worktree dir
#   3: friendly name (lowercase words)
#   4: handoff section text (may be empty)
#   5: quiet mode ("true" in parallel waves, "false" otherwise)
#
# Reads from globals: $TRUNK_SESSIONS_DIR, $OPERATOR_PROMPT, $SKIP_PLAN, $MODEL
# Writes plan/exec logs at $TRUNK_SESSIONS_DIR/.session-NN-{plan,exec}.log
# Writes plan markdown at $TRUNK_SESSIONS_DIR/.session-NN-plan.md
# Returns: 0 on success, non-zero on failure.
# ---------------------------------------------------------------------------
run_one_session() {
  local sid="$1" wt_dir="$2" friendly="$3" handoff_text="$4" quiet="$5"
  local fname="${SESSION_FILE_BASENAME[$sid]}"
  local session_path="$TRUNK_SESSIONS_DIR/$fname"
  local plan_file="$TRUNK_SESSIONS_DIR/.session-${sid}-plan.md"
  local plan_log="$TRUNK_SESSIONS_DIR/.session-${sid}-plan.log"
  local exec_log="$TRUNK_SESSIONS_DIR/.session-${sid}-exec.log"
  local session_prompt
  session_prompt="$(extract_prompt "$session_path")"
  if [[ -z "$session_prompt" ]]; then
    echo "ERROR: could not extract prompt from $session_path" > "$exec_log"
    return 2
  fi

  local prev_cwd; prev_cwd="$(pwd)"
  cd "$wt_dir"

  local rc=0
  local attempt=1
  local max_attempts=$((RETRY + 1))
  
  while [[ $attempt -le $max_attempts ]]; do
    if [[ $attempt -gt 1 ]]; then
      echo "Retry attempt $attempt of $max_attempts" >> "$exec_log"
    fi
    
    if $SKIP_PLAN; then
      local prompt; prompt="$(cat <<SINGLE_EOF
Execute Session ${sid}. Read referenced files and handoff docs, implement, commit.

OPERATOR RULES:
$OPERATOR_PROMPT

SESSION INSTRUCTIONS:
$session_prompt
$handoff_text

DO:
1. Read all referenced files and handoff docs.
2. Implement everything — no TBDs/TODOs.
3. Run all CI checks. Create the handoff doc if required.
4. Stage and commit ALL changes:
   git add -A
   git commit -m "feat: Session ${sid} — ${friendly}"
SINGLE_EOF
)"
       local prompt_file; prompt_file="$(mktemp)"
       echo "$prompt" > "$prompt_file"
       run_claude "$prompt_file" "$exec_log" "S${sid}-EXEC" "$quiet" "$sid" || rc=$?
       rm -f "$prompt_file"
     else
        # PLAN pass
        local plan_prompt; plan_prompt="$(cat <<PLAN_EOF
Read-only planning for Session ${sid}. Do NOT modify source, tests, or docs.

OPERATOR RULES:
$OPERATOR_PROMPT

SESSION INSTRUCTIONS:
$session_prompt
$handoff_text

TASK: Write an implementation plan to ${plan_file} containing:
- Files to create/modify (full paths)
- Design decisions and rationale
- Risks and mitigations
- Verification steps (tests, CI commands)
- Exact order of operations

Write the plan and finish. Do not wait for approval.
PLAN_EOF
)"
       local prompt_file; prompt_file="$(mktemp)"
       echo "$plan_prompt" > "$prompt_file"
       if ! run_claude "$prompt_file" "$plan_log" "S${sid}-PLAN" "$quiet" "$sid"; then
         rc=1
       else
         rm -f "$prompt_file"

         if [[ ! -f "$plan_file" ]]; then
           echo "ERROR: plan phase finished but plan file was not created: $plan_file" >> "$exec_log"
           rc=1
         else
           local plan_contents; plan_contents="$(cat "$plan_file")"

           # EXECUTE pass
            local exec_prompt; exec_prompt="$(cat <<EXEC_EOF
Execute Session ${sid} using the plan below. Implement fully — no TBDs/TODOs, no re-exploration.

OPERATOR RULES:
$OPERATOR_PROMPT

SESSION INSTRUCTIONS:
$session_prompt
$handoff_text

PLAN:
$plan_contents

DO:
1. Implement everything in the plan.
2. Run all CI checks the plan calls for.
3. Create the handoff doc if required.
4. Stage and commit ALL changes:
   git add -A
   git commit -m "feat: Session ${sid} — ${friendly}"

Commit with descriptive message. Execute immediately.
EXEC_EOF
)"
           prompt_file="$(mktemp)"
           echo "$exec_prompt" > "$prompt_file"
           run_claude "$prompt_file" "$exec_log" "S${sid}-EXEC" "$quiet" "$sid" || rc=$?
           rm -f "$prompt_file"
         fi
       fi
       rm -f "$prompt_file"
     fi
    
     # Break on success or final attempt
     if [[ $rc -eq 0 || $attempt -eq $max_attempts ]]; then
       break
     fi
     
     attempt=$((attempt + 1))
     echo "Session failed, retrying in 5 seconds..." >> "$exec_log"
     sleep 5
   done

  # Auto-commit fallback inside the session worktree
  if [[ $rc -eq 0 ]] && $AUTO_COMMIT; then
    if ! git diff --quiet HEAD 2>/dev/null \
       || [[ -n "$(git ls-files --others --exclude-standard)" ]]; then
      git add -A
      git commit -q -m "feat: Session ${sid} — ${friendly}

Automated execution of $fname.

Co-Authored-By: AI <noreply@ai>" || true
    fi
  fi

  cd "$prev_cwd"
  return $rc
}

# ---------------------------------------------------------------------------
# Error classification — determine why a session failed
# ---------------------------------------------------------------------------
classify_error() {
  local log_file="$1" exit_code="$2"
  
  if [[ "$exit_code" -eq 124 ]]; then
    echo "timeout"
    return
  fi
  
  if grep -q "ERROR: could not extract prompt" "$log_file" 2>/dev/null; then
    echo "prompt_error"
    return
  fi
  
  if grep -q "ERROR: plan phase finished but plan file was not created" "$log_file" 2>/dev/null; then
    echo "plan_failure"
    return
  fi
  
  if grep -qi "merge conflict" "$log_file" 2>/dev/null; then
    echo "merge_conflict"
    return
  fi
  
  # Check for error markers in the log
  if grep -qE '^\[ERROR\]' "$log_file" 2>/dev/null; then
    local err_msg
    err_msg="$(grep -oE '^\[ERROR\] .+' "$log_file" | head -1 | sed 's/^\[ERROR\] //')"
    err_msg="$(echo "$err_msg" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g')"
    echo "cli_error"
    echo "$err_msg" > "${log_file}.errtype"
    return
  fi
  
  # Check for CLI-specific error patterns
  if grep -qE '(exit code|returned|fatal|Segmentation|Killed|out of memory|OOM)' "$log_file" 2>/dev/null; then
    echo "cli_crash"
    return
  fi
  
  echo "unknown"
}

# ---------------------------------------------------------------------------
# Generate epic result file and structured output block for orchestrator
# ---------------------------------------------------------------------------
write_epic_result() {
  local epic_status="$1"  # "success" or "failed"
  local result_file="${TRUNK_SESSIONS_DIR}/.epic-result.json"
  local result_wave
  if $EPIC_FAILED; then result_wave="$wn"; else result_wave="$WAVE_COUNT"; fi
  
  local start_ts="$EPIC_START_TS"
  local end_ts
  end_ts="$(date +%s)"
  local runtime=$(( end_ts - start_ts ))
  
  # Write session data to temp file for Python to consume
  local tmp_data
  tmp_data="$(mktemp)"
  for (( sid=1; sid<=999; sid++ )); do
    [[ -z "${SESSION_STATUS[$sid]:-}" ]] && break
    local padded
    padded="$(printf '%02d' "$sid")"
    local key="session-${padded}"
    local status="${SESSION_STATUS[$sid]:-unknown}"
    local slug="${SESSION_SLUG_BY_ID[$sid]:-unknown}"
    local elapsed="${SESSION_ELAPSED_BY_ID[$sid]:-0}"
    local exit_code="${SESSION_EXIT_BY_ID[$sid]:-0}"
    local error_type="none"
    local log_path="$TRUNK_SESSIONS_DIR/.session-${padded}-exec.log"
    local stderr_path="$TRUNK_SESSIONS_DIR/.session-${padded}-exec.log.errtype"
    
    if [[ "$status" == "failed" ]]; then
      error_type="$(classify_error "$TRUNK_SESSIONS_DIR/.session-${padded}-exec.log" "$exit_code")"
    fi
    
    echo "${key}|${status}|${slug}|${elapsed}|${exit_code}|${error_type}|${log_path}|${stderr_path}" >> "$tmp_data"
  done
  
  # Build merged sessions list via temp file
  local tmp_merged
  tmp_merged="$(mktemp)"
  for entry in "${MERGED_SESSIONS[@]:-}"; do
    [[ -z "$entry" ]] && continue
    echo "$entry" >> "$tmp_merged"
  done
  
  local first_failed="null"
  if [[ -n "$FIRST_FAILED_ID" ]]; then
    local padded_ff
    padded_ff="$(printf '%02d' "$FIRST_FAILED_ID")"
    first_failed="\"session-${padded_ff}\""
  fi
  
  local epic_status_lower
  if [[ "$epic_status" == "success" ]]; then
    epic_status_lower="success"
  else
    epic_status_lower="failed"
  fi
  
  # Write result file using Python for safe JSON generation
  EPIC_NAME="$EPIC_NAME_SLUG" EPIC_STATUS="$epic_status_lower" \
  RESULT_WAVE="$result_wave" RUNTIME_SECS="$runtime" \
  START_TS="$start_ts" END_TS="$end_ts" \
  FIRST_FAILED="$first_failed" \
    "$PYTHON_CMD" - "$result_file" "$tmp_data" "$tmp_merged" <<'PYEOF_RESULT'
import json, os, sys

data = {
    'epic':           os.environ['EPIC_NAME'],
    'status':         os.environ['EPIC_STATUS'],
    'first_failed_id': json.loads(os.environ['FIRST_FAILED']),
    'sessions':       {},
    'merged_sessions': [l.strip() for l in open(sys.argv[3]) if l.strip()],
    'wave':           int(os.environ['RESULT_WAVE']),
    'runtime_seconds': int(os.environ['RUNTIME_SECS']),
    'started_at':     int(os.environ['START_TS']),
    'ended_at':       int(os.environ['END_TS']),
}

with open(sys.argv[2], encoding='utf-8') as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        parts = line.split('|')
        sid = parts[0]
        status = parts[1]
        slug = parts[2]
        elapsed = int(parts[3]) if len(parts) > 3 else 0
        entry = {'status': status, 'slug': slug, 'elapsed': elapsed}
        if status == 'failed':
            entry['exit_code'] = int(parts[4]) if len(parts) > 4 else 1
            entry['error_type'] = parts[5] if len(parts) > 5 else 'unknown'
            entry['log_path'] = parts[6] if len(parts) > 6 else ''
            stderr_path = parts[7] if len(parts) > 7 else ''
            if stderr_path:
                try:
                    with open(stderr_path, encoding='utf-8') as ef:
                        entry['error_detail'] = ef.read().strip()
                except Exception:
                    pass
        data['sessions'][sid] = entry

with open(sys.argv[1], 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
PYEOF_RESULT
  
  rm -f "$tmp_data" "$tmp_merged"
  
  # Clean up .errtype temp files created by classify_error
  for (( sid=1; sid<=999; sid++ )); do
    [[ -z "${SESSION_STATUS[$sid]:-}" ]] && break
    rm -f "$TRUNK_SESSIONS_DIR/.session-$(printf '%02d' "$sid")-exec.log.errtype"
  done
  
  # Print structured output block for orchestrator consumption
  echo ""
  echo "[EPIC_RESULT_START]"
  "$PYTHON_CMD" - "$result_file" <<'PYEOF_PRINT'
import json, sys
data = json.load(open(sys.argv[1]))
if data['status'] == 'failed':
    print('STATUS=failed')
    print('FIRST_FAILED=' + str(data['first_failed_id']))
    print('WAVE=' + str(data['wave']))
    print('RUNTIME=' + str(data['runtime_seconds']) + 's')
    for sid, sdata in data['sessions'].items():
        if sdata['status'] == 'failed':
            print('SESSION_FAIL=' + sid + ' error=' + sdata.get('error_type','unknown') + ' exit=' + str(sdata.get('exit_code',0)) + ' log=' + sdata.get('log_path',''))
            if sdata.get('error_detail'):
                print('ERROR_DETAIL=' + sdata['error_detail'])
else:
    print('STATUS=success')
    print('SESSIONS_COMPLETED=' + str(len(data['sessions'])))
    print('RUNTIME=' + str(data['runtime_seconds']) + 's')
PYEOF_PRINT
  echo "[EPIC_RESULT_END]"
  echo ""
  
  # Only retain result file on failure (clean up on success)
  if [[ "$epic_status" == "success" ]]; then
    rm -f "$result_file"
  fi
}

# ---------------------------------------------------------------------------
# Wave loop
# ---------------------------------------------------------------------------
declare -a SESSION_STATUS         # by id: pending|done|failed|skipped
declare -a SESSION_BRANCH_BY_ID   # by id
declare -a SESSION_WT_BY_ID       # by id
declare -a SESSION_START_TS       # by id (epoch seconds)
declare -a SESSION_ELAPSED_BY_ID  # by id (seconds, populated on reap)
declare -a SESSION_EXIT_BY_ID     # by id (exit code, populated on reap)
declare -a MERGED_SESSIONS        # ids merged into trunk

EPIC_FAILED=false
FIRST_FAILED_ID=""
EPIC_START_TS="$(date +%s)"

# reap_finished_jobs — defined once, called each wave iteration.
# Uses JOB_PIDS[], JOB_SIDS[] (re-initialized per wave) and
# SESSION_STATUS[], SESSION_ELAPSED_BY_ID[], SESSION_EXIT_BY_ID[] (global).
reap_finished_jobs() {
  local i new_pids=() new_sids=()
  for i in "${!JOB_PIDS[@]}"; do
    if ! kill -0 "${JOB_PIDS[$i]}" 2>/dev/null; then
      local rc=0
      wait "${JOB_PIDS[$i]}" || rc=$?
      local sid="${JOB_SIDS[$i]}"
      local elapsed=$(( $(date +%s) - ${SESSION_START_TS[$sid]:-0} ))
      SESSION_ELAPSED_BY_ID[$sid]=$elapsed
      SESSION_EXIT_BY_ID[$sid]=$rc
      if [[ $rc -eq 0 ]]; then
        SESSION_STATUS[$sid]="done"
        mark_session "${STATUS_FILE:-}" "$sid" "done" "$elapsed"
        ! $LIVE_UI && ok "  ✓ session $(printf '%02d' "$sid") (${SESSION_SLUG_BY_ID[$sid]}) finished in $(format_elapsed "$elapsed")"
      else
        SESSION_STATUS[$sid]="failed"
        mark_session "${STATUS_FILE:-}" "$sid" "failed" "$elapsed"
        ! $LIVE_UI && err "  ✗ session $(printf '%02d' "$sid") (${SESSION_SLUG_BY_ID[$sid]}) FAILED in $(format_elapsed "$elapsed") (exit $rc)"
      fi
    else
      new_pids+=("${JOB_PIDS[$i]}")
      new_sids+=("${JOB_SIDS[$i]}")
    fi
  done
  if [[ ${#new_pids[@]} -gt 0 ]]; then
    JOB_PIDS=("${new_pids[@]}")
    JOB_SIDS=("${new_sids[@]}")
  else
    JOB_PIDS=()
    JOB_SIDS=()
  fi
}

for (( wn=1; wn<=WAVE_COUNT; wn++ )); do
  ids="${WAVE_IDS[$wn]:-}"
  ids="$(echo "$ids" | xargs)"   # trim
  [[ -z "$ids" ]] && continue

  # Filter by --start/--end
  IFS=' ' read -ra ALL_IN_WAVE <<< "$ids"
  IN_RANGE=()
  for sid in "${ALL_IN_WAVE[@]}"; do
    if [[ "$sid" -lt "$START_FROM" ]]; then
      SESSION_STATUS[$sid]="skipped"
      mark_session "${STATUS_FILE:-}" "$sid" "skipped"
      continue
    fi
    if [[ "$sid" -gt "$END_AT" ]]; then
      SESSION_STATUS[$sid]="skipped"
      mark_session "${STATUS_FILE:-}" "$sid" "skipped"
      continue
    fi
    IN_RANGE+=("$sid")
  done

  if [[ ${#IN_RANGE[@]} -eq 0 ]]; then
    dim "  skip wave $wn (all sessions outside range)"
    continue
  fi

  # Wave banner (suppressed when live UI owns the display)
  if ! $LIVE_UI; then
    echo ""
    log "┌─────────────────────────────────────────────────────"
    if [[ ${#IN_RANGE[@]} -gt 1 ]]; then
      log "│ ${BOLD}Wave $wn / $WAVE_COUNT${NC}  (${MAGENTA}${#IN_RANGE[@]} sessions in parallel${NC})"
    else
      log "│ ${BOLD}Wave $wn / $WAVE_COUNT${NC}  (1 session, serial)"
    fi
    for sid in "${IN_RANGE[@]}"; do
      log "│   • session $(printf '%02d' "$sid") — ${SESSION_SLUG_BY_ID[$sid]}  (deps: ${SESSION_DEPS_BY_ID[$sid]})"
    done
    log "└─────────────────────────────────────────────────────"
  fi

  # Make sure trunk is checked out at TRUNK_WORKTREE_DIR (so siblings branch off it)
  if $USE_WORKTREE; then
    git -C "$TRUNK_WORKTREE_DIR" checkout -q "$BRANCH"
  fi

  # --- Spawn wave sessions ---
  declare -a JOB_PIDS=()
  declare -a JOB_SIDS=()

  for sid in "${IN_RANGE[@]}"; do
    # Throttle to MAX_PARALLEL
    while [[ ${#JOB_PIDS[@]} -ge $MAX_PARALLEL ]]; do
      reap_finished_jobs
      [[ ${#JOB_PIDS[@]} -ge $MAX_PARALLEL ]] && sleep 1
    done

    slug="${SESSION_SLUG_BY_ID[$sid]}"
    friendly="${slug//-/ }"
    # Use "--" not "/" as the trunk→session separator so per-session branch
    # names don't try to nest under the trunk's ref path. Git rejects creating
    # `epic/shipment-io/s01-charter` while `epic/shipment-io` already exists
    # as a leaf branch (ref-directory conflict).
    sess_branch="${BRANCH}--s$(printf '%02d' "$sid")-${slug}"
    sess_wt_dir_name="${BRANCH_SANITIZED}--s$(printf '%02d' "$sid")-${slug}"
    SESSION_BRANCH_BY_ID[$sid]="$sess_branch"

    if $USE_WORKTREE; then
      sess_wt="$WORKTREE_BASE/$sess_wt_dir_name"
      # Wipe stale worktree from a prior failed run
      if [[ -d "$sess_wt" ]]; then
        log "  ↻ removing stale worktree for session $sid: $sess_wt"
        git worktree remove "$sess_wt" --force 2>/dev/null || rm -rf "$sess_wt"
      fi
      if git show-ref --verify --quiet "refs/heads/$sess_branch"; then
        git branch -D "$sess_branch" 2>/dev/null || true
      fi
      git -C "$TRUNK_WORKTREE_DIR" worktree add -b "$sess_branch" "$sess_wt" "$BRANCH"
    else
      # No worktree → commit directly to trunk; no per-session branch.
      sess_wt="$TRUNK_WORKTREE_DIR"
      SESSION_BRANCH_BY_ID[$sid]="$BRANCH"
    fi
    SESSION_WT_BY_ID[$sid]="$sess_wt"

    handoff_text="$(build_handoff_section "${SESSION_DEPS_BY_ID[$sid]}")"

    quiet="false"
    [[ ${#IN_RANGE[@]} -gt 1 ]] && quiet="true"

    ! $LIVE_UI && log "  ▶ session $(printf '%02d' "$sid") (${slug}) starting → branch $sess_branch"

    SESSION_STATUS[$sid]="running"
    SESSION_START_TS[$sid]=$(date +%s)
    mark_session "${STATUS_FILE:-}" "$sid" "running"
    (
      run_one_session "$sid" "$sess_wt" "$friendly" "$handoff_text" "$quiet"
    ) &
    pid=$!
    JOB_PIDS+=("$pid")
    JOB_SIDS+=("$sid")
  done

  # Wait for all jobs in this wave
  while [[ ${#JOB_PIDS[@]} -gt 0 ]]; do
    reap_finished_jobs
    [[ ${#JOB_PIDS[@]} -gt 0 ]] && sleep 2
  done

  # --- Merge successful wave children into trunk ---
  if $USE_WORKTREE; then
    git -C "$TRUNK_WORKTREE_DIR" checkout -q "$BRANCH"
  fi

  any_failed=false
  for sid in "${IN_RANGE[@]}"; do
    case "${SESSION_STATUS[$sid]:-}" in
      done)
        slug="${SESSION_SLUG_BY_ID[$sid]}"
        if $USE_WORKTREE; then
          sess_branch="${SESSION_BRANCH_BY_ID[$sid]}"
          if git -C "$TRUNK_WORKTREE_DIR" merge --no-ff -q \
              -m "Merge session $(printf '%02d' "$sid") (${slug}) into ${BRANCH}" \
              "$sess_branch"
          then
            ! $LIVE_UI && ok "  ⇢ merged session $(printf '%02d' "$sid") into trunk"
            MERGED_SESSIONS+=("$sid ${SESSION_FILE_BASENAME[$sid]}")
          else
            err "  ⇢ MERGE CONFLICT merging session $(printf '%02d' "$sid") into trunk"
            err "      Trunk worktree: $TRUNK_WORKTREE_DIR"
            err "      Resolve, commit, and resume with --start $((sid + 1))"
            git -C "$TRUNK_WORKTREE_DIR" merge --abort 2>/dev/null || true
            EPIC_FAILED=true
            [[ -z "$FIRST_FAILED_ID" ]] && FIRST_FAILED_ID="$sid"
          fi
        else
          # Session committed directly to trunk; no merge required.
          MERGED_SESSIONS+=("$sid ${SESSION_FILE_BASENAME[$sid]}")
        fi
        ;;
      failed)
        any_failed=true
        EPIC_FAILED=true
        [[ -z "$FIRST_FAILED_ID" ]] && FIRST_FAILED_ID="$sid"
        ;;
    esac
  done

  if $EPIC_FAILED; then
    err "Wave $wn had failures — halting before subsequent waves."
    break
  fi

  # Remove successful per-session worktrees (keep on failure for inspection)
  if $USE_WORKTREE && ! $KEEP_SESSION_WORKTREES; then
    for sid in "${IN_RANGE[@]}"; do
      if [[ "${SESSION_STATUS[$sid]:-}" == "done" ]]; then
        sess_wt="${SESSION_WT_BY_ID[$sid]}"
        if [[ -d "$sess_wt" ]]; then
          git -C "$TRUNK_WORKTREE_DIR" worktree remove "$sess_wt" --force 2>/dev/null || true
        fi
      fi
    done
  fi
done

# ---------------------------------------------------------------------------
# Shut down live UI and print final summary
# ---------------------------------------------------------------------------
if [[ -n "$UI_PID" ]] && kill -0 "$UI_PID" 2>/dev/null; then
  sleep 0.5    # allow one last redraw to show final state
  kill "$UI_PID" 2>/dev/null || true
  wait "$UI_PID" 2>/dev/null || true
fi
echo ""
if ! $EPIC_FAILED; then
  write_epic_result "success"
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  ok "${BOLD}Epic completed successfully${NC}"
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  if [[ ${#MERGED_SESSIONS[@]} -gt 0 ]]; then
    ok ""
    ok "Sessions merged into ${BRANCH}:"
    for entry in "${MERGED_SESSIONS[@]}"; do
      ok "  $entry"
    done
  fi
  ok ""
  ok "Logs:  ${TRUNK_SESSIONS_DIR/#$HOME/~}/.session-*-{plan,exec}.log"
  ok "Plans: ${TRUNK_SESSIONS_DIR/#$HOME/~}/.session-*-plan.md"
  if [[ "$TIMEOUT" -gt 0 || "$RETRY" -gt 0 ]]; then
    ok ""
    ok "Runtime settings:"
    [[ "$TIMEOUT" -gt 0 ]] && ok "  Timeout: ${TIMEOUT}m per session"  
    [[ "$RETRY" -gt 0 ]] && ok "  Retry: up to $RETRY attempts per session"
  fi
  ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  # --- Cleanup orchestrator artifacts before PR ---
  # Only remove the orchestrator's internal working files (dotfiles like
  # `.session-NN-plan.md`, `.session-NN-{plan,exec}.log`). The session prompt
  # files (`session-NN-*.md`) and the handoff docs under
  # `docs/roadmap/<epic>/session-NN-handoff.md` are user-facing deliverables —
  # they document the work, contain the CI-gate verdict, and are needed for
  # epic resume / audit. Keeping them on the branch is the safe default.
  log "Cleaning up orchestrator artifacts..."
  CLEANED=0
  for f in "$TRUNK_SESSIONS_DIR"/.session-* \
            "${STATUS_FILE:-}" "${DAG_PLAN_FILE:-}" \
            "${STATUS_FILE:-}.lock" "${STATUS_FILE:-}.tmp"; do
    [[ -f "$f" ]] && rm -f "$f" && CLEANED=$((CLEANED + 1))
  done

  if [[ $CLEANED -gt 0 ]]; then
    cd "$REPO_ROOT"
    git add -A
    git commit -q -m "chore: clean up orchestrator artifacts

Remove $CLEANED internal working files (per-session plans + logs).

Co-Authored-By: AI <noreply@ai>" 2>/dev/null || true
    ok "Cleaned $CLEANED orchestrator artifacts (handoffs preserved)"
  fi

  # --- Auto-PR ---
  if $AUTO_PR && command -v gh &>/dev/null; then
    CURRENT_BRANCH="$(git -C "$REPO_ROOT" branch --show-current)"
    DEFAULT_BRANCH="$(gh repo view --json defaultBranchRef -q '.defaultBranchRef.name' 2>/dev/null || echo main)"
    if [[ "$CURRENT_BRANCH" != "$DEFAULT_BRANCH" ]]; then
      EXISTING_PR="$(gh pr view "$CURRENT_BRANCH" --json url -q '.url' 2>/dev/null || true)"
      if [[ -n "$EXISTING_PR" ]]; then
        ok "PR already exists: $EXISTING_PR"
      else
        log "Creating pull request..."
        if ! git -C "$REPO_ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{u}' &>/dev/null; then
          git -C "$REPO_ROOT" push -u origin "$CURRENT_BRANCH"
        elif [[ -n "$(git -C "$REPO_ROOT" log '@{u}..HEAD' --oneline 2>/dev/null)" ]]; then
          git -C "$REPO_ROOT" push
        fi

        EPIC_NAME="$(basename "$TRUNK_SESSIONS_DIR" | tr '-' ' ')"
        SESSION_LIST=""
        for entry in "${MERGED_SESSIONS[@]}"; do
          SESSION_LIST="${SESSION_LIST}
- ${entry}"
        done
        DIFF_STATS="$(git -C "$REPO_ROOT" diff --shortstat "${DEFAULT_BRANCH}...HEAD" 2>/dev/null || echo "")"

        PR_TITLE="feat: ${EPIC_NAME}"
        PR_BODY="## Summary

Automated DAG epic: **${EPIC_NAME}**

Executed ${WAVE_COUNT} wave(s) across the parallel scheduler using \`${MODEL}\`.

### Sessions${SESSION_LIST}

### Stats

${DIFF_STATS}

🤖 Generated with DAG epic runner"

        PR_URL="$(gh pr create --title "$PR_TITLE" --body "$PR_BODY" 2>&1)" || true
        if [[ "$PR_URL" == http* ]]; then
          ok "Pull request created: $PR_URL"
        else
          warn "Could not create PR: $PR_URL"
        fi
      fi
    else
      warn "On default branch ($DEFAULT_BRANCH) — skipping PR creation"
    fi
  elif $AUTO_PR; then
    warn "gh CLI not found — skipping PR creation"
  fi

  # --- Trunk worktree cleanup ---
  if $USE_WORKTREE && [[ -n "$TRUNK_WORKTREE_DIR" ]]; then
    if $KEEP_WORKTREE; then
      log "Keeping trunk worktree: $TRUNK_WORKTREE_DIR"
    else
      cd "$ORIG_REPO_ROOT"
      log "Removing trunk worktree (work is on remote branch)..."
      git worktree remove "$TRUNK_WORKTREE_DIR" --force 2>/dev/null || true
      ok "Trunk worktree cleaned. Branch '$BRANCH' remains for the PR."
    fi
  fi
else
  write_epic_result "failed"
  err "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  err "${BOLD}Epic stopped${NC}"
  err "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  if [[ -n "$FIRST_FAILED_ID" ]]; then
    err "First failure: session $(printf '%02d' "$FIRST_FAILED_ID")"
    err "Resume with: $(basename "$0") $ORIG_SESSIONS_DIR --start $FIRST_FAILED_ID --branch $BRANCH"
  fi
  if $USE_WORKTREE; then
    err "Trunk worktree preserved : $TRUNK_WORKTREE_DIR"
    err "Failed session worktrees : $WORKTREE_BASE/${BRANCH_SANITIZED}--s*-* (inspect, then re-run)"
  fi
  err "Detailed results: ${TRUNK_SESSIONS_DIR}/.epic-result.json"
  exit 1
fi
