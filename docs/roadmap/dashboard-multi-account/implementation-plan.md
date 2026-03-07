# Multi-Account Implementation Plan

Session sequence for completing dashboard multi-account support. Each session targets a focused slice with explicit code targets and regression risks.

## Session 2: Effective Config & Per-Account File Layout

**Goal:** Enable per-account configuration resolution and credential file isolation.

**Code targets:**

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/config/mod.rs` | Add `effective_config(base: &Config, overrides_json: &str) -> Result<Config, ConfigError>` function |
| `crates/tuitbot-core/src/config/merge.rs` (new) | JSON merge-patch logic for Config values |
| `crates/tuitbot-core/src/storage/accounts.rs` | Add `account_data_dir(data_dir: &Path, account_id: &str) -> PathBuf` helper |
| `crates/tuitbot-server/src/routes/scraper_session.rs` | Accept `AccountContext`, resolve per-account session path via `account_data_dir` |
| `crates/tuitbot-server/src/routes/accounts.rs` | Set `token_path` on account creation; create `accounts/{id}/` directory |
| `crates/tuitbot-server/src/routes/settings.rs` | `get_settings`/`patch_settings` apply account overrides when `X-Account-Id` is non-default |

**Tests:**
- Unit test: `effective_config` merges overrides correctly, rejects invalid overrides.
- Unit test: `account_data_dir` returns root for default, subdirectory for others.
- Integration test: Scraper session endpoints respect account context.

**Regression risks:**
- Config loading breaks for existing single-account setups.
- Token path resolution breaks `sync-profile` for default account.

**Verification:** `cargo test --workspace`, manual test of `GET /api/settings` with and without `X-Account-Id`.

---

## Session 3: Automation Loop Scoping

**Goal:** Every automation loop accepts and threads `account_id` through all storage calls.

**Code targets:**

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/automation/approval_poster.rs` | Add `account_id: String` parameter; use `get_next_approved_for`, `mark_posted_for`, `log_action_for` |
| `crates/tuitbot-core/src/automation/discovery_loop.rs` | Thread `account_id` through `DiscoveryLoop` constructor and all storage calls |
| `crates/tuitbot-core/src/automation/content_loop.rs` | Same pattern |
| `crates/tuitbot-core/src/automation/mentions_loop.rs` | Same pattern |
| `crates/tuitbot-core/src/automation/thread_loop.rs` | Same pattern |
| `crates/tuitbot-core/src/automation/analytics_loop.rs` | Same pattern |
| `crates/tuitbot-core/src/automation/target_loop.rs` | Ensure `account_id` flows through all paths (partially done) |
| `crates/tuitbot-server/src/routes/runtime.rs` | Pass `account_id` from `AccountContext` to runtime bootstrap |

**Tests:**
- Existing loop tests must pass (adapter mocks unchanged).
- New tests: approval poster with scoped queries returns only matching account's items.

**Regression risks:**
- Approval poster stops processing items for default account if `_for()` variants filter incorrectly.
- Runtime start/stop breaks if account_id threading is incomplete.

**Verification:** `RUSTFLAGS="-D warnings" cargo test --workspace`, start runtime for default account via dashboard.

---

## Session 4: WebSocket Account Scoping

**Goal:** WebSocket events carry `account_id`; dashboard filters events for the active account.

**Code targets:**

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/ws.rs` | Add `account_id: String` to every `WsEvent` variant |
| All `event_tx.send()` call sites | Include `account_id` in event construction |
| `dashboard/src/lib/stores/websocket.ts` | Filter incoming events by `currentAccountId` |

**Tests:**
- Unit test: WsEvent serialization includes `account_id`.
- Frontend: verify filtered events don't leak across accounts.

**Regression risks:**
- Real-time updates stop working if `account_id` field breaks existing event parsing.
- Dashboard event handlers may rely on event shape — update all consumers.

**Verification:** `cargo test --workspace`, `cd dashboard && npm run check`, manual WebSocket test.

---

## Session 5: Dashboard Account Management UX

**Goal:** Full in-dashboard account creation, credential linking, switching, and per-account settings.

**Code targets:**

| File | Change |
|------|--------|
| `dashboard/src/lib/components/AccountSwitcher.svelte` | Add "Add Account" button; show switcher always (not only when >1 account); display avatars |
| `dashboard/src/lib/components/AddAccountFlow.svelte` (new) | Multi-step credential linking UI (label, OAuth or scraper session, profile sync) |
| `dashboard/src/routes/(app)/settings/+page.svelte` | Split into instance-scoped and account-scoped sections; account settings read/write `config_overrides` via `PATCH /api/accounts/{id}` |
| `dashboard/src/lib/stores/accounts.ts` | Add `createAccount`, `deleteAccount` actions |

**Tests:**
- `npm run check` passes.
- `npm run build` succeeds.

**Regression risks:**
- Settings save/load regression for single-account users.
- Svelte 5 rune usage errors in new components.

**Verification:** `cd dashboard && npm run build && npm run check`, manual test of add-account flow.

---

## Session 6: Watchtower Per-Account & Content Sources

**Goal:** Content sources can be attributed to specific accounts; Watchtower ingests content per-account.

**Code targets:**

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Accept `account_id` in `WatchtowerLoop::new()` or resolve from source config |
| `crates/tuitbot-core/src/config/types.rs` | Add optional `account_id` field to `ContentSourceEntry` |
| `crates/tuitbot-server/src/main.rs` | Spawn per-account watchtower loops (or single loop with account routing) |
| `crates/tuitbot-core/src/storage/watchtower/` | Ensure `content_nodes` queries filter by account_id |

**Tests:**
- Existing watchtower tests pass.
- New test: content nodes ingested with correct account_id.

**Regression risks:**
- Content ingestion breaks for default account.
- File watcher resource usage scales linearly with accounts.

**Verification:** `cargo test --workspace`, manual content source test.

---

## Session 7: Integration Testing & Polish

**Goal:** End-to-end validation, edge cases, backward compatibility, documentation.

**Code targets:**

| Focus | Details |
|-------|---------|
| Cross-account data isolation test | Create two accounts, run operations, verify no data leakage |
| Default account backward compatibility | Fresh install with no `X-Account-Id` header works identically to pre-multi-account |
| Account deletion cleanup | Archiving an account stops its runtime, cleans up event subscriptions |
| Factory reset with multiple accounts | Verify all account data is cleared |
| Documentation | Update `docs/architecture.md` with multi-account section |

**Regression risks:**
- None unique to this session (cumulative verification).

**Verification:** Full CI checklist: `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`, `RUSTFLAGS="-D warnings" cargo test --workspace`, `cd dashboard && npm run build && npm run check`.
