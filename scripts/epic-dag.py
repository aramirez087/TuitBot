#!/usr/bin/env python3
"""
epic-dag.py — Parse epic session files, build a DAG, compute parallel waves.

Each session-NN-*.md may declare YAML frontmatter:

    ---
    session: 03
    title: "Auth API"
    depends_on: [01]
    touches:
      - src/api/auth/**
      - supabase/migrations/*auth*
    parallel_safe: true
    ---

    # Session 03: Auth API
    ...

Sessions without frontmatter implicitly depend on session N-1 (linear chain),
preserving back-compat with pre-DAG epics.

Session 00 is the operator-rules file; it is excluded from the DAG and
prepended to every prompt at runtime.

Output formats:
  --bash    line-oriented, parsed by run-sessions.sh (default for the runner)
  --json    structured plan
  --show    human-readable ASCII rendering
  --validate  exit 0 if schedule is valid, else non-zero

Exit codes:
  0  success
  1  hard error (cycle, missing dep, malformed file)
  2  --strict and a touches-glob overlap was detected within a wave
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import sys
from typing import Any

SESSION_RE = re.compile(r"^session-(\d+)-(.+)\.md$")


def parse_frontmatter(text: str) -> tuple[dict[str, Any], str]:
    """Minimal YAML-frontmatter parser. Stdlib only."""
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end == -1:
        end = text.find("\n---", 4)
        if end == -1:
            return {}, text
        body_start = end + 4
    else:
        body_start = end + 5
    fm_text = text[4:end]
    body = text[body_start:]

    def _strip_inline_comment(line: str) -> str:
        """Remove YAML inline comments ( # ...) while preserving # inside quotes."""
        in_quote: str | None = None
        for i, ch in enumerate(line):
            if ch in ('"', "'") and (i == 0 or line[i - 1] != "\\"):
                if in_quote == ch:
                    in_quote = None
                elif in_quote is None:
                    in_quote = ch
            elif ch == "#" and in_quote is None and i > 0 and line[i - 1] == " ":
                return line[:i].rstrip()
        return line

    result: dict[str, Any] = {}
    current_key: str | None = None
    for raw in fm_text.splitlines():
        raw = _strip_inline_comment(raw)
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        if raw.startswith(("  - ", "- ", "    - ")):
            item = raw.lstrip().lstrip("-").strip().strip("'\"")
            if current_key is not None:
                result.setdefault(current_key, []).append(item)
            continue
        if ":" not in raw:
            continue
        k, _, v = raw.partition(":")
        k = k.strip()
        v = v.strip()
        current_key = None
        if not v:
            current_key = k
            result[k] = []
        elif v.startswith("[") and v.endswith("]"):
            inner = v[1:-1].strip()
            result[k] = (
                [x.strip().strip("'\"") for x in inner.split(",") if x.strip()]
                if inner
                else []
            )
        elif v.lower() in ("true", "false"):
            result[k] = v.lower() == "true"
        elif v.lstrip("-").isdigit():
            result[k] = int(v)
        else:
            result[k] = v.strip("'\"")
    return result, body


def load_sessions(sessions_dir: str) -> tuple[list[dict[str, Any]], str | None]:
    sessions: list[dict[str, Any]] = []
    operator: str | None = None
    for entry in sorted(os.listdir(sessions_dir)):
        m = SESSION_RE.match(entry)
        if not m:
            continue
        path = os.path.join(sessions_dir, entry)
        sid = int(m.group(1))
        slug = m.group(2)
        if sid == 0:
            operator = path
            continue
        with open(path) as f:
            text = f.read()
        fm, _ = parse_frontmatter(text)
        fm_session = fm.get("session")
        if fm_session is not None and int(fm_session) != sid:
            raise SystemExit(
                f"ERROR: {entry} declares session={fm_session} "
                f"but filename says {sid:02d}"
            )
        deps_raw = fm.get("depends_on", None)
        implicit = deps_raw is None
        if implicit:
            deps: list[int] = []
        else:
            try:
                deps = [int(d) for d in deps_raw]
            except (TypeError, ValueError) as exc:
                raise SystemExit(
                    f"ERROR: {entry} depends_on must be a list of session numbers; "
                    f"got {deps_raw!r}"
                ) from exc
        touches = fm.get("touches", []) or []
        if not isinstance(touches, list):
            raise SystemExit(f"ERROR: {entry} touches must be a list of globs")
        sessions.append(
            {
                "id": sid,
                "file": entry,
                "path": path,
                "slug": slug,
                "title": fm.get("title", slug.replace("-", " ").title()),
                "depends_on": deps,
                "implicit_dep": implicit,
                "touches": [str(t) for t in touches],
                "parallel_safe": bool(fm.get("parallel_safe", True)),
                "model": fm.get("model", ""),
                "cli": fm.get("cli", ""),
                "has_frontmatter": bool(fm),
            }
        )

    sessions.sort(key=lambda s: s["id"])

    # Implicit linear chain for sessions without frontmatter or without depends_on:
    # session N → [N-1] (the prior session in numeric order).
    for i, s in enumerate(sessions):
        if s["implicit_dep"]:
            s["depends_on"] = [] if i == 0 else [sessions[i - 1]["id"]]

    return sessions, operator


def build_dag(sessions: list[dict[str, Any]]) -> dict[int, dict[str, Any]]:
    by_id = {s["id"]: s for s in sessions}
    for s in sessions:
        for d in s["depends_on"]:
            if d not in by_id:
                raise SystemExit(
                    f"ERROR: session {s['id']:02d} depends on {d:02d}, "
                    f"which is not present"
                )
            if d == s["id"]:
                raise SystemExit(f"ERROR: session {s['id']:02d} cannot depend on itself")

    color = {s["id"]: 0 for s in sessions}  # 0 white, 1 gray, 2 black
    stack: list[tuple[int, int]] = []  # (node, dep-iter-index)

    def visit(start: int) -> None:
        stack.append((start, 0))
        color[start] = 1
        while stack:
            node, idx = stack[-1]
            deps = by_id[node]["depends_on"]
            if idx >= len(deps):
                color[node] = 2
                stack.pop()
                continue
            stack[-1] = (node, idx + 1)
            nxt = deps[idx]
            if color[nxt] == 1:
                path = " → ".join(f"{n:02d}" for n, _ in stack) + f" → {nxt:02d}"
                raise SystemExit(f"ERROR: cycle detected: {path}")
            if color[nxt] == 0:
                color[nxt] = 1
                stack.append((nxt, 0))

    for s in sessions:
        if color[s["id"]] == 0:
            visit(s["id"])
    return by_id


def compute_waves(sessions: list[dict[str, Any]]) -> list[list[dict[str, Any]]]:
    """
    Kahn-style level partitioning. A session that declares parallel_safe=False
    runs alone in its wave (after its deps finish, before its dependents start).
    """
    waves: list[list[dict[str, Any]]] = []
    completed: set[int] = set()
    remaining = {s["id"] for s in sessions}
    by_id = {s["id"]: s for s in sessions}

    while remaining:
        ready = sorted(
            (
                s
                for s in sessions
                if s["id"] in remaining and all(d in completed for d in s["depends_on"])
            ),
            key=lambda x: x["id"],
        )
        if not ready:
            blocked = ", ".join(f"{x:02d}" for x in sorted(remaining))
            raise SystemExit(
                f"ERROR: schedule deadlock — could not place sessions {blocked}"
            )

        # Partition: any parallel_safe=False session is solo and goes first
        # among ready (so dependents in later waves are not held up unnecessarily).
        non_parallel = [s for s in ready if not s["parallel_safe"]]
        parallel = [s for s in ready if s["parallel_safe"]]

        if non_parallel:
            wave = [non_parallel[0]]
        else:
            wave = parallel

        waves.append(wave)
        for s in wave:
            completed.add(s["id"])
            remaining.discard(s["id"])

    return waves


def globs_overlap(a_globs: list[str], b_globs: list[str]) -> bool:
    """
    Heuristic overlap: directory-prefix containment, fnmatch on the patterns
    themselves, or literal equality. Not exact (we don't enumerate the FS).
    """

    def _stem(g: str) -> str:
        return g.split("**")[0].rstrip("/").rstrip("*")

    for ag in a_globs:
        for bg in b_globs:
            if ag == bg:
                return True
            if fnmatch.fnmatch(ag, bg) or fnmatch.fnmatch(bg, ag):
                return True
            ap, bp = _stem(ag), _stem(bg)
            if ap and bp and (ap.startswith(bp) or bp.startswith(ap)):
                return True
    return False


def find_overlaps(wave: list[dict[str, Any]]) -> list[tuple[int, int]]:
    pairs: list[tuple[int, int]] = []
    for i in range(len(wave)):
        for j in range(i + 1, len(wave)):
            a, b = wave[i], wave[j]
            if a["touches"] and b["touches"] and globs_overlap(a["touches"], b["touches"]):
                pairs.append((a["id"], b["id"]))
    return pairs


def render_ascii(waves: list[list[dict[str, Any]]]) -> str:
    lines = []
    width = max(
        (len(s["slug"]) for w in waves for s in w),
        default=10,
    )
    for i, w in enumerate(waves, 1):
        cells = "  ".join(f"[{s['id']:02d} {s['slug']:<{width}}]" for s in w)
        marker = "║" if len(w) == 1 else "╠"
        lines.append(f"  {marker} Wave {i}: {cells}")
    return "\n".join(lines)


def emit_bash(plan: dict[str, Any]) -> None:
    """
    Emit bash-parseable plan. SESSION lines have been extended with model/cli columns
    for backward compatibility: older parsers can ignore the extra trailing columns.
    Format: SESSION <wave> <id> <file> <deps> <slug> <parallel> <model> <cli>
    """
    print(f"META operator_path={plan['operator_path']}")
    print(f"META operator_file={os.path.basename(plan['operator_path'])}")
    print(f"META wave_count={len(plan['waves'])}")
    print(f"META any_frontmatter={'true' if plan['any_frontmatter'] else 'false'}")
    for wi, w in enumerate(plan["waves"], 1):
        print(f"WAVE {wi} {len(w)}")
        for s in w:
            parents = ",".join(f"{p}" for p in s["depends_on"]) or "-"
            print(
                f"SESSION {wi} {s['id']} {s['file']} {parents} {s['slug']} "
                f"{'1' if s['parallel_safe'] else '0'} {s['model']} {s['cli']}"
            )


def main() -> None:
    # On Windows, Python's text-mode stdout translates "\n" to "\r\n", which
    # breaks downstream bash parsing. Force LF newlines and UTF-8 so the
    # shell wrapper sees clean output regardless of platform.
    try:
        sys.stdout.reconfigure(newline="\n", encoding="utf-8")
    except (AttributeError, ValueError):
        pass
    ap = argparse.ArgumentParser()
    ap.add_argument("sessions_dir")
    ap.add_argument("--bash", action="store_true", help="emit line-oriented format")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--show", action="store_true")
    ap.add_argument("--validate", action="store_true")
    ap.add_argument("--strict", action="store_true", help="fail on touches overlap")
    args = ap.parse_args()

    if not os.path.isdir(args.sessions_dir):
        print(f"ERROR: not a directory: {args.sessions_dir}", file=sys.stderr)
        sys.exit(1)

    sessions, operator = load_sessions(args.sessions_dir)
    if operator is None:
        print(
            f"ERROR: no session-00-*.md (operator rules) in {args.sessions_dir}",
            file=sys.stderr,
        )
        sys.exit(1)
    if not sessions:
        print(f"ERROR: no session-NN-*.md files (NN > 0) in {args.sessions_dir}", file=sys.stderr)
        sys.exit(1)

    build_dag(sessions)
    waves = compute_waves(sessions)

    overlaps: list[tuple[int, int, int]] = []
    for wi, w in enumerate(waves, 1):
        for a, b in find_overlaps(w):
            overlaps.append((wi, a, b))

    plan = {
        "operator_path": os.path.abspath(operator).replace("\\", "/"),
        "waves": [
            [
                {
                    "id": s["id"],
                    "file": s["file"],
                    "path": os.path.abspath(s["path"]).replace("\\", "/"),
                    "slug": s["slug"],
                    "title": s["title"],
                    "depends_on": s["depends_on"],
                    "touches": s["touches"],
                    "parallel_safe": s["parallel_safe"],
                    "model": s["model"],
                    "cli": s["cli"],
                }
                for s in w
            ]
            for w in waves
        ],
        "overlaps": [{"wave": wi, "a": a, "b": b} for (wi, a, b) in overlaps],
        "any_frontmatter": any(s["has_frontmatter"] for s in sessions),
    }

    if args.json:
        print(json.dumps(plan, indent=2))
    elif args.bash:
        emit_bash(plan)
    elif args.show or args.validate:
        print(f"Sessions: {len(sessions)} across {len(waves)} wave(s)")
        print(render_ascii(waves))
        if overlaps:
            print("\nTouches-overlap warnings:")
            for wi, a, b in overlaps:
                print(f"  wave {wi}: session {a:02d} ↔ session {b:02d}")
        if not plan["any_frontmatter"]:
            print(
                "\nNote: no session declared frontmatter. "
                "Falling back to linear chain (one session per wave)."
            )
    else:
        # Default = --show
        print(f"Sessions: {len(sessions)} across {len(waves)} wave(s)")
        print(render_ascii(waves))

    if args.strict and overlaps:
        sys.exit(2)


if __name__ == "__main__":
    main()
