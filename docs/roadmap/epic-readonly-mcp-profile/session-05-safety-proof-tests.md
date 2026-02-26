# Session 05 Prompt: Safety Proof Tests (No Mutations by Construction)

## Use this as your Claude Code prompt

You are implementing Session 05 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Prove read-only safety properties by construction and regression tests.

Safety properties to prove:
1. In `readonly`, mutation tools are not registered.
2. In `api-readonly`, mutation tools are not registered.
3. In both read-only profiles, approval/queue/mutation composite tools are not registered.
4. Tool discovery for read-only profiles cannot return mutation-capable tools.

Scope for this session:
1. Add/expand profile registration tests in `crates/tuitbot-mcp/src/tools/boundary_tests.rs` or a new test module.
2. Add manifest-driven tests asserting zero mutation tools in read-only profiles.
3. Add discoverability tests (if possible via server tools/list integration) to ensure non-registration is externally visible.
4. Add regression test for accidental mutation reintroduction (denylist assertion for known mutation tools).
5. Tighten count assertions from broad ranges to explicit expected counts once stabilized.

High-value test expectations:
- Explicitly assert absence of:
  - `x_post_tweet`, `x_reply_to_tweet`, `x_quote_tweet`, `x_delete_tweet`, `x_post_thread`
  - `x_like_tweet`, `x_unlike_tweet`, `x_follow_user`, `x_unfollow_user`, `x_retweet`, `x_unretweet`
  - `x_upload_media`
  - approval and queue tools (`approve_item`, `approve_all`, `propose_and_queue_replies`, etc.)

Validation commands:
- `cargo test -p tuitbot-mcp boundary_tests`
- `cargo test -p tuitbot-mcp`
- `cargo test --workspace`

Deliverables:
- Strong safety proof test suite.
- Clear failing messages that identify profile, tool, and violated guarantee.
