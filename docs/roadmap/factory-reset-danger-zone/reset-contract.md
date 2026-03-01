# Factory Reset API Contract

Finalized in Session 2. This is the authoritative contract for Session 3
(frontend) to build against.

---

## Endpoint

```
POST /api/settings/factory-reset
```

## Authentication

Required. Bearer token OR session cookie + CSRF header.

This route is NOT in `AUTH_EXEMPT_PATHS`. Unauthenticated requests receive
HTTP 401.

## Request Body

```json
{
  "confirmation": "RESET TUITBOT"
}
```

- `confirmation` (string, required): Must be exactly `"RESET TUITBOT"`
  (case-sensitive ASCII). Any other value returns HTTP 400.

## Success Response (200 OK)

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

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Always `"reset_complete"` on success |
| `cleared.tables_cleared` | u32 | Number of DB tables cleared (always 30) |
| `cleared.rows_deleted` | u64 | Total rows deleted across all tables |
| `cleared.config_deleted` | bool | Whether `config.toml` was deleted |
| `cleared.passphrase_deleted` | bool | Whether `passphrase_hash` was deleted |
| `cleared.media_deleted` | bool | Whether `media/` directory was deleted |
| `cleared.runtimes_stopped` | u32 | Number of automation runtimes stopped |

### Response Headers

```
Set-Cookie: tuitbot_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0
```

Always present. Clears the session cookie for web callers. Harmless for
bearer callers (they don't use cookies).

## Error Responses

| Status | Condition | Body |
|--------|-----------|------|
| 400 | Wrong or missing confirmation phrase | `{"error": "incorrect confirmation phrase"}` |
| 401 | Not authenticated | `{"error": "unauthorized"}` |
| 403 | Missing CSRF token (cookie auth) | `{"error": "..."}` |
| 500 | DB transaction failed | `{"error": "database query error: ..."}` |

## Post-Reset State

After a successful reset:

1. `GET /api/settings/status` returns `{"configured": false, "claimed": false, ...}`.
2. The frontend `+layout.svelte` boot logic detects `configured=false` and
   redirects to `/onboarding`.
3. `POST /api/settings/init` accepts a new configuration (re-onboarding).
4. Bearer auth (Tauri) continues to work -- `api_token` is preserved.
5. All automation runtimes are stopped.
6. All sessions are deleted from the DB.

## What Gets Cleared

| Category | Items | Method |
|----------|-------|--------|
| DB tables | All 30 user tables (FK-safe order) | Single transaction |
| Config | `config.toml` | `fs::remove_file` |
| Passphrase | `passphrase_hash` file + in-memory | `fs::remove_file` + `RwLock` |
| Media | `media/` directory tree | `fs::remove_dir_all` |
| Runtimes | All automation loops | `Runtime::shutdown()` |
| Watchtower | File watcher | `CancellationToken::cancel()` |
| Generators | LLM content generators | `HashMap::clear()` |
| Rate limits | Login attempt tracking | `HashMap::clear()` |

## What Is Preserved

| Item | Reason |
|------|--------|
| `api_token` file | Bearer auth must survive for Tauri |
| DB schema (tables, indexes) | Pool and migrations must remain usable |
| `_sqlx_migrations` table | Migration tracking |
| `backups/` directory | Safety artifacts |
| Server process | Live reset requirement |

## Frontend Integration Notes (Session 3)

### API Client Method

```typescript
factoryReset: (confirmation: string) =>
    request<{ status: string; cleared: Record<string, unknown> }>(
        '/api/settings/factory-reset',
        {
            method: 'POST',
            body: JSON.stringify({ confirmation })
        }
    )
```

### Post-Reset Redirect

No special redirect logic needed. After the API call succeeds:

1. The response clears the session cookie via `Set-Cookie`.
2. On the next navigation or API call, `+layout.svelte` calls
   `GET /api/settings/status` and sees `configured: false`.
3. The existing boot logic redirects to `/onboarding`.

For the best UX, the frontend should do a hard redirect to `/onboarding`
immediately after a successful reset response (e.g., `window.location.href =
'/onboarding'` or `goto('/onboarding')`).

### Confirmation Phrase

The typed phrase is `RESET TUITBOT` (uppercase ASCII, exact match). The
frontend should:

1. Show a text input with placeholder "Type RESET TUITBOT to confirm".
2. Disable the submit button until the input value === "RESET TUITBOT".
3. Send the exact typed value in the `confirmation` field.
