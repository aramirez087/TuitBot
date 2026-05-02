#!/usr/bin/env python3
"""epic-poll-progress.py — Polling-based progress display for CLIs without stream-json.

Tails a session log file (written by the AI CLI to stdout) and displays a
single-line progress indicator on stderr by counting tool-use markers.
Used as a fallback when the AI CLI does not support --output-format stream-json
(e.g. OpenCode).

Writes text output to --log and updates the shared status JSON (--status-file).

Usage:
    # Spawned by run-sessions.sh when CLI_CMD != claude
    opencode -p -m sonnet < prompt > session.log 2>&1 &
    python3 epic-poll-progress.py --log session.log --phase EXEC \
        --session-id 2 --status-file .epic-status.json &
"""

import argparse
import os
import re
import sys
import time

SPINNER = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
REFRESH = 0.5  # seconds between poll cycles

# Patterns that indicate the CLI performed a tool use.
# Match common tool markers emitted by OpenCode or other CLIs into the log.
TOOL_PATTERNS = [
    (r'(?:Reading|Read|Editing|Edit|Writing|Write|Created)\s+(?:file|files?)\s+["\']?([^\s"\']+)', 'FileOp'),
    (r'(?:Running|Executed?)\s+(?:command|bash):\s+["\']?([^\n"\']+)', 'Bash'),
    (r'(?:Searched|Searching|Grep|Glob)\s+["\']?([^\s"\']+)', 'Search'),
    (r'(?:Applied|Attempting)\s+(?:\d+\s+)?edit(?:s)?\s+(?:to\s+)?["\']?([^\s"\']+)', 'Edit'),
    (r'(?:Created|Wrote)\s+["\']?([^\s"\']+)', 'Write'),
    (r'(?:Running)\s+(.+)', 'Run'),
]


def update_status_file(path: str, session_id: int, update: dict) -> None:
    """Atomically merge update into sessions[session_id] in the shared status JSON."""
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
        import json
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


def format_elapsed(seconds: float) -> str:
    m, s = divmod(int(seconds), 60)
    return f"{m}m{s:02d}s"


def render_line(spinner_char: str, elapsed: float, step: int,
                tool: str, target: str, term_width: int) -> str:
    prefix = f"  {spinner_char} {format_elapsed(elapsed)} │ "
    if tool:
        middle = f"Step {step:<3} │ {tool:<8}→ "
        avail = term_width - len(prefix) - len(middle) - 1
        target_str = target[:max(10, avail)] if target else ""
        return prefix + middle + target_str
    return prefix + f"Step {step:<3} │ Thinking..."


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Polling progress display for CLI sessions without stream-json")
    parser.add_argument("--log", required=True, help="Path to append text output to")
    parser.add_argument("--phase", default="RUN", help="Phase label (PLAN/EXEC)")
    parser.add_argument("--session-id", type=int, default=0, help="Session ID for status file")
    parser.add_argument("--status-file", default="", help="Shared status JSON path")
    args = parser.parse_args()

    session_id = args.session_id
    status_file = args.status_file

    try:
        term_width = os.get_terminal_size(sys.stderr.fileno()).columns
    except (OSError, ValueError):
        term_width = 80

    start = time.time()
    step = 0
    current_tool = ""
    current_target = ""
    spinner_idx = 0
    last_size = 0
    last_status_ts = time.time()

    # Open log file for appending raw cli output
    log_file = open(args.log, "a", encoding="utf-8")

    # Wait for log file to exist and start polling.
    # Exit after IDLE_TIMEOUT seconds with no new log content (session is done).
    IDLE_TIMEOUT = 60
    idle_start = time.time()
    try:
        while True:
            time.sleep(REFRESH)

            elapsed = time.time() - start

            # Read new content from the log by tracking file position
            try:
                file_size = os.path.getsize(args.log)
            except OSError:
                file_size = 0

            if file_size > last_size:
                idle_start = time.time()  # reset idle timer on new content
                try:
                    with open(args.log, "r", encoding="utf-8", errors="replace") as f:
                        f.seek(last_size)
                        new_text = f.read()
                    last_size = file_size

                    # Scan new text for tool-use markers
                    for pattern, tool_name in TOOL_PATTERNS:
                        for match in re.finditer(pattern, new_text, re.IGNORECASE):
                            step += 1
                            current_tool = tool_name
                            current_target = match.group(1).strip()
                except Exception:
                    pass

            # Render progress
            c = SPINNER[spinner_idx % len(SPINNER)]
            spinner_idx += 1

            if current_tool:
                line_out = render_line(c, elapsed, step, current_tool, current_target, term_width)
            elif step > 0:
                line_out = render_line(c, elapsed, step, "", "", term_width)
            else:
                line_out = f"  {c} {format_elapsed(elapsed)} │ Starting..."

            sys.stderr.write(f"\r{line_out:<{term_width}}")
            sys.stderr.flush()

            # Periodic status file update
            now = time.time()
            if session_id > 0 and status_file and (now - last_status_ts) >= 1.5:
                update_status_file(status_file, session_id, {
                    'step': step, 'tool': current_tool,
                    'target': current_target, 'elapsed': elapsed,
                })
                last_status_ts = now

            # Idle timeout: exit if no new log content for IDLE_TIMEOUT seconds
            if now - idle_start >= IDLE_TIMEOUT:
                break

    except KeyboardInterrupt:
        pass
    finally:
        log_file.close()

    # Scan log for errors before writing final status
    error_type = None
    error_detail = ""
    try:
        with open(args.log, "r", encoding="utf-8", errors="replace") as lf:
            log_content = lf.read()
        import re
        error_matches = re.findall(r'^\[ERROR\] (.+)$', log_content, re.MULTILINE)
        if error_matches:
            error_type = "cli_error"
            error_detail = error_matches[0]
    except Exception:
        pass

    # Final summary
    elapsed = time.time() - start
    summary = f"  ✓ {format_elapsed(elapsed)} │ Done    │ {step} tool actions"
    sys.stderr.write(f"\r{summary:<{term_width}}\n")
    sys.stderr.flush()

    # Final status update
    if session_id > 0 and status_file:
        update_data = {
            'step': step, 'tool': '', 'target': '',
            'elapsed': elapsed, 'status': 'done',
        }
        if error_type:
            update_data['error_type'] = error_type
            update_data['error_detail'] = error_detail
        update_status_file(status_file, session_id, update_data)


if __name__ == "__main__":
    main()