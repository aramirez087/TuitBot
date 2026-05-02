#!/usr/bin/env python3
"""epic-progress.py — Live progress display for AI-CLI stream-json output.

Reads newline-delimited JSON from stdin and displays a single-line progress
indicator on stderr. Writes the AI's text output to a log file.

Supports two JSON formats (auto-detected):
  - Claude Code: --output-format stream-json  (content_block_start/delta/stop)
  - OpenCode:    --format json                 (step_start/tool_use/text/step_finish)

Usage:
    claude -p --output-format stream-json < prompt \\
      | python3 epic-progress.py --log session.log --phase PLAN

    opencode run --format json -m opencode/claude-sonnet-4 < prompt \\
      | python3 epic-progress.py --log session.log --phase PLAN
"""

import argparse
import json
import os
import queue
import re
import sys
import threading
import time


def update_status_file(path: str, session_id: int, update: dict) -> None:
    """Atomically merge `update` into sessions[session_id] in the shared status JSON."""
    if not path or not os.path.isfile(path):
        return
    lock = path + '.lock'
    acquired = False
    for _ in range(100):
        try:
            fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
            os.close(fd)
            acquired = True
            break
        except (FileExistsError, OSError):
            time.sleep(0.02)
    if not acquired:
        return
    try:
        with open(path, encoding='utf-8') as f:
            data = json.load(f)
        key = str(session_id)
        data.setdefault('sessions', {}).setdefault(key, {}).update(update)
        tmp = path + '.tmp'
        with open(tmp, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2)
        os.replace(tmp, path)
    except Exception:
        pass
    finally:
        try:
            os.unlink(lock)
        except Exception:
            pass

SPINNER = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
REFRESH = 0.2  # seconds between spinner frames


def parse_target(tool_name, buf):
    """Extract a human-readable target from accumulated input_json_delta (Claude)."""
    patterns = {
        "Read": r'"file_path"\s*:\s*"([^"]*)"',
        "Edit": r'"file_path"\s*:\s*"([^"]*)"',
        "Write": r'"file_path"\s*:\s*"([^"]*)"',
        "NotebookEdit": r'"notebook_path"\s*:\s*"([^"]*)"',
        "Glob": r'"pattern"\s*:\s*"([^"]*)"',
        "Grep": r'"pattern"\s*:\s*"([^"]*)"',
        "Bash": r'"command"\s*:\s*"([^"]*)"',
        "Task": r'"description"\s*:\s*"([^"]*)"',
        "WebFetch": r'"url"\s*:\s*"([^"]*)"',
        "WebSearch": r'"query"\s*:\s*"([^"]*)"',
    }
    pat = patterns.get(tool_name)
    if pat:
        m = re.search(pat, buf)
        if m:
            return m.group(1)
    return ""


def parse_opencode_target(tool_name, state_input):
    """Extract a human-readable target from opencode tool state.input dict."""
    if not isinstance(state_input, dict):
        return ""
    field_map = {
        "read": "filePath",
        "edit": "filePath",
        "write": "filePath",
        "glob": "pattern",
        "grep": "pattern",
        "bash": "command",
        "task": "description",
        "webfetch": "url",
        "websearch": "query",
    }
    field = field_map.get(tool_name, "")
    if field:
        val = state_input.get(field, "")
        if isinstance(val, str):
            return val
    return ""


OPENCODE_TOOL_ALIASES = {
    "read": "Read",
    "edit": "Edit",
    "write": "Write",
    "glob": "Glob",
    "grep": "Grep",
    "bash": "Bash",
    "task": "Task",
    "webfetch": "WebFetch",
    "websearch": "WebSearch",
}


def shorten(path, max_len):
    """Shorten a file path or string to fit max_len."""
    if len(path) <= max_len:
        return path
    # For file paths, show .../<last components>
    if "/" in path:
        parts = path.split("/")
        result = parts[-1]
        for p in reversed(parts[:-1]):
            candidate = p + "/" + result
            if len(candidate) + 4 > max_len:
                break
            result = candidate
        return ".../" + result if len(".../" + result) <= max_len else result[:max_len]
    return path[:max_len - 3] + "..."


def format_elapsed(seconds):
    """Format seconds as Xm XXs."""
    m, s = divmod(int(seconds), 60)
    return f"{m}m{s:02d}s"


def render_line(spinner_char, elapsed, step, tool, target, term_width):
    """Build the single-line progress string."""
    prefix = f"  {spinner_char} {format_elapsed(elapsed)} │ "
    if tool:
        middle = f"Step {step:<3} │ {tool:<8}→ "
        avail = term_width - len(prefix) - len(middle) - 1
        target_str = shorten(target, max(10, avail)) if target else ""
        return prefix + middle + target_str
    else:
        return prefix + f"Step {step:<3} │ Thinking..."


def main():
    parser = argparse.ArgumentParser(description="Live progress for Claude stream-json")
    parser.add_argument("--log", required=True, help="Path to write Claude text output")
    parser.add_argument("--phase", default="RUN", help="Phase label (for display)")
    parser.add_argument("--session-id", type=int, default=0, help="Session ID (for status file)")
    parser.add_argument("--status-file", default="", help="Shared status JSON path")
    args = parser.parse_args()

    session_id  = args.session_id
    status_file = args.status_file

    log_file = open(args.log, "a", encoding="utf-8")
    start = time.time()
    step = 0
    tool_calls = 0
    current_tool = ""
    current_target = ""
    input_buf = ""
    spinner_idx = 0
    activity = ""  # "tool" or "text" or ""
    last_status_ts = time.time()
    STATUS_INTERVAL = 1.5  # seconds between periodic status file writes

    try:
        term_width = os.get_terminal_size(sys.stderr.fileno()).columns
    except (OSError, ValueError):
        term_width = 80

    # Cross-platform non-blocking stdin: a reader thread pumps chunks into a
    # Queue; the main loop polls with a timeout so the spinner animates even
    # while no events arrive. Replaces select.select(), which only works on
    # POSIX sockets/pipes — it errors on Windows pipes.
    stdin_buffer = sys.stdin.buffer
    chunk_q: queue.Queue = queue.Queue()

    def _reader() -> None:
        while True:
            try:
                chunk = stdin_buffer.read1(65536)
            except (OSError, ValueError):
                chunk_q.put(b"")
                return
            chunk_q.put(chunk)
            if not chunk:
                return

    threading.Thread(target=_reader, daemon=True).start()
    line_buf = ""
    eof = False

    try:
        while True:
            try:
                chunk = chunk_q.get(timeout=REFRESH)
            except queue.Empty:
                chunk = None

            if chunk is not None:
                if not chunk:
                    eof = True
                else:
                    line_buf += chunk.decode("utf-8", errors="replace")

                while "\n" in line_buf:
                    line, line_buf = line_buf.split("\n", 1)
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        evt = json.loads(line)
                    except json.JSONDecodeError:
                        continue

                    evt_type = evt.get("type", "")

                    # ── Claude stream-json events ──
                    if evt_type == "content_block_start":
                        cb = evt.get("content_block", {})
                        if cb.get("type") == "tool_use":
                            step += 1
                            tool_calls += 1
                            current_tool = cb.get("name", "?")
                            current_target = ""
                            input_buf = ""
                            activity = "tool"
                            if session_id > 0 and status_file:
                                now = time.time()
                                update_status_file(status_file, session_id, {
                                    'step': step, 'tool': current_tool,
                                    'target': '', 'elapsed': now - start,
                                })
                                last_status_ts = now
                        elif cb.get("type") == "text":
                            if activity != "tool":
                                activity = "text"
                                current_tool = ""
                                current_target = ""

                    elif evt_type == "content_block_delta":
                        delta = evt.get("delta", {})
                        delta_type = delta.get("type", "")

                        if delta_type == "input_json_delta":
                            input_buf += delta.get("partial_json", "")
                            new_target = parse_target(current_tool, input_buf)
                            if new_target and new_target != current_target:
                                current_target = new_target
                                if session_id > 0 and status_file:
                                    now = time.time()
                                    if now - last_status_ts >= 0.5:
                                        update_status_file(status_file, session_id, {
                                            'target': current_target,
                                            'elapsed': now - start,
                                        })
                                        last_status_ts = now

                        elif delta_type == "text_delta":
                            text = delta.get("text", "")
                            log_file.write(text)
                            log_file.flush()
                            if activity != "tool":
                                activity = "text"

                    elif evt_type == "content_block_stop":
                        if activity == "tool":
                            activity = ""
                            input_buf = ""

                    elif evt_type == "message_stop":
                        pass

                    # ── OpenCode --format json events ──
                    elif evt_type == "step_start":
                        step += 1
                        activity = ""
                        current_tool = ""
                        current_target = ""

                    elif evt_type == "tool_use":
                        part = evt.get("part", {})
                        raw_tool = part.get("tool", "?")
                        current_tool = OPENCODE_TOOL_ALIASES.get(raw_tool, raw_tool)
                        tool_calls += 1
                        state = part.get("state", {})
                        state_input = state.get("input", {}) if isinstance(state, dict) else {}
                        current_target = parse_opencode_target(raw_tool, state_input)
                        activity = "tool"
                        if session_id > 0 and status_file:
                            now = time.time()
                            update_status_file(status_file, session_id, {
                                'step': step, 'tool': current_tool,
                                'target': current_target, 'elapsed': now - start,
                            })
                            last_status_ts = now

                    elif evt_type == "text":
                        part = evt.get("part", {})
                        text = part.get("text", "")
                        if text:
                            log_file.write(text)
                            log_file.flush()
                        activity = "text"

                    elif evt_type == "step_finish":
                        if activity == "tool":
                            activity = ""
                            current_tool = ""
                            current_target = ""

                    elif evt_type == "error":
                        err_data = evt.get("error", {})
                        err_msg = err_data.get("message", "") if isinstance(err_data, dict) else str(err_data)
                        if err_msg:
                            log_file.write(f"\n[ERROR] {err_msg}\n")
                            log_file.flush()
                            if session_id > 0 and status_file:
                                update_status_file(status_file, session_id, {
                                    'error_type': 'cli_error',
                                    'error_detail': err_msg,
                                })

            if eof and not line_buf.strip():
                break

            # Render progress
            elapsed = time.time() - start
            c = SPINNER[spinner_idx % len(SPINNER)]
            spinner_idx += 1

            # Periodic status file update (elapsed + current tool/target)
            now_abs = time.time()
            if session_id > 0 and status_file and (now_abs - last_status_ts) >= STATUS_INTERVAL:
                update_status_file(status_file, session_id, {
                    'elapsed': elapsed,
                    'step': step,
                    'tool': current_tool,
                    'target': current_target,
                })
                last_status_ts = now_abs

            if activity == "tool" and current_tool:
                line_out = render_line(c, elapsed, step, current_tool, current_target, term_width)
            elif activity == "text":
                prefix = f"  {c} {format_elapsed(elapsed)} │ "
                line_out = prefix + "Writing response..."
            elif step > 0:
                line_out = render_line(c, elapsed, step, current_tool, current_target, term_width)
            else:
                prefix = f"  {c} {format_elapsed(elapsed)} │ "
                line_out = prefix + "Starting..."

            # Overwrite line on stderr
            sys.stderr.write(f"\r{line_out:<{term_width}}")
            sys.stderr.flush()

    except KeyboardInterrupt:
        pass
    finally:
        log_file.close()

    # Final summary line
    elapsed = time.time() - start
    summary = f"  ✓ {format_elapsed(elapsed)} │ Done    │ {tool_calls} tool calls"
    sys.stderr.write(f"\r{summary:<{term_width}}\n")
    sys.stderr.flush()


if __name__ == "__main__":
    main()
