# Session 01 Prompt: Profile Contract and Routing

## Use this as your Claude Code prompt

You are implementing Session 01 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Introduce explicit MCP profiles with server-side enforcement by profile-specific routing.
- New public profile names:
  - `full` (default)
  - `readonly`
  - `api-readonly`

Hard requirements:
- No policy-only approach. Enforcement must happen by not registering tools for a profile.
- Existing full behavior should remain equivalent under `full`.
- Keep codebase Rust-idiomatic and testable.

Scope for this session:
1. Update profile enum + parser + display in `crates/tuitbot-mcp/src/state.rs`.
2. Update CLI help text and defaults in:
   - `crates/tuitbot-cli/src/commands/mod.rs`
   - `crates/tuitbot-cli/src/commands/mcp.rs`
3. Update runtime dispatch in `crates/tuitbot-mcp/src/lib.rs` so profile mapping is explicit and clean.
4. Add unit tests for:
   - parsing canonical names
   - invalid profile errors listing valid names
5. Remove legacy profile names from help/docs examples if present in this code path.

Implementation notes:
- If needed, split server constructors into clear profile entry points now, even if tool curation comes in Session 02.

Validation commands:
- `cargo test -p tuitbot-mcp state`
- `cargo test -p tuitbot-cli`
- `cargo test --workspace`

Deliverables:
- Working code changes.
- Updated inline docs/comments where profile names changed.
- Brief summary of what changed and any follow-up needed for Session 02.
