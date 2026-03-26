# CLAUDE.md

Read `AGENTS.md` first. It is the canonical repository guide for coding agents in this repo and holds the shared project description, architecture, commands, and conventions.

This file stays intentionally small and only carries Claude-specific deltas:

- Keep shared instructions in `AGENTS.md`. Do not duplicate or fork them here.
- Before writing any frontend code, invoke the `frontend-design` skill for the session.
- For frontend changes, run `cd dashboard && npx vitest run` before handoff.
- If a user is using one of the `.claude/commands/` workflows, follow that command file in addition to `AGENTS.md`.
- When updating agent guidance, update `AGENTS.md` first and keep `CLAUDE.md` as the thin overlay.
