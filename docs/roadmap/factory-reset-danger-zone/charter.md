# Factory Reset -- Danger Zone: Charter

## Problem Statement

Users need an explicit, safe way to erase all Tuitbot-managed data and return
to the onboarding wizard without stopping the server or manually deleting
files.  The operation must be clearly destructive (typed confirmation phrase),
authenticated, and leave the server process and SQLite schema intact so the
user can immediately re-onboard.

---

## Current System Summary

### Auth Modes

**Bearer (Tauri/desktop):** `+layout.svelte` tries `invoke("get_api_token")`
from Tauri, falls back to `__DEV_API_TOKEN__`.  Sets bearer mode, checks
`GET /api/settings/status` -- if `configured=false`, redirects to `/onboarding`.

**Cookie (web/LAN):** If no bearer token, checks `/api/settings/status` first:
- Not configured --> redirect to `/onboarding` (with `?claimed=1` if passphrase
  hash exists).
- Configured --> checks session via `GET /api/auth/status`.
  - Valid session --> connect WebSocket, redirect away from `/login`.
  - No session --> redirect to `/login`.
- Login: `POST /api/auth/login` with passphrase --> session row in `sessions`
  table, sets `tuitbot_session` cookie (HttpOnly, SameSite=Strict, 7-day TTL).
- CSRF: mutating requests require `X-CSRF-Token` header matching session's
  token.

**Auth-exempt routes:** `/api/health`, `/api/health/detailed`,
`/api/settings/status`, `/api/settings/init`, `/api/ws`, `/api/auth/login`,
`/api/auth/status`.

### Onboarding Flow

Multi-step wizard: Welcome > X API > Business > LLM > Language > Sources >
Validate > Review > [Secure/Claim].

On submit: `POST /api/settings/init` with config JSON + optional `claim`
object.  `init_settings()` creates `config.toml`, optionally creates
`passphrase_hash` and session.  After success, frontend redirects to
`/content?compose=true`.

### Storage Layer

**Database:** SQLite, WAL mode, pool of 4 connections.  Default path:
`~/.tuitbot/tuitbot.db`.  Schema managed by 20 embedded SQLx migrations.

**30 user tables** (verified from all 20 migration files + `init_test_db`):

```
accounts                 account_roles            action_log
approval_edit_history    approval_queue           author_interactions
content_nodes            content_scores           cursors
discovered_tweets        draft_seeds              follower_snapshots
llm_usage                mcp_telemetry            media_uploads
mutation_audit           original_tweets          rate_limits
replies_sent             reply_performance        scheduled_content
sessions                 source_contexts          strategy_reports
target_accounts          target_tweets            thread_tweets
threads                  tweet_performance        x_api_usage
```

Plus `_sqlx_migrations` (infrastructure, never cleared).

**Foreign key constraints** (7 total, verified from migration SQL):

| Child                                   | Parent                        | On Delete |
|-----------------------------------------|-------------------------------|-----------|
| `thread_tweets.thread_id`               | `threads.id`                  | CASCADE   |
| `account_roles.account_id`              | `accounts.id`                 | CASCADE   |
| `target_tweets.account_id`              | `target_accounts.account_id`  | (none)    |
| `approval_edit_history.approval_id`     | `approval_queue.id`           | (none)    |
| `content_nodes.source_id`               | `source_contexts.id`          | (none)    |
| `draft_seeds.node_id`                   | `content_nodes.id`            | (none)    |
| `original_tweets.source_node_id`        | `content_nodes.id`            | (none)    |

Note: `reply_performance` and `tweet_performance` have no FK constraints in the
migrations despite logical parent relationships.

**File artifacts in `data_dir` (`~/.tuitbot/`):**

| File / Dir           | Purpose                                   |
|----------------------|-------------------------------------------|
| `config.toml`        | User configuration (TOML)                 |
| `passphrase_hash`    | bcrypt hash of web login passphrase (0600) |
| `api_token`          | Bearer token for Tauri/API auth (0600)    |
| `media/`             | Uploaded media files (`{uuid}.{ext}`)     |
| `tuitbot.db` + WAL   | SQLite database files                     |
| `backups/`           | Pre-migration database backups            |

### In-Memory State (`AppState` in `server/state.rs`)

| Field                | Type                                          |
|----------------------|-----------------------------------------------|
| `passphrase_hash`    | `RwLock<Option<String>>`                      |
| `runtimes`           | `Mutex<HashMap<String, Runtime>>`             |
| `content_generators` | `Mutex<HashMap<String, Arc<ContentGenerator>>>` |
| `login_attempts`     | `Mutex<HashMap<IpAddr, (u32, Instant)>>`      |
| `watchtower_cancel`  | `Option<CancellationToken>`                   |
| `circuit_breaker`    | `Option<Arc<CircuitBreaker>>`                 |

### Settings Page Layout

Settings page (`+page.svelte`) renders sections in order: Business, Persona,
Scoring, Limits, Schedule, LLM, X API, Storage, Sources, LAN.
`LanAccessSection.svelte` is the last section -- the natural neighbor for a
Danger Zone section.

---

## Reset Scope

### Cleared (Tuitbot-managed data)

| Category             | What                                  | How                                       |
|----------------------|---------------------------------------|-------------------------------------------|
| Config file          | `config.toml`                         | `fs::remove_file()`                       |
| Passphrase           | `passphrase_hash` file                | `fs::remove_file()` + clear in-memory     |
| All sessions         | `sessions` table rows                 | `DELETE FROM sessions` in transaction     |
| All DB table contents| All 30 user tables                    | `DELETE FROM <table>` (not DROP)          |
| Media files          | `media/` directory                    | `fs::remove_dir_all(data_dir/media)`      |
| Runtimes             | All automation loops                  | `Runtime::shutdown()` for each            |
| Content generators   | LLM generator cache                   | `content_generators.lock().clear()`       |
| Login rate limits    | IP-based tracking                     | `login_attempts.lock().clear()`           |
| Watchtower           | File watcher                          | Cancel via `watchtower_cancel` token      |

### Preserved (infrastructure)

| What                             | Why                                           |
|----------------------------------|-----------------------------------------------|
| SQLite schema (tables, indexes)  | Migrations track applied state; empty tables are the designed init path |
| `_sqlx_migrations` table         | Must not be cleared                           |
| `api_token` file                 | Bearer auth must survive so Tauri can reach the server |
| `AppState.api_token` in memory   | Same bearer token remains valid               |
| Database files (`.db`, WAL, SHM) | Pool stays open; only row contents cleared    |
| Server process                   | The whole point of "live reset"               |
| `backups/` directory             | Pre-migration backups are safety artifacts    |
| `circuit_breaker`                | Stateless rate-limit protector, no user data  |

### Explicitly Excluded

| What                                        | Why                                           |
|---------------------------------------------|-----------------------------------------------|
| User content source folders (`local_fs` paths in config) | User-authored files outside app data dir. Never touched. |
| Any files outside `data_dir`                | Reset scope is strictly `~/.tuitbot/` minus `api_token` and `backups/` |

---

## Endpoint Contract

### Request

```
POST /api/settings/factory-reset
```

**Authentication:** Required.  Bearer token OR session cookie + CSRF.
NOT auth-exempt -- this is a destructive action behind existing auth.

**Request body:**

```json
{
  "confirmation": "RESET TUITBOT"
}
```

The typed confirmation phrase is `RESET TUITBOT` (uppercase ASCII, exact match,
case-sensitive).  The server validates this string exactly before proceeding.

### Response (200 OK)

```json
{
  "status": "reset_complete",
  "cleared": {
    "tables_cleared": 30,
    "rows_deleted": 1542,
    "config_deleted": true,
    "passphrase_deleted": true,
    "media_deleted": true,
    "runtimes_stopped": 1
  }
}
```

**Response headers (cookie-auth only):**

```
Set-Cookie: tuitbot_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0
```

Clears the session cookie since all sessions are deleted from the DB.

### Error Responses

| Status | Condition                                    |
|--------|----------------------------------------------|
| 400    | Confirmation phrase missing or incorrect      |
| 401    | Not authenticated                             |
| 403    | Missing CSRF token (cookie auth)              |
| 500    | Partial failure during reset (body includes partial stats) |

### Post-Reset Behavior

After reset, `GET /api/settings/status` returns
`{ "configured": false, "claimed": false }`.  The frontend `+layout.svelte`
boot logic detects this and redirects to `/onboarding`.  No special redirect
logic needed -- the existing unconfigured-instance flow handles it.

---

## Execution Order

Order matters to avoid partial states and race conditions:

1. **Stop all runtimes** -- Lock `state.runtimes`, call `Runtime::shutdown()`
   for each, clear the map.  Cancel `watchtower_cancel` if present.

2. **Clear all DB table contents** -- Single transaction with
   `DELETE FROM <table>` for each of 30 tables (except `_sqlx_migrations`).
   FK-safe order: children before parents.

3. **Delete config.toml** -- `fs::remove_file(config_path)`.  Tolerate
   `NotFound` (idempotent).

4. **Delete passphrase_hash** -- `fs::remove_file(data_dir/passphrase_hash)`.
   Tolerate `NotFound`.

5. **Delete media directory** -- `fs::remove_dir_all(data_dir/media)`.
   Tolerate `NotFound`.

6. **Clear in-memory state** -- `passphrase_hash` RwLock -> `None`, clear
   `content_generators`, clear `login_attempts`.

7. ~~**VACUUM**~~ -- Dropped in Session 2 (see `session-02-handoff.md`).
   SQLite reclaims space on its own over time; VACUUM on a WAL-mode DB
   can block concurrent readers and is unnecessary for correctness.

8. **Return response** -- Include cleared stats, set cookie-clearing header.

Steps 2-6 log partial failures but continue.  The response indicates
what succeeded and what failed.

---

## FK-Safe Table Deletion Order

Children first, then parents.  Verified from all 20 migration SQL files:

```
 1. draft_seeds              (FK -> content_nodes.id)
 2. original_tweets          (FK -> content_nodes.id via source_node_id)
 3. content_nodes            (FK -> source_contexts.id)
 4. thread_tweets            (FK -> threads.id, ON DELETE CASCADE)
 5. account_roles            (FK -> accounts.id, ON DELETE CASCADE)
 6. target_tweets            (FK -> target_accounts.account_id)
 7. approval_edit_history    (FK -> approval_queue.id)
--- remaining tables have no FK constraints; order is arbitrary ---
 8. reply_performance
 9. tweet_performance
10. replies_sent
11. discovered_tweets
12. threads
13. approval_queue
14. scheduled_content
15. target_accounts
16. follower_snapshots
17. content_scores
18. strategy_reports
19. rate_limits
20. action_log
21. cursors
22. author_interactions
23. media_uploads
24. llm_usage
25. x_api_usage
26. mcp_telemetry
27. mutation_audit
28. source_contexts
29. sessions
30. accounts
```

---

## Architecture Placement

### Core (`tuitbot-core`)

New file: `crates/tuitbot-core/src/storage/reset.rs`

```rust
pub struct ResetStats {
    pub tables_cleared: u32,
    pub rows_deleted: u64,
}

pub async fn factory_reset(pool: &DbPool) -> Result<ResetStats, StorageError>
```

This is a storage-layer concern.  It only handles DB table clearing in a
transaction.  File deletion and runtime management are the server handler's
responsibility (they depend on `AppState`).

### Server (`tuitbot-server`)

Handler in `routes/settings.rs` (same module as other settings endpoints):

```rust
pub async fn factory_reset(
    State(state): State<Arc<AppState>>,
    Json(body): Json<FactoryResetRequest>,
) -> Result<impl IntoResponse, ApiError>
```

Thin adapter: validates phrase, orchestrates stop/clear/delete steps, returns
envelope.

Route: `.route("/settings/factory-reset", post(routes::settings::factory_reset))`

### Dashboard

New component: `DangerZoneSection.svelte` in settings page.
New API method: `api.settings.factoryReset(confirmation)`.
New nav entry: `{ id: 'danger', label: 'Danger', icon: AlertTriangle }`.

---

## UX Design

The Danger Zone section appears at the bottom of the Settings page, visually
separated with red/danger styling.

**Contents:**
1. Section title: "Danger Zone"
2. Warning text explaining what reset does (deletes all data, returns to
   onboarding).
3. Explicit list of what gets deleted vs preserved.
4. Text input: "Type RESET TUITBOT to confirm"
5. Red "Factory Reset" button, disabled until phrase matches exactly.
6. Loading spinner during API call.
7. On success: hard redirect to `/onboarding`.

---

## Safety Rules

1. **Typed confirmation required** -- The phrase "RESET TUITBOT" must be typed
   exactly.  No timer-based, no double-click.
2. **Auth required** -- Destructive action stays behind existing auth and CSRF
   protections.
3. **No content source deletion** -- User-authored content folders referenced
   in `content_sources` config are never touched.
4. **No schema deletion** -- Tables are cleared (DELETE), not dropped.
5. **api_token preserved** -- Bearer auth survives reset for Tauri/CLI.
6. **Transaction safety** -- DB changes in a single transaction to prevent
   partial state.
7. **Runtime stop-first** -- All automation loops stopped before clearing data
   to prevent writes during reset.

---

## Risks and Mitigations

| Risk                           | Mitigation                                           |
|--------------------------------|------------------------------------------------------|
| Partial reset (crash mid-way)  | DB changes in single transaction; file deletions after DB success; response includes partial stats |
| User accidentally resets       | Explicit typed phrase "RESET TUITBOT"; red danger styling; warning text |
| FK constraint violations       | Delete in child-first order verified from migration FOREIGN KEY clauses |
| Pool open while clearing       | DELETE rows, never DROP tables or delete DB file      |
| Watchtower writes during reset | Cancel watchtower token before clearing DB            |
| Content source folders deleted | Only delete `data_dir/media/`; never touch paths from `content_sources` config |
| `api_token` accidentally deleted | Explicitly preserved; not in deletion list          |
| Requests during reset          | Runtimes lock held during shutdown; DB transaction for atomicity |

---

## File Plan

### Session 2: Backend

| Action | File |
|--------|------|
| Create | `crates/tuitbot-core/src/storage/reset.rs` |
| Modify | `crates/tuitbot-core/src/storage/mod.rs` (add `pub mod reset;`) |
| Modify | `crates/tuitbot-server/src/routes/settings.rs` (add handler) |
| Modify | `crates/tuitbot-server/src/lib.rs` (add route) |
| Create | `crates/tuitbot-server/tests/factory_reset.rs` |

### Session 3: Frontend

| Action | File |
|--------|------|
| Create | `dashboard/src/routes/(app)/settings/DangerZoneSection.svelte` |
| Modify | `dashboard/src/lib/api.ts` (add `factoryReset` method) |
| Modify | `dashboard/src/routes/(app)/settings/+page.svelte` (add section) |

---

## Testing Strategy

### Unit Tests (`tuitbot-core`)
- `factory_reset_clears_all_tables` -- insert sample data, reset, verify zero
  rows in each table.
- `factory_reset_preserves_migrations` -- verify `_sqlx_migrations` untouched.
- `factory_reset_returns_accurate_stats` -- check row counts match.

### Integration Tests (`tuitbot-server`)
- `factory_reset_requires_auth` -- 401 without token.
- `factory_reset_requires_correct_confirmation` -- 400 with wrong phrase.
- `factory_reset_clears_data_and_files` -- full reset, verify everything
  cleared.
- `factory_reset_clears_session_cookie` -- check Set-Cookie header.
- `factory_reset_allows_re_onboarding` -- reset then POST /settings/init
  succeeds.
- `factory_reset_idempotent` -- reset on already-reset instance succeeds.

### Frontend Checks
- `npm run check` (svelte-check) passes.
- `npm run build` succeeds.
- Manual: reset -> redirects to onboarding -> can re-onboard.
