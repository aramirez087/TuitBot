# Epic: Best X MCP for Full and Read-Only Teams

## Milestone

`Read-only MCP profile shipped + CI doc sync`

## Why this exists

TuitBot already performs well as a growth workflow MCP. This epic closes the trust and adoption gap for teams that need a strict, minimal, read-only integration, while keeping full workflow power for automation users.

## Outcome

TuitBot becomes the default recommendation in both cases:

- `--profile full`: full workflow + mutation + approval features.
- `--profile readonly`: minimal, read-only by construction.
- `--profile api-readonly`: broader read-only API access, still no mutations.

## Non-negotiable constraints

- Read-only enforcement must be server-side by tool registration, not policy rejection.
- Read-only profiles must not expose mutation tools, approval tools, or queue/approve actions.
- Docs and shipped binary behavior must not drift.
- Safety must be provable via tests that verify mutation tools are not registered/discoverable.

## Task list (high priority)

1. Profile enforcement
- Add explicit profile model: `full`, `readonly`, `api-readonly`.
- Ensure runtime dispatch creates profile-specific server/tool routers.

2. Tool curation
- Define and enforce minimal `readonly` tool surface.
- Define and enforce richer `api-readonly` surface.
- Keep `full` as complete workflow profile.

3. CI validation and doc sync
- Emit machine-readable profile manifest including:
  - `tuitbot_version`
  - `mcp_schema_version`
  - `profile`
  - `tool_count`
  - tool list
- Generate docs tables from the manifest or fail CI when drift is detected.
- CI must verify profile tool lists/counts against committed docs artifacts.

4. Safety proofs + tests
- Add tests that assert mutation tools are absent in read-only profiles.
- Add tests that assert approval/queue tools are absent in read-only profiles.
- Add regression tests for profile parsing and error messages.

5. Docs and release notes
- Update README + `docs/mcp-reference.md` with new profile story and commands.
- Add release notes section: rationale, safety guarantees, and upgrade steps.

## Session sequence

Run these in order:

1. [session-01-profile-contract-and-routing.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-01-profile-contract-and-routing.md)
2. [session-02-readonly-tool-curation.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-02-readonly-tool-curation.md)
3. [session-03-manifest-and-discoverability-contract.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-03-manifest-and-discoverability-contract.md)
4. [session-04-ci-doc-sync-and-drift-guards.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-04-ci-doc-sync-and-drift-guards.md)
5. [session-05-safety-proof-tests.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-05-safety-proof-tests.md)
6. [session-06-positioning-docs-and-release-notes.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-06-positioning-docs-and-release-notes.md)
7. [session-07-final-hardening-and-ship-checklist.md](/Users/aramirez/Code/ReplyGuy/docs/roadmap/epic-readonly-mcp-profile/session-07-final-hardening-and-ship-checklist.md)

## Definition of done

- `tuitbot mcp serve --profile readonly` exposes a small, clear, read-only tool list.
- `tuitbot mcp serve --profile api-readonly` exposes richer read-only tools, still no mutation tools.
- `tuitbot mcp serve --profile full` preserves current full workflow behavior.
- Tool list/count in docs always matches executable behavior via CI gates.
- Tests prove no mutation capability is discoverable in read-only profiles.
