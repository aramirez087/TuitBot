# Session 03 Handoff

## What Was Done

Session 03 wired `provider_backend = "scraper"` into the product runtime so the CLI and server honor the selected backend without requiring X API credentials in scraper mode.

### tuitbot-core

1. **`error.rs`** — Added three new `XApiError` variants:
   - `ScraperMutationBlocked` — writes disabled via config
   - `ScraperTransportUnavailable` — transport not yet implemented
   - `FeatureRequiresAuth` — method requires authenticated user
   - Added three corresponding Display output tests

2. **`x_api/local_mode/mod.rs`** (new) — `LocalModeXClient` implementing full `XApiClient` trait (26 methods):
   - Auth-gated methods (get_me, get_mentions, get_home_timeline, bookmarks) → `FeatureRequiresAuth`
   - Read methods (search, get_tweet, get_user, etc.) → `ScraperTransportUnavailable` (stubs pending transport)
   - Write methods (post_tweet, reply, like, follow, etc.) → mutation gate check, then `ScraperTransportUnavailable`
   - Media upload → `MediaUploadError`
   - Helper methods: `check_mutation()`, `read_stub()`, `auth_required()`

3. **`x_api/local_mode/tests.rs`** (new) — 19 tests covering:
   - Factory function: returns `Some` for scraper, `None` for empty/x_api backends
   - Auth-gated: get_me, get_mentions, get_home_timeline, get_bookmarks return `FeatureRequiresAuth`
   - Mutation blocked: post_tweet, reply, like, follow, delete blocked when `allow_mutations=false`
   - Mutation enabled: post_tweet, reply return `ScraperTransportUnavailable` when enabled
   - Read stubs: search_tweets, get_tweet, get_user_by_username return `ScraperTransportUnavailable`
   - Media: upload_media returns `MediaUploadError`
   - Trait object: Send + Sync verification across tokio::spawn

4. **`x_api/mod.rs`** — Added `pub mod local_mode`, `pub use LocalModeXClient`, and `create_local_client()` factory function

5. **`automation/adapters/helpers.rs`** — Added explicit match arms for `ScraperMutationBlocked`, `ScraperTransportUnavailable`, `FeatureRequiresAuth` → `LoopError::Other` (previously handled by catch-all but now explicit for documentation)

### tuitbot-cli

6. **`deps.rs`** — Refactored `RuntimeDeps`:
   - `token_manager` and `x_client` changed from required to `Option<_>`
   - Added `dyn_client: Arc<dyn XApiClient>` field
   - `init()` now branches: `init_scraper_mode()` vs `init_official_mode()`
   - Extracted common adapter construction to `build_adapters()` (shared by both paths)
   - Scraper mode: no token loading, no tier detection, no `get_me()`, synthetic capabilities

7. **`commands/run.rs`** — Updated loop spawning:
   - Token refresh: only spawned when `token_manager.is_some()`
   - Approval poster: uses `deps.dyn_client` instead of casting `deps.x_client`
   - Removed unused `XApiClient` import

### tuitbot-server

8. **`state.rs`** — Added `provider_backend: String` to `AppState`

9. **`main.rs`** — Extracts `provider_backend` from config and passes to `AppState`

10. **`routes/runtime.rs`** — `GET /api/runtime/status` now includes `"provider_backend"` in JSON response

11. **Test files updated** — Added `provider_backend: String::new()` to all `AppState` constructions in:
    - `tests/api_tests.rs` (21 occurrences)
    - `tests/factory_reset.rs` (1 occurrence)
    - `tests/compose_contract_tests.rs` (1 occurrence)
    - `tests/fresh_install_auth.rs` (4 occurrences)

### Documentation

12. **`runtime-backend-plan.md`** — Runtime behavior matrix, error semantics, loop availability, server integration, backward compatibility, future integration points

## What Was Decided

| Decision | Outcome |
|----------|---------|
| `LocalModeXClient` implements `XApiClient` directly | Not the MCP `SocialReadProvider` — keeps product and MCP traits independent |
| Error variants on `XApiError` (not separate enum) | Minimizes changes to existing error mapping in adapters |
| Factory returns `Option` not `Result` | Official path requires tokens the factory can't provide — caller decides |
| `RuntimeDeps` fields made optional | `token_manager` and `x_client` are `Option<_>` rather than dummy stubs — cleaner, no phantom network calls |
| `build_adapters()` extracted | Eliminates ~100 lines of duplication between official and scraper init paths |
| Scraper capabilities: `mentions=false, discovery=true` | Mentions/target/analytics loops are skipped; discovery runs (will fail gracefully with transport stubs) |
| No changes to `approval_poster.rs` | Existing error logging is sufficient — scraper error messages are actionable |
| No changes to `workflow/discover.rs` or `workflow/publish.rs` | Trait dispatch is transparent — `LocalModeXClient` returns errors that the existing error paths handle |
| No changes to `automation/mod.rs` | Runtime struct is generic — no X API awareness needed there |
| Server `main.rs` minimal changes | Server doesn't create X API clients — only stores `provider_backend` for status endpoint |

## What Was NOT Done (Deferred)

| Topic | Deferred To | Notes |
|-------|-------------|-------|
| Actual scraper transport (HTTP/browser) | Session 04+ | Session 03 delivered the factory + stubs. Real I/O is separate. |
| Server runtime loop spawning | Future | `POST /api/runtime/start` creates empty `Runtime` — full loop setup from server is a separate epic |
| `config.example.toml` updates | Session 04 | Deferred per Session 02 decision |
| Integration tests with running server | Session 04 | Type-level round-trip tested. Full HTTP test deferred. |
| Dashboard runtime status display | Session 04+ | Frontend can use `provider_backend` from status endpoint |
| SeedWorker scraper awareness | Future | Lower priority |
| Circuit breaker for scraper transport | Session 04+ | Needed when real transport is implemented |
| Scraper mode startup banner | Session 04 | Could add "[SCRAPER MODE]" to banner |

## Changed Files in Session 03

```
crates/tuitbot-core/src/error.rs                        (modified — 3 variants + 3 tests)
crates/tuitbot-core/src/x_api/mod.rs                    (modified — module + factory fn)
crates/tuitbot-core/src/x_api/local_mode/mod.rs         (new — LocalModeXClient, ~315 lines)
crates/tuitbot-core/src/x_api/local_mode/tests.rs       (new — 19 tests)
crates/tuitbot-core/src/automation/adapters/helpers.rs   (modified — explicit error arms)
crates/tuitbot-cli/src/deps.rs                           (rewritten — scraper init path)
crates/tuitbot-cli/src/commands/run.rs                   (modified — optional token manager)
crates/tuitbot-server/src/state.rs                       (modified — provider_backend field)
crates/tuitbot-server/src/main.rs                        (modified — provider_backend extraction)
crates/tuitbot-server/src/routes/runtime.rs              (modified — provider_backend in status)
crates/tuitbot-server/tests/api_tests.rs                 (modified — 21 AppState updates)
crates/tuitbot-server/tests/factory_reset.rs             (modified — 1 AppState update)
crates/tuitbot-server/tests/compose_contract_tests.rs    (modified — 1 AppState update)
crates/tuitbot-server/tests/fresh_install_auth.rs        (modified — 4 AppState updates)
docs/roadmap/no-x-api-local-mode/runtime-backend-plan.md (new)
docs/roadmap/no-x-api-local-mode/session-03-handoff.md   (new)
```

## Quality Gate Results

- `cargo fmt --all && cargo fmt --all --check` — pass
- `RUSTFLAGS="-D warnings" cargo test --workspace` — pass (all tests green, including 19 new local_mode tests)
- `cargo clippy --workspace -- -D warnings` — pass (zero warnings)

## Session 04: Scraper Transport

### Mission

Implement actual scraper transport in `LocalModeXClient` read methods so discovery and profile lookup work without API credentials.

### Inputs from Session 03

- **Factory + client wired.** `create_local_client()` returns a `LocalModeXClient` that dispatches all `XApiClient` methods.
- **Error variants defined.** `ScraperTransportUnavailable` is returned by stubs — replace with real transport calls.
- **CLI init branches.** `RuntimeDeps::init_scraper_mode()` creates `LocalModeXClient` and skips OAuth.
- **Server exposes backend.** `GET /api/runtime/status` includes `provider_backend` — frontend can adapt.
- **Loops are gated.** Mentions/analytics/target loops skip in scraper mode. Discovery runs but search stub fails gracefully.

### Files to Modify

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/x_api/local_mode/mod.rs` | Replace `ScraperTransportUnavailable` stubs with real HTTP scraping for `search_tweets`, `get_tweet`, `get_user_by_username` |
| `crates/tuitbot-core/src/x_api/local_mode/transport.rs` | New. HTTP client for scraping public X endpoints |
| `crates/tuitbot-core/src/x_api/local_mode/tests.rs` | Add transport integration tests |

### Key Design Constraints

1. Read-only transport first (search, tweet lookup, user lookup)
2. Must handle X rate limiting and anti-bot measures gracefully
3. Circuit breaker needed for transport reliability
4. Error messages must distinguish "transport failed" from "not implemented"
5. No actual write transport until explicitly scoped
