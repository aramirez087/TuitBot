# Session 02 Prompt: Read-Only Tool Curation and Enforcement

## Use this as your Claude Code prompt

You are implementing Session 02 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Implement strict server-side tool curation for:
  - `readonly` (minimal read-only surface)
  - `api-readonly` (richer read-only API surface)
- Ensure neither read-only profile registers mutation/approval/queue/media tools.

Target tool sets:

`readonly` (minimal):
- `x_search_tweets`
- `get_tweet_by_id` (or rename to `x_get_tweet_by_id`)
- `x_get_user_by_username`
- `x_get_user_by_id`
- `x_get_user_tweets`
- `x_get_user_mentions`
- `x_get_home_timeline` (optional: include if available and safe)
- `get_rate_limits` (optional if available without mutability exposure)
- `health_check` (only if it does not imply mutability exposure)

`api-readonly` (broader read-only):
- Include all safe read tools currently available in API profile.
- Exclude all write/engage/media tools.
- Exclude approval/queue/mutation composite tools.

`full`:
- Preserve current complete workflow surface.

Hard requirements:
- Read-only enforcement by registration only. No runtime reject of exposed mutation tools.
- Tool discoverability for read-only profiles must not show mutation tools.

Scope for this session:
1. Refactor server registration paths to make profile-specific tool router composition explicit.
2. Add or adjust server types/modules if needed (for clarity over cleverness).
3. Update profile-aware startup flow in `crates/tuitbot-mcp/src/lib.rs`.
4. Ensure command help and docs comments in code match reality.
5. Add focused tests that check profile tool-family composition (counts can be finalized in Session 03/04).

File focus:
- `crates/tuitbot-mcp/src/server/*.rs`
- `crates/tuitbot-mcp/src/lib.rs`
- `crates/tuitbot-mcp/src/state.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs` (or new profile registration tests)

Validation commands:
- `cargo test -p tuitbot-mcp boundary_tests`
- `cargo test -p tuitbot-mcp`

Deliverables:
- Profile-specific registration in place.
- Minimal `readonly` is auditable and clearly intentional.
- Short summary listing tools available per profile after this session.
