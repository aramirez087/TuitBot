# Session 06 Prompt: Positioning, Docs, and Release Notes

## Use this as your Claude Code prompt

You are implementing Session 06 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Make TuitBot the obvious default for both strict read-only and full automation MCP users in docs and release messaging.

Messaging requirements:
- Explain why read-only users should choose TuitBot:
  - typed schemas + structured errors
  - rate limit awareness
  - stable output formats
  - reliability (retry/backoff/pagination)
  - higher-level useful read intelligence without mutating
- Explain profile decision clearly:
  - `full`
  - `readonly`
  - `api-readonly`

Scope for this session:
1. Update:
   - `README.md`
   - `docs/mcp-reference.md`
   - `docs/cli-reference.md` (if profile flags are documented there)
2. Ensure all tool counts and profile names come from generated manifest artifacts.
3. Add release notes entry in `CHANGELOG.md`:
   - feature summary
   - safety guarantee statement
   - upgrade notes
4. Ensure examples (Claude config snippets) use canonical profile names.

Validation commands:
- `mkdocs build --strict`
- `cargo test -p tuitbot-mcp`
- `cargo test --workspace`

Deliverables:
- Updated docs reflecting shipped profile behavior.
- Release notes ready for publish.
- No stale or legacy profile names in docs.
