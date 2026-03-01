# Session 02 Handoff -- Factory Reset Danger Zone

## Completed Work

1. **Core storage layer** (`crates/tuitbot-core/src/storage/reset.rs`):
   - `ResetStats` struct with `tables_cleared` and `rows_deleted` fields.
   - `TABLES_TO_CLEAR` compile-time constant: 30 tables in FK-safe deletion
     order (children before parents).
   - `factory_reset(pool) -> Result<ResetStats, StorageError>`: single-
     transaction DELETE across all 30 tables.
   - 5 unit tests: clears all tables, preserves migrations, accurate stats,
     idempotent on empty DB, table list covers all user tables.

2. **Module registration** (`crates/tuitbot-core/src/storage/mod.rs`):
   - Added `pub mod reset;` in alphabetical order.

3. **Server handler** (`crates/tuitbot-server/src/routes/settings.rs`):
   - `FactoryResetRequest`, `FactoryResetResponse`, `FactoryResetCleared`
     types.
   - `factory_reset` handler: validates phrase, stops runtimes, cancels
     watchtower, clears DB, deletes files, clears in-memory state, returns
     response with `Set-Cookie` clearing header.

4. **Route registration** (`crates/tuitbot-server/src/lib.rs`):
   - `.route("/settings/factory-reset", post(routes::settings::factory_reset))`
     placed after `/settings/test-llm` and before `/settings` (literal
     before parameterized).

5. **Integration tests** (`crates/tuitbot-server/tests/factory_reset.rs`):
   - 7 test cases, all passing:
     - `factory_reset_requires_auth` -- 401 without token.
     - `factory_reset_rejects_wrong_confirmation` -- 400 for wrong phrase,
       case mismatch, and empty string.
     - `factory_reset_success` -- full happy path: seeds config, passphrase,
       media, verifies all cleared, cookie header present, in-memory state
       cleared.
     - `factory_reset_idempotent` -- second reset succeeds with 0 rows.
     - `factory_reset_clears_config_status` -- `configured=false`,
       `claimed=false` after reset.
     - `factory_reset_cookie_clearing` -- verifies all cookie attributes.
     - `factory_reset_allows_re_onboarding` -- reset then init succeeds.

6. **API contract documentation** (`reset-contract.md`):
   - Finalized endpoint, request/response shapes, error codes, post-reset
     state, frontend integration notes.

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Skip VACUUM | Cannot run inside transaction; SQLite reuses freed pages; avoids exclusive lock. Resolves Session 1 open issue #1. |
| No `AccountContext` extractor | Factory reset is instance-level. Auth middleware still protects the route via bearer/session. |
| Always emit `Set-Cookie` clearing | Harmless for bearer; necessary for cookie callers. Simpler than detecting auth strategy. |
| Per-step booleans in response | Resolves Session 1 open issue #2: partial failure transparency. |
| No WebSocket event | Frontend detects reset via `configured=false`. Resolves Session 1 open issue #3. |
| `drain()` all runtimes | Resolves Session 1 open issue #4: multi-account clearing. |
| Migration-seeded rows in tests | The `multi_account_foundation` migration seeds 1 account + 2 roles. Tests account for these 3 rows. |
| No direct sqlx in integration tests | `tuitbot-server` doesn't depend on `sqlx`. Verified DB clearing via second reset returning `rows_deleted: 0`. |

## CI Results

All gates pass:

```
cargo fmt --all && cargo fmt --all --check    # OK
RUSTFLAGS="-D warnings" cargo test --workspace  # all tests pass
cargo clippy --workspace -- -D warnings         # clean
```

## Open Issues

None. All four open issues from Session 1 have been resolved.

## Exact Inputs for Session 3

### Files to Create

1. `dashboard/src/routes/(app)/settings/DangerZoneSection.svelte`

### Files to Modify

2. `dashboard/src/lib/api.ts` -- add `factoryReset` method to `api.settings`
3. `dashboard/src/routes/(app)/settings/+page.svelte` -- import section, add
   nav entry (`AlertTriangle` icon, id `danger`), render after LAN section

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

### UX Requirements

- Section title: "Danger Zone"
- Warning text explaining what reset does.
- Explicit list of what gets deleted vs preserved.
- Text input: "Type RESET TUITBOT to confirm"
- Red "Factory Reset" button, disabled until phrase matches exactly.
- Loading spinner during API call.
- On success: hard redirect to `/onboarding`.

### Frontend Checks

```bash
cd dashboard && npm run check
cd dashboard && npm run build
```

### Backend Contract

See `reset-contract.md` for the complete API contract.
