# Session 03 Prompt: Manifest Contract and Discoverability

## Use this as your Claude Code prompt

You are implementing Session 03 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Make MCP tool/profile metadata machine-verifiable and release-grade.
- Add manifest fields needed to prevent docs/binary drift.

Required manifest contract:
- `tuitbot_version`
- `mcp_schema_version`
- `profile`
- `tool_count`
- `tools` (stable order)

Hard requirements:
- Manifest reflects actual registered tool surface for each profile.
- Any profile/tool mismatch should fail tests.
- Keep manifest generation deterministic and stable in CI.

Scope for this session:
1. Extend manifest types in `crates/tuitbot-mcp/src/tools/manifest.rs`.
2. Add profile-specific manifest generation (`full`, `readonly`, `api-readonly`).
3. Add tests to assert:
   - manifest `tool_count == tools.len()`
   - read-only profiles contain no mutation tools
   - profile field matches requested profile
   - stable ordering for diff-friendly output
4. Add a CLI-accessible way to emit manifest JSON for CI/docs (choose one):
   - `tuitbot mcp manifest --profile <name>`
   - or `tuitbot mcp serve --profile <name> --emit-manifest`
5. Keep CLI behavior simple and explicit; canonical profile names only.

Design guidance:
- Prefer one source of truth for tool membership and metadata.
- Avoid duplicating tool names across router and manifest without an assertion tying them together.

Validation commands:
- `cargo test -p tuitbot-mcp manifest`
- `cargo test -p tuitbot-mcp boundary_tests`
- Run the new CLI manifest command for each profile and verify JSON shape.

Deliverables:
- Profile-aware manifest contract implemented.
- Tests proving contract integrity.
- Example output snippets saved or referenced for Session 04 docs generation.
