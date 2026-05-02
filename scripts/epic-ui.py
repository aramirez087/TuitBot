#!/usr/bin/env python3
"""
epic-ui.py — Live terminal dashboard for epic-toolkit.

Watches a shared JSON status file and renders a continuously-updated display
of all sessions, their wave assignments, and live progress (tool, step, target).

TTY mode  : cursor-up redraws for smooth in-place updates
Non-TTY   : append-only state-change lines (safe in Claude Code / piped output)

Usage (launched by run-sessions.sh):
    python3 epic-ui.py --plan-bash DAG_TMP --status-file STATUS_JSON --epic NAME
"""

import argparse
import json
import os
import re
import signal
import sys
import time

REFRESH = 0.25  # seconds between polls / redraws

# ── ANSI escape codes ───────────────────────────────────────────────────────
RESET   = '\033[0m'
BOLD    = '\033[1m'
DIM     = '\033[2m'
BGREEN  = '\033[92m'
BCYAN   = '\033[96m'
BYELLOW = '\033[93m'
BRED    = '\033[91m'

SPINNER = '⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'

STATUS_ICONS = {
    'pending': '○',
    'running': '',      # replaced by spinner char at render time
    'done':    '✓',
    'failed':  '✗',
    'skipped': '─',
}
STATUS_COLORS = {
    'pending': DIM,
    'running': BCYAN,
    'done':    BGREEN,
    'failed':  BRED,
    'skipped': DIM,
}


# ── Utilities ────────────────────────────────────────────────────────────────

def strip_ansi(s: str) -> str:
    return re.sub(r'\033\[[0-9;]*[mABCDEFGHJKSTf]', '', s)


def visible_len(s: str) -> int:
    return len(strip_ansi(s))


def pad_right(s: str, width: int) -> str:
    return s + ' ' * max(0, width - visible_len(s))


def fmt_elapsed(secs: float) -> str:
    secs = max(0, int(secs))
    if secs < 60:
        return f'{secs:3d}s'
    m, s = divmod(secs, 60)
    if m < 60:
        return f'{m}m{s:02d}s'
    h, m = divmod(m, 60)
    return f'{h}h{m:02d}m'


def shorten(s: str, max_len: int) -> str:
    """Shorten a path to fit max_len, preferring the tail."""
    if not s or len(s) <= max_len:
        return s or ''
    s = s.replace('\\', '/')
    if '/' in s:
        parts = s.split('/')
        tail  = parts[-1]
        result = tail
        for part in reversed(parts[:-1]):
            candidate = part + '/' + result
            if len(candidate) + 2 > max_len:
                break
            result = candidate
        out = '…/' + result
        if len(out) > max_len:
            out = '…' + tail[-(max_len - 1):]
        return out
    return '…' + s[-(max_len - 1):]


# ── Plan / status I/O ────────────────────────────────────────────────────────

def parse_bash_plan(plan_file: str) -> dict:
    """Parse epic-dag.py --bash output into a structured plan dict."""
    waves: dict[int, list] = {}
    sessions: dict[int, dict] = {}
    wave_count = 0

    with open(plan_file, encoding='utf-8', errors='replace') as f:
        for line in f:
            line = line.strip()
            if line.startswith('META wave_count='):
                try:
                    wave_count = int(line.split('=', 1)[1])
                except ValueError:
                    pass
            elif line.startswith('WAVE '):
                parts = line.split()
                wn = int(parts[1])
                waves.setdefault(wn, [])
            elif line.startswith('SESSION '):
                parts = line.split(None, 6)
                if len(parts) < 6:
                    continue
                wn   = int(parts[1])
                sid  = int(parts[2])
                slug = parts[5]
                session = {
                    'id': sid, 'wave': wn,
                    'slug': slug,
                    'title': slug.replace('-', ' ').title(),
                }
                sessions[sid] = session
                waves.setdefault(wn, []).append(session)

    wave_list = [waves[wn] for wn in sorted(waves.keys())]
    return {
        'wave_list': wave_list,
        'sessions': sessions,
        'wave_count': wave_count or len(wave_list),
    }


def load_status(path: str) -> dict:
    """Load shared status JSON. Returns empty dict on any error."""
    try:
        with open(path, encoding='utf-8') as f:
            return json.load(f).get('sessions', {})
    except Exception:
        return {}


# ── Live UI renderer ─────────────────────────────────────────────────────────

class EpicUI:
    def __init__(self, plan: dict, status_file: str, epic_name: str):
        self.plan        = plan
        self.status_file = status_file
        self.epic_name   = epic_name
        self.spin_idx    = 0
        self.status: dict = {}
        self.is_tty      = sys.stderr.isatty()
        self._lines_drawn = 0
        self._seen_states: dict = {}   # sid:status → True (for append-mode dedup)
        self._running    = True

    # ── helpers ─────────────────────────────────────────────────────────────

    def _get(self, sid: int) -> dict:
        d = self.status.get(str(sid), {})
        return {
            'status':  d.get('status',  'pending'),
            'step':    d.get('step',    0),
            'tool':    d.get('tool',    ''),
            'target':  d.get('target',  ''),
            'elapsed': d.get('elapsed', 0.0),
        }

    def _all_done(self) -> bool:
        if not self.status:
            return False
        for wave in self.plan['wave_list']:
            for sess in wave:
                if self._get(sess['id'])['status'] not in ('done', 'failed', 'skipped'):
                    return False
        return True

    def _term_width(self) -> int:
        try:
            return os.get_terminal_size(sys.stderr.fileno()).columns
        except Exception:
            return 100

    # ── TTY render ──────────────────────────────────────────────────────────

    def _build_lines(self) -> list[str]:
        tw    = self._term_width()
        inner = tw - 2          # space inside left + right border chars
        spin  = SPINNER[self.spin_idx % len(SPINNER)]
        plan  = self.plan

        all_sess = [s for w in plan['wave_list'] for s in w]
        total    = len(all_sess)
        counts: dict[str, int] = {'done': 0, 'running': 0, 'failed': 0, 'pending': 0}
        for sess in all_sess:
            st = self._get(sess['id'])['status']
            counts[st] = counts.get(st, 0) + 1

        cur_wave = next(
            (s['wave'] for s in all_sess if self._get(s['id'])['status'] == 'running'),
            None,
        )

        lines: list[str] = []
        border = '═' * inner
        lines.append(f'{BOLD}╔{border}╗{RESET}')

        # Title row
        wave_note  = f'  {DIM}Wave {cur_wave} of {plan["wave_count"]}{RESET}' if cur_wave else ''
        title_left = f' {BOLD}\U0001f680 EPIC: {self.epic_name}{RESET}{wave_note}'

        parts = []
        if counts['done']:    parts.append(f'{BGREEN}✓ {counts["done"]}{RESET}')
        if counts['running']: parts.append(f'{BCYAN}⟳ {counts["running"]}{RESET}')
        if counts['failed']:  parts.append(f'{BRED}✗ {counts["failed"]}{RESET}')
        if counts['pending']: parts.append(f'{DIM}○ {counts["pending"]}{RESET}')
        parts.append(f'{DIM}/ {total}{RESET}')
        title_right = '  '.join(parts) + ' '

        pad = max(0, inner - visible_len(title_left) - visible_len(title_right))
        lines.append(
            f'{BOLD}║{RESET}{title_left}{" " * pad}{title_right}{BOLD}║{RESET}'
        )
        lines.append(f'{BOLD}╠{border}╣{RESET}')

        for wi, wave in enumerate(plan['wave_list']):
            wn = wi + 1

            # Wave header
            par = f'  {DIM}· {len(wave)} parallel{RESET}' if len(wave) > 1 else ''
            wh  = f'  {DIM}Wave {wn}{RESET}{par}'
            lines.append(
                f'{BOLD}║{RESET}{pad_right(wh, inner)}{BOLD}║{RESET}'
            )

            # Session rows
            for sess in wave:
                sid    = sess['id']
                slug   = sess['slug']
                st     = self._get(sid)
                status = st['status']
                color  = STATUS_COLORS.get(status, '')

                if status == 'running':
                    icon = f'{BCYAN}{spin}{RESET}'
                else:
                    icon_ch = STATUS_ICONS.get(status, '?')
                    icon    = f'{color}{icon_ch}{RESET}'

                name = f'{BOLD}{sid:02d}-{slug}{RESET}'

                # Build detail string (right side of row)
                if status == 'running':
                    elapsed = st['elapsed']
                    step    = st['step']
                    tool    = st['tool'] or ''
                    target  = st['target'] or ''

                    detail = f'{DIM}{fmt_elapsed(elapsed)}{RESET}  step {step:2d}'
                    if tool:
                        detail += f'  {BYELLOW}{tool:<10}{RESET}'
                    if target:
                        # How much room is left for the target?
                        static_part = f'    {" "} {sid:02d}-{slug}  {detail}'
                        avail = inner - visible_len(static_part) - 3
                        if avail > 6:
                            detail += f'  {DIM}{shorten(target, avail)}{RESET}'

                elif status == 'done':
                    elapsed = st['elapsed']
                    step    = st['step']
                    detail  = f'{DIM}{fmt_elapsed(elapsed)}'
                    if step:
                        detail += f'  {step} steps'
                    detail += RESET

                elif status == 'failed':
                    detail = f'{BRED}FAILED{RESET}'

                elif status == 'skipped':
                    detail = f'{DIM}skipped{RESET}'

                else:
                    detail = f'{DIM}pending…{RESET}'

                row = f'    {icon} {name}  {detail}'
                lines.append(
                    f'{BOLD}║{RESET}{pad_right(row, inner)}{BOLD}║{RESET}'
                )

            if wi < len(plan['wave_list']) - 1:
                lines.append(f'{BOLD}╠{border}╣{RESET}')

        lines.append(f'{BOLD}╚{border}╝{RESET}')
        return lines

    def _draw_initial(self) -> None:
        lines = self._build_lines()
        self._lines_drawn = len(lines)
        sys.stderr.write('\n')
        for line in lines:
            sys.stderr.write(line + '\n')
        sys.stderr.flush()

    def _redraw(self) -> None:
        new_lines  = self._build_lines()
        n_old      = self._lines_drawn
        n_new      = len(new_lines)

        # Move cursor up past old panel (+1 for the blank spacer line)
        sys.stderr.write(f'\033[{n_old + 1}A\033[G')
        sys.stderr.write('\n')
        for line in new_lines:
            sys.stderr.write(f'\033[2K{line}\n')
        for _ in range(max(0, n_old - n_new)):
            sys.stderr.write('\033[2K\n')

        sys.stderr.flush()
        self._lines_drawn = n_new

    # ── Non-TTY (append) render ──────────────────────────────────────────────

    def _print_changes(self) -> None:
        """Print one line per status transition (no cursor movement)."""
        for sid_str, st_data in self.status.items():
            status = st_data.get('status', 'pending')
            key    = f'{sid_str}:{status}'
            if key in self._seen_states:
                continue
            self._seen_states[key] = True

            try:
                sid  = int(sid_str)
            except ValueError:
                continue
            slug = self.plan['sessions'].get(sid, {}).get('slug', f'session-{sid:02d}')

            if status == 'running':
                sys.stderr.write(f'  ▶ {sid:02d}-{slug} running…\n')
            elif status == 'done':
                el   = fmt_elapsed(st_data.get('elapsed', 0))
                step = st_data.get('step', 0)
                sys.stderr.write(f'  ✓ {sid:02d}-{slug} done in {el} ({step} steps)\n')
            elif status == 'failed':
                sys.stderr.write(f'  ✗ {sid:02d}-{slug} FAILED\n')
            elif status == 'skipped':
                sys.stderr.write(f'  ― {sid:02d}-{slug} skipped\n')

        sys.stderr.flush()

    # ── Main loop ────────────────────────────────────────────────────────────

    def run(self) -> None:
        if self.is_tty:
            self._draw_initial()

        tick = 0
        while self._running:
            time.sleep(REFRESH)
            tick += 1
            self.spin_idx = tick
            self.status   = load_status(self.status_file)

            if self.is_tty:
                self._redraw()
            else:
                self._print_changes()

            if self._all_done():
                # One final refresh to show terminal state
                self.status = load_status(self.status_file)
                if self.is_tty:
                    self._redraw()
                else:
                    self._print_changes()
                break

        if self.is_tty:
            sys.stderr.write('\n')
            sys.stderr.flush()


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    ap = argparse.ArgumentParser(description='Epic live terminal dashboard')
    ap.add_argument('--plan-bash',    required=True, help='Path to epic-dag.py --bash output file')
    ap.add_argument('--status-file',  required=True, help='Path to shared session status JSON')
    ap.add_argument('--epic',         required=True, help='Epic name (display only)')
    args = ap.parse_args()

    def _on_signal(sig, frame):
        sys.exit(0)

    signal.signal(signal.SIGTERM, _on_signal)
    signal.signal(signal.SIGINT,  _on_signal)

    plan = parse_bash_plan(args.plan_bash)
    ui   = EpicUI(plan, args.status_file, args.epic)
    ui.status = load_status(args.status_file)
    ui.run()


if __name__ == '__main__':
    main()
