# Multi-Account QA Matrix

Comprehensive test scenario matrix covering all multi-account flows. Each row documents the expected behavior and verification method.

## 1. Default Account Backward Compatibility

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 1.1 | Fresh install, no accounts created | Default account (`00000000-...`) seeded in migration. All APIs work without `X-Account-Id` header. | Integration test: `init_without_claim_works_as_before` |
| 1.2 | Missing `X-Account-Id` header | `AccountContext` extractor defaults to `DEFAULT_ACCOUNT_ID`. Request proceeds normally. | `account.rs:103` fallback logic + integration tests |
| 1.3 | Existing `config.toml` | Loaded as base config. Default account uses it directly (no overrides). | `config/mod.rs` + `merge.rs` unit tests |
| 1.4 | Existing `tokens.json` at root | Default account resolves to root-level `tokens.json`. No file move required. | `account_data_dir` returns root for default; `token_path` helper tests |
| 1.5 | Existing `scraper_session.json` at root | Default account resolves to root-level `scraper_session.json`. | Scraper session endpoint tests with default account context |
| 1.6 | Existing DB rows (no account_id) | Migration sets `account_id = DEFAULT_ACCOUNT_ID` for all existing rows. | Migration `20260227000015` DEFAULT clauses |
| 1.7 | CLI operation (single account) | CLI continues to work without account awareness. Uses default account implicitly. | CLI does not send `X-Account-Id`; server defaults to sentinel |
| 1.8 | Single-account user never sees switcher | `AccountSwitcher` renders only when >1 account exists (plus "Add Account" option). | Frontend conditional rendering in `AccountSwitcher.svelte` |

## 2. Account Lifecycle

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 2.1 | Create account | `POST /api/accounts` creates DB row, `accounts/{id}/` directory, sets `token_path`. Returns new account. | Integration test: `create_account_with_label` |
| 2.2 | Rename account | `PATCH /api/accounts/{id}` with `label` updates display name. | Integration test: account update |
| 2.3 | Archive account | `DELETE /api/accounts/{id}` sets `status = 'archived'`. Account hidden from switcher but data preserved. | Integration test: soft delete |
| 2.4 | Cannot archive default account | `DELETE /api/accounts/{DEFAULT_ACCOUNT_ID}` returns 400. | Integration test: `default_account_delete_rejected` |
| 2.5 | Auto-switch on create | After creating a new account, dashboard switches to it and reloads all stores. | Frontend: `createAccount` in accounts store calls `switchAccount` |
| 2.6 | List accounts | `GET /api/accounts` returns all non-archived accounts. Default account always first. | Integration test: list accounts |
| 2.7 | Account with credentials shows profile | After `sync-profile`, account displays X avatar and `@username`. | Integration test: `sync_profile_updates_account` |

## 3. Credential Management

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 3.1 | OAuth PKCE link (new account) | `POST /api/x/auth/start` initiates PKCE flow scoped to account. Code paste completes auth. Token saved to `accounts/{id}/tokens.json`. | Integration test + `x_auth.rs` route tests |
| 3.2 | OAuth relink (existing tokens) | Same flow as 3.1 but overwrites existing token file. TokenManager cache evicted. | `x_auth.rs` handles overwrite |
| 3.3 | OAuth unlink | `DELETE /api/accounts/{id}/x-auth/tokens` removes token file, evicts TokenManager cache. Returns `{deleted: true}`. | Integration test: `x_auth_unlink_removes_tokens` |
| 3.4 | OAuth unlink (no tokens) | Returns `{deleted: false}`. Not an error. | Integration test: `x_auth_unlink_no_tokens_returns_false` |
| 3.5 | Cross-account OAuth isolation | Unlinking account A's tokens does not affect account B's tokens. | Integration test: `x_auth_unlink_cross_account_isolation` |
| 3.6 | Scraper session import | `POST /api/scraper-session` with `X-Account-Id` override saves `auth_token`/`ct0` to `accounts/{id}/scraper_session.json`. | Scraper session endpoint tests |
| 3.7 | Scraper session replace | Same as import; overwrites existing session file. | Endpoint handles overwrite |
| 3.8 | Scraper session remove | `DELETE /api/scraper-session` with `X-Account-Id` override removes session file. | Scraper session delete test |
| 3.9 | Cross-account scraper isolation | Importing session for account A does not affect account B's session. | Explicit `X-Account-Id` header override per-call |
| 3.10 | Default account uses root-level files | Default account's tokens at `data_dir/tokens.json`, scraper at `data_dir/scraper_session.json`. No subdirectory. | `account_data_dir` returns root for default; unit tests |

## 4. Account Switching

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 4.1 | Switch updates HTTP client header | `switchAccount()` updates the `X-Account-Id` header on all subsequent API calls. | `accounts.ts:switchAccount` sets header in `http.ts` |
| 4.2 | Switch invalidates page stores | All page-level stores refetch data for the new account on next access. | `ACCOUNT_SWITCHED_EVENT` listeners on 12+ pages |
| 4.3 | Switch flushes WebSocket events | `clearEvents()` called on switch. Old account's events removed from UI. | `websocket.ts` clears on `ACCOUNT_SWITCHED_EVENT` |
| 4.4 | Switch reloads approval stats | Sidebar pending count badge updates to reflect new account's approval queue. | Layout `onAccountSwitched` listener calls `loadApprovalStats()` |
| 4.5 | Switch reloads runtime capabilities | `reloadCapabilities()` fetches `can_post` status for new account. | `runtime.ts:reloadCapabilities` |
| 4.6 | Switch discards dirty drafts | Unsaved settings changes are discarded with a notification. No confirmation modal. | Settings page `ACCOUNT_SWITCHED_EVENT` listener resets draft |
| 4.7 | Switch persists to localStorage | Active account ID saved to localStorage. Survives page reload. | `accounts.ts` writes to localStorage on switch |
| 4.8 | Auto-refresh timers continue correctly | Timers keep running but fetch data with updated `X-Account-Id` header. No stale data. | HTTP client header already switched; timer fetches use current header |

## 5. Settings Isolation

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 5.1 | Default account sees full settings | All sections editable. No override badges. Changes write to `config.toml`. | Settings page with default account context |
| 5.2 | Non-default account sees scoped sections | Account-scoped sections (business, scoring, mode, limits, intervals, schedule, targets) are editable. Instance-scoped sections (server, storage, logging, connectors, LLM, circuit breaker, MCP) are locked. | Settings page scope badges + lockout logic |
| 5.3 | Override badge on non-default | Sections with overrides show "Account Override" badge. | `AccountScopeBadge` component |
| 5.4 | Save account override | `PATCH /api/settings` with non-default `X-Account-Id` writes to `accounts.config_overrides` column, not `config.toml`. | Settings API + merge logic tests |
| 5.5 | Reset to base | "Reset to Base" action removes account's override for that section. Section reverts to base config values. | Settings reset endpoint + frontend reset action |
| 5.6 | Config validation on save | Invalid overrides rejected at API boundary. Error displayed in UI. | `effective_config` validation + API error response |
| 5.7 | Effective config resolution | `effective_config(base, overrides)` deep-merges overrides into base config. | 16 unit tests in `config/merge.rs` |

## 6. Runtime Isolation

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 6.1 | Start runtime per-account | `POST /api/runtime/start` with `X-Account-Id` creates a `Runtime` entry keyed by account ID. | `state.rs:54` runtime map |
| 6.2 | Stop runtime per-account | `POST /api/runtime/stop` with `X-Account-Id` removes only that account's runtime. | Runtime stop handler |
| 6.3 | Runtime status per-account | `GET /api/runtime/status` returns status for the requesting account only. | AccountContext scoping |
| 6.4 | `can_post` per-account | Runtime capabilities check uses account-specific credentials. | `capabilities.rs` with account context |
| 6.5 | Content generator per-account | `get_or_create_content_generator()` lazily creates per-account generator using effective config. | `state.rs:56` content_generators map |

## 7. WebSocket Isolation

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 7.1 | Events carry account_id | All `WsEvent` variants include `account_id` field in serialized JSON. | WsEvent serialization tests |
| 7.2 | Client filters by active account | `websocket.ts` drops events where `account_id` does not match `currentAccountId`. | Frontend filtering logic |
| 7.3 | `clearEvents()` on switch | Event log cleared when switching accounts. No stale events from previous account. | `ACCOUNT_SWITCHED_EVENT` listener in websocket store |
| 7.4 | Events without account_id pass through | System-level events (if any) without `account_id` are not filtered. | Filter condition checks for presence of field |

## 8. Data Isolation

| # | Scenario | Expected Behavior | Verification |
|---|----------|-------------------|--------------|
| 8.1 | Activity page scoped | Activity feed shows only active account's data. | API call includes `X-Account-Id` |
| 8.2 | Approval page scoped | Approval queue shows only active account's pending items. | `_for()` storage variants |
| 8.3 | Content page scoped | Generated content scoped to active account. | AccountContext in content endpoints |
| 8.4 | Targets page scoped | Monitored accounts scoped per-account. | AccountContext in target endpoints |
| 8.5 | Costs page scoped | LLM usage/costs scoped to active account. | AccountContext in cost endpoints |
| 8.6 | Observability page scoped | Logs and metrics filtered by active account. | AccountContext in observability endpoints |
| 8.7 | Strategy page scoped | Strategy data scoped to active account. | AccountContext in strategy endpoints |
| 8.8 | Drafts page scoped | Draft posts scoped to active account. | AccountContext in draft endpoints |

## 9. Known Limitations (Accepted)

| # | Limitation | Risk Level | Rationale |
|---|-----------|------------|-----------|
| 9.1 | Automation loops create empty Runtime (no loops spawned) | Low | `_for()` storage variants exist. Loop wiring is future work. No data leakage since loops don't run. |
| 9.2 | Watchtower is instance-level | Low | Content sources are account-scoped in config but not attributed at ingestion time. Acceptable for initial release. |
| 9.3 | CLI is single-account | Low | Dashboard is the primary UX. CLI uses default account implicitly. |
| 9.4 | No pre-switch confirmation modal | Low | Auto-discard + notification is safe. Enhancement for future. |
| 9.5 | Config reload requires runtime restart | Low | Running runtimes are empty. Config changes apply on next start. |
| 9.6 | Field-level override indicators not implemented | Low | Section-level badges are sufficient and match backend merge granularity. |
| 9.7 | OAuth uses paste-code flow | Low | Works correctly. Auto-redirect is a future enhancement. |
| 9.8 | Scraper session not validated on import | Low | Invalid sessions fail at posting time with clear errors. |
