# Dashboard Multi-Account Charter

## Problem Statement

Tuitbot is a single-user, local-first X automation tool. A single operator may manage multiple X accounts (personal brand, company, side project). The database schema and API routing already support per-account data isolation via `account_id` columns and an `X-Account-Id` header, but several critical runtime paths remain singleton — config loading, token/session files, automation loops, and WebSocket events. This charter documents what exists, what is missing, and the contracts for completing full multi-account support within the dashboard.

## Current State Inventory

### Complete

| Component | Key Files | Notes |
|-----------|-----------|-------|
| Account registry (DB) | `migrations/20260227000015_multi_account_foundation.sql` | `accounts` table with `config_overrides`, `token_path`, `status`; `account_roles` table with RBAC |
| Default account sentinel | `storage/accounts.rs:10` | `DEFAULT_ACCOUNT_ID = "00000000-..."`, seeded in migration, cannot be deleted |
| Account CRUD (storage) | `storage/accounts.rs` | `list_accounts`, `get_account`, `create_account`, `update_account`, `delete_account` (soft-archive), `account_exists` |
| Role management | `storage/accounts.rs:187-250` | `get_role`, `set_role`, `remove_role`, `list_roles`; default account grants admin to all actors |
| Account CRUD (API) | `routes/accounts.rs` | Full REST: `GET/POST /api/accounts`, `GET/PATCH/DELETE /api/accounts/{id}`, role endpoints, `sync-profile` |
| `AccountContext` extractor | `account.rs` | Axum `FromRequestParts` impl resolving `X-Account-Id` header; `Role` enum (Admin/Approver/Viewer); `require_mutate`/`require_approve` guards |
| `X-Account-Id` on all requests | `dashboard/src/lib/api/http.ts:49,88` | Both `request()` and `uploadFile()` attach the header |
| Account store + switcher | `dashboard/src/lib/stores/accounts.ts`, `AccountSwitcher.svelte` | `currentAccountId` persisted to localStorage; `switchAccount` updates store + HTTP client; switcher renders when >1 account |
| `account_id` column on 20+ tables | Migration lines 38-94 | All data tables have `account_id` with default sentinel; composite PKs on `rate_limits`, `cursors` |
| Indexes on `account_id` | Migration lines 97-109 | 13 indexes for high-query tables |
| `_for()` storage variants | `approval_queue/queries.rs`, `replies.rs`, `scheduled_content.rs` | Account-scoped query variants exist alongside legacy unscoped versions |
| Runtime map keyed by account_id | `state.rs:54` | `runtimes: Mutex<HashMap<String, Runtime>>` |
| Content generators map | `state.rs:56` | `content_generators: Mutex<HashMap<String, Arc<ContentGenerator>>>` |
| Token managers map | `state.rs:72` | `token_managers: Mutex<HashMap<String, Arc<TokenManager>>>` |
| Profile sync endpoint | `routes/accounts.rs:177-224` | Loads per-account token path, calls X API `/users/me`, updates account record |

### Singleton Seams (Blocking Full Multi-Account)

#### S1: Config Loading (CRITICAL)

- **Where:** `config/mod.rs:161-185` — `Config::load()` reads a single `config.toml`.
- **Problem:** No per-account config resolution. The `accounts.config_overrides` column (JSON text, defaults to `'{}'`) exists but is never read or merged.
- **Impact:** All accounts share the same persona, scoring weights, rate limits, operating mode, and schedule.

#### S2: Token & Scraper Session Files (CRITICAL)

- **Where:** `routes/scraper_session.rs:30,64` — hardcoded `state.data_dir.join("scraper_session.json")` and `data_dir.join("tokens.json")`.
- **Problem:** Only one set of credentials can exist on disk. The `accounts.token_path` column exists and `sync-profile` falls back to the default path (`routes/accounts.rs:192-193`), but scraper sessions and the general token-loading path are unscoped.
- **Impact:** Cannot authenticate a second X account.

#### S3: Automation Loops Ignore account_id (CRITICAL)

- **Where:** `approval_poster.rs:42` — calls `get_next_approved(&pool)` (unscoped); `approval_poster.rs:95` — `mark_posted` unscoped; `approval_poster.rs:104` — `log_action` unscoped.
- **Also:** `discovery_loop.rs`, `content_loop.rs`, `mentions_loop.rs`, `thread_loop.rs`, `target_loop.rs`, `analytics_loop.rs` — loop constructors do not accept `account_id`; all storage calls use the legacy unscoped functions.
- **Partial exception:** `target_loop.rs` passes `account_id` to some storage methods (target_accounts), but receives it from config, not from a loop-level parameter.
- **Impact:** A second account's runtime would read/write the default account's data.

#### S4: WebSocket Events Unscoped (HIGH)

- **Where:** `ws.rs:25-84` — `WsEvent` variants have no `account_id` field.
- **Also:** `handle_ws()` at `ws.rs:137` subscribes to a single global broadcast channel with no filtering.
- **Impact:** All dashboard clients see events from all accounts. When switching accounts, stale events from the prior account appear.

#### S5: Watchtower is Global (MEDIUM)

- **Where:** `main.rs:209-217` — single `WatchtowerLoop` spawned with the global `content_sources` config.
- **Problem:** Content sources in `config.toml` are instance-level. No mapping of sources to accounts.
- **Impact:** Ingested content nodes cannot be attributed to a specific account's persona.

#### S6: Settings UX is Instance-Level (MEDIUM)

- **Where:** `routes/settings.rs:286-351` — `get_settings` and `patch_settings` read/write the global `config.toml` directly.
- **Also:** `dashboard/src/routes/(app)/settings/+page.svelte` — the settings page has no account-scoped sections.
- **Impact:** Changing persona, keywords, or scoring for account B also changes account A.

#### S7: Content Generator Init (LOW)

- **Where:** `main.rs:160-163` — only default account's content generator is inserted at startup.
- **Fix:** The `content_generators` HashMap pattern is already correct. Just extend lazy-init (like token managers) to create per-account generators on first use.

#### S8: CLI Has No Account Awareness (LOW)

- **Where:** `crates/tuitbot-cli/` — the `run` command boots a single-account runtime.
- **Impact:** CLI remains a fallback; the primary UX is the dashboard. No change needed for MVP.

## Design Decisions

### D1: Account-Scoped vs Instance-Scoped Config

Settings that belong to the server infrastructure are instance-scoped (shared). Settings that define an account's behavior/identity are account-scoped (stored in `accounts.config_overrides` as JSON, merged at runtime).

**Instance-scoped (shared, in `config.toml`):**

| Section | Rationale |
|---------|-----------|
| `server` (host, port) | One server process |
| `storage` (db path, retention) | Shared SQLite file |
| `logging` | Shared process |
| `deployment_mode` | Infrastructure choice |
| `connectors` (OAuth app credentials) | One registered X app |
| `llm` (provider, API key, model) | Single LLM billing |
| `circuit_breaker` | Global X API protection |
| `mcp_policy` | Instance-wide MCP rules |
| `auth` | Server-level auth config |

**Account-scoped (per-account overrides in `config_overrides` JSON):**

| Section | Rationale |
|---------|-----------|
| `mode` (autopilot/composer) | Per-account autonomy level |
| `business` (profile, keywords, persona) | Distinct brand identity |
| `scoring` (weights, thresholds) | Per-persona relevance tuning |
| `limits` (rate limits) | Per-account API budget |
| `intervals` (loop timings) | Different cadences |
| `approval_mode` | Per-account review policy |
| `schedule` (active hours) | Different time zones / audiences |
| `x_api.client_id`, `x_api.provider_backend` | Per-account auth method |
| `targets` (monitored accounts) | Per-account engagement targets |
| `content_sources` (which sources) | Per-account content pipeline |

### D2: Effective Config Resolution

A new function in `tuitbot-core::config` (not the server crate, respecting the server-boundary rule):

```rust
pub fn effective_config(base: &Config, overrides_json: &str) -> Result<Config, ConfigError>
```

**Algorithm:**
1. Serialize `base` Config to a JSON `Value`.
2. Parse `overrides_json` as a JSON `Value`.
3. Deep-merge overrides into base (RFC 7396 merge-patch semantics).
4. Deserialize the merged `Value` back into `Config`.
5. Validate the result.

**Call sites:**
- Runtime bootstrap (`routes/runtime.rs`) — when starting loops for a non-default account.
- Settings read (`GET /api/settings`) — when `X-Account-Id` is present.
- Content generator creation — when initializing per-account generators.

**Validation:** `config_overrides` is validated against the Config schema before being saved via `PATCH /api/accounts/{id}`. Invalid overrides are rejected at write time.

### D3: Per-Account File Layout

```
~/.tuitbot/
  config.toml                          # Instance config (shared)
  tuitbot.db                           # Shared DB (all accounts, isolated by account_id)
  tokens.json                          # Default account (backward compat)
  scraper_session.json                 # Default account (backward compat)
  accounts/
    {uuid}/
      tokens.json                      # Per-account OAuth tokens
      scraper_session.json             # Per-account browser session
```

**Rules:**
- Default account (`00000000-...`) always uses root-level files. No migration needed.
- `create_account` API sets `token_path` to `accounts/{id}/tokens.json` and creates the directory.
- Scraper session endpoints accept an optional `account_id` parameter; when present, read/write from `accounts/{id}/scraper_session.json`.
- A helper function `account_data_dir(data_dir: &Path, account_id: &str) -> PathBuf` centralizes path resolution.

### D4: WebSocket Account Scoping

**Approach:** Add `account_id: String` field to every `WsEvent` variant. Server broadcast remains global (single `broadcast::channel`). Client-side filtering in the WebSocket store:

```typescript
socket.onmessage = (e) => {
    const event = JSON.parse(e.data);
    if (event.account_id && event.account_id !== get(currentAccountId)) return;
    // dispatch as before
};
```

**Rationale:** Server-side per-account channels add complexity (subscribe/unsubscribe on switch, multiple channels). Client-side filtering is simpler, and the event volume is low enough that broadcasting all events is not a performance concern.

### D5: Credential Linking Flow (In-Dashboard)

1. User clicks "Add Account" in AccountSwitcher.
2. Dashboard sends `POST /api/accounts` with a label.
3. Dashboard navigates to a credential entry UI (new component):
   - **OAuth flow:** Initiate PKCE flow via existing `POST /api/x/auth/start` (scoped to account).
   - **Scraper session import:** Paste `auth_token` + `ct0` (scoped to account).
4. On credential save, call `POST /api/accounts/{id}/sync-profile` to fetch X avatar/username.
5. Account appears in switcher with avatar and `@username`.

The AccountSwitcher needs an "Add Account" button regardless of account count (currently hidden when only 1 account).

### D6: Default Account Backward Compatibility

- Default account ID (`00000000-...`) always exists in DB, cannot be deleted or archived.
- Missing `X-Account-Id` header defaults to it (already implemented in `account.rs:103`).
- Existing `config.toml`, `tokens.json`, `scraper_session.json` map to default account — no file moves.
- All existing DB rows already have `account_id = '00000000-...'` from the migration defaults.
- Singleton storage functions (e.g., `get_next_approved`) continue to work for the default account. New `_for()` variants are used by per-account runtime code.

## UX Goals

1. **Zero-friction single account:** Users who never add a second account experience no change. The AccountSwitcher is hidden (or shows only "Add Account"), config.toml works as before.
2. **In-dashboard account linking:** Adding a second account is entirely within the dashboard — no CLI commands, no manual file editing.
3. **Seamless switching:** Account switching is instant. The sidebar, all data views, settings, and real-time events reflect the active account within one render cycle.
4. **Per-account settings:** Each account has its own persona, scoring, schedule, and operating mode, editable from the existing settings page with an account-scoped toggle.
5. **Isolated automation:** Each account runs its own set of automation loops with its own credentials, rate limits, and content pipeline.

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Data leakage across accounts | High | Every storage query must include `account_id` in WHERE clauses. Code review checklist item. |
| Config merge produces invalid Config | Medium | `effective_config` validates after merge; `config_overrides` validated on write. |
| Backward compatibility breakage | Medium | Default account uses root-level files; missing header defaults to sentinel; singleton functions preserved. |
| Automation loop regression for default account | Medium | Existing unscoped functions remain; per-account `_for()` variants are additive. Run full test suite after each session. |
| Token file race conditions | Low | Each account has its own file; `TokenManager` is keyed by account_id. |
| WebSocket event flood with many accounts | Low | Client-side filtering is O(1); event volume per account is low (<10/minute). |

## Implementation Slices

| Session | Focus | Primary Risk |
|---------|-------|-------------|
| 2 | Effective config resolution + per-account file layout | Config loading breaks |
| 3 | Automation loop scoping (all `*_loop.rs` + `approval_poster`) | Loops stop working for default account |
| 4 | WebSocket account scoping + UI event invalidation | Real-time updates break |
| 5 | Dashboard account management UX (add account, credential linking, per-account settings) | Settings save/load regression |
| 6 | Watchtower per-account + content sources attribution | Content ingestion breaks |
| 7 | Integration testing, edge cases, backward compatibility validation | — |
