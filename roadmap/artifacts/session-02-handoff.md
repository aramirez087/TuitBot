# Session 02 — Handoff

## CI Status

All three validation commands pass:

```
cargo fmt --all && cargo fmt --all --check    ✓ CLEAN
RUSTFLAGS="-D warnings" cargo test --workspace ✓ 905 tests, 0 failures
cargo clippy --workspace -- -D warnings        ✓ CLEAN
```

## Files Added

| File | Lines | Purpose |
|------|-------|---------|
| `src/contract/mod.rs` | 10 | Contract layer re-exports |
| `src/contract/envelope.rs` | 237 | ToolResponse/ToolError/ToolMeta + tests (moved from tools/response.rs) |
| `src/contract/error.rs` | 120 | ProviderError enum + mapping + tests |
| `src/provider/mod.rs` | 32 | SocialReadProvider trait definition |
| `src/provider/x_api.rs` | 57 | XApiProvider adapter (XApiClient → SocialReadProvider) |
| `src/kernel/mod.rs` | 8 | Kernel layer module declarations |
| `src/kernel/read.rs` | 60 | Provider-agnostic get_tweet, get_user_by_username, search_tweets |
| `src/kernel/tests.rs` | 176 | MockProvider + ErrorProvider + 9 tests |

## Files Modified

| File | Change |
|------|--------|
| `src/tools/response.rs` | Replaced with 7-line re-export shim from contract::envelope |
| `src/tools/x_actions/read.rs` | 3 functions delegate to kernel; 4 functions unchanged |
| `src/lib.rs` | Added `pub mod contract; mod kernel; mod provider;` |

## Files Deleted

None.

## Definition of Done — Verification

| Criterion | Status |
|-----------|--------|
| Direct X read tools no longer depend on workflow-only services | **Done** — `get_tweet_by_id`, `get_user_by_username`, `search_tweets` route through kernel, which depends only on `SocialReadProvider` + contract envelope |
| Contract types are reusable by non-TuitBot consumers | **Done** — `contract` module is `pub`, depends only on `serde`/`serde_json` |
| Test suite includes provider-mock tests for refactored tools | **Done** — 9 kernel tests with MockProvider + ErrorProvider, zero DB/network |
| Handoff names exact modules moved and why | **This document** |

## What Session 03 Must Decouple Next

1. **Remaining read tools**: Migrate `get_user_mentions`, `get_user_tweets`, `get_home_timeline` to kernel. Requires adding methods to `SocialReadProvider` and handling `authenticated_user_id` at the kernel level (pass as parameter, not via AppState).

2. **Write provider trait**: Define `SocialWriteProvider` for mutations (`post_tweet`, `reply_to_tweet`, `like_tweet`, etc.). Must integrate with policy gating — either kernel wraps policy checks or the write provider trait includes policy awareness.

3. **`get_x_usage` decoupling**: This tool reads from DB (`x_api_usage` table), not from the X API. It belongs in the workflow layer, not the kernel. Clarify its home.

4. **Deprecate legacy error helpers**: Once all X tools route through kernel, remove `x_actions/mod.rs::error_response()`, `not_configured_response()`, `no_user_id_response()` in favor of `contract::error::provider_error_to_response()`.

5. **API profile documentation**: Formalize which `SocialReadProvider` methods each tool requires, enabling capability-based provider matching.

## Artifacts Produced

- `roadmap/artifacts/session-02-adr-decoupled-mcp.md`
- `roadmap/artifacts/session-02-boundary-map.md`
- `roadmap/artifacts/session-02-refactor-notes.md`
- `roadmap/artifacts/session-02-handoff.md` (this file)
