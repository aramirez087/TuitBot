# Session 05 Handoff

## What Was Done

Built the dashboard shell that boots, persists, and switches active account context without stale data leaking across routes.

### New Files

- **`docs/roadmap/dashboard-multi-account/frontend-switching-flow.md`** — Documents the bootstrap sequence, switch flow, store invalidation pattern, WebSocket filtering contract, and AccountSwitcher states.

### Modified Files (Backend)

- **`crates/tuitbot-server/src/ws.rs`** — Added `AccountWsEvent` wrapper struct with `account_id: String` and `#[serde(flatten)] event: WsEvent`. Lagged-channel error events use empty `account_id`.

- **`crates/tuitbot-server/src/state.rs`** — Changed `event_tx` type from `broadcast::Sender<WsEvent>` to `broadcast::Sender<AccountWsEvent>`.

- **`crates/tuitbot-server/src/main.rs`** — Updated broadcast channel creation to use `AccountWsEvent`.

- **`crates/tuitbot-server/src/routes/approval.rs`** — Wrapped 3 `event_tx.send()` calls with `AccountWsEvent { account_id: ctx.account_id.clone(), event: ... }`.

- **`crates/tuitbot-server/src/routes/runtime.rs`** — Wrapped 2 `event_tx.send()` calls with `AccountWsEvent`.

- **`crates/tuitbot-server/src/routes/content/compose.rs`** — Wrapped 7 `event_tx.send()` calls with `AccountWsEvent`.

- **`crates/tuitbot-server/src/routes/assist.rs`** (tests module) — Updated test `AppState` construction to use `AccountWsEvent`.

- **`crates/tuitbot-server/tests/api_tests.rs`** — Updated all `broadcast::channel` type parameters from `WsEvent` to `AccountWsEvent`.

- **`crates/tuitbot-server/tests/compose_contract_tests.rs`** — Same.

- **`crates/tuitbot-server/tests/factory_reset.rs`** — Same.

- **`crates/tuitbot-server/tests/assist_rag_tests.rs`** — Same.

- **`crates/tuitbot-server/tests/fresh_install_auth.rs`** — Same.

### Modified Files (Frontend)

- **`dashboard/src/lib/stores/websocket.ts`** — Added `account_id` filtering in `onmessage` (skips events for non-active accounts). Added `clearEvents()` export to flush buffer on account switch.

- **`dashboard/src/lib/stores/accounts.ts`** — Made `initAccounts()` async with bootstrap validation against `/api/accounts`. Added `bootstrapState` store (`'loading' | 'ready' | 'error'`). `switchAccount()` now flushes WS events and dispatches `tuitbot:account-switched` CustomEvent. Exported `ACCOUNT_SWITCHED_EVENT` constant. `fetchAccounts()` now returns the list.

- **`dashboard/src/lib/components/AccountSwitcher.svelte`** — Always rendered (removed `{#if accounts.length > 1}` guard). Shows avatar from `x_avatar_url` when available. Accepts `collapsed` prop for sidebar-collapsed mode. Added "Add Account" dropdown item (navigates to `/settings`).

- **`dashboard/src/lib/components/Sidebar.svelte`** — Removed `accounts` import and conditional guard. Always renders `<AccountSwitcher {collapsed} />`.

- **`dashboard/src/routes/(app)/+layout.svelte`** — `onMount` now awaits async `initAccounts()`. Shows loading spinner while `bootstrapState === 'loading'`. Removed redundant `fetchAccounts()` call (now handled inside `initAccounts`).

- **12 page files** — Each account-scoped page now listens for `tuitbot:account-switched` and refetches its data:
  - `+page.svelte` (Home)
  - `activity/+page.svelte`
  - `approval/+page.svelte`
  - `content/+page.svelte`
  - `costs/+page.svelte`
  - `discovery/+page.svelte`
  - `drafts/+page.svelte`
  - `mcp/+page.svelte`
  - `observability/+page.svelte`
  - `settings/+page.svelte`
  - `strategy/+page.svelte`
  - `targets/+page.svelte`

## Key Decisions Made

| Decision | Rationale |
|----------|-----------|
| `AccountWsEvent` wrapper with `#[serde(flatten)]` | Zero breaking change to JSON shape — just one new `account_id` field. Keeps `WsEvent` enum clean. |
| Client-side WS filtering (not server-side channels) | Per charter D4. Simpler than channel-per-account. Adequate for low event volume. |
| `window.dispatchEvent` for switch notification | Decouples accounts store from all page stores. Each page owns its own refetch logic. |
| `bootstrapState` store for loading gate | Prevents pages from rendering with stale/invalid account context during async validation. |
| AccountSwitcher always visible | Even with one account, shows identity and "Add Account" entry point. |
| `clearEvents()` on switch | Prevents stale WS events from previous account flashing in the new context. |
| "Add Account" navigates to `/settings` | Placeholder for future credential linking flow (deferred from this session). |

## Quality Gates Passed

- `cargo fmt --all && cargo fmt --all --check` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — all tests pass
- `cargo clippy --workspace -- -D warnings` — clean
- `npm --prefix dashboard run check` — 0 errors, 6 pre-existing warnings
- `npm --prefix dashboard run build` — clean

## Open Issues for Session 6

1. **Credential linking UI.** The "Add Account" button navigates to `/settings` as a placeholder. A dedicated account management page or settings section needs: (a) OAuth flow initiation for new X accounts, (b) scraper session import, (c) account label editing, (d) account deletion with confirmation.

2. **WebSocket account scoping for watchtower.** Watchtower events (content source changes) are instance-level, not account-scoped. If watchtower events are added to WsEvent in the future, they should use an empty `account_id` to pass through the client filter.

3. **Config reload after PATCH.** When `PATCH /api/settings` modifies config, running runtimes and cached generators don't pick up changes. Pre-existing issue, more relevant now with per-account caches.

4. **Automation loop spawning.** `start` creates an empty `Runtime`. Full loop setup (discovery, content, mentions) needs XApiClient + LLM adapters wired up in the server crate.

5. **Approval stats refetch on switch.** The sidebar's pending count badge (`loadApprovalStats`) is loaded once on layout mount. It should also refetch on account switch to show the correct count for the new account.

6. **Auto-refresh timers reset on switch.** Pages with `startAutoRefresh()` (activity, approval, content, targets, mcp) keep their timers running after switch. The refetch handler fires the load function, but the auto-refresh timer may fire with stale state between switches. Consider resetting timers in the switch handler.

## Exact Inputs for Session 6

### Files to Create/Modify

- `dashboard/src/routes/(app)/settings/AccountsSection.svelte` — New component for account management UI
- `dashboard/src/routes/(app)/settings/+page.svelte` — Add AccountsSection to settings page
- `crates/tuitbot-server/src/routes/x_auth.rs` — May need frontend-facing OAuth flow helpers

### Key Contracts to Respect

- `switchAccount()` dispatches `tuitbot:account-switched` and clears WS events
- `initAccounts()` validates persisted ID against API response
- `AccountWsEvent` wraps all broadcast events with `account_id`
- X-auth endpoints are at `/api/accounts/{id}/x-auth/start|callback|status`
- Account CRUD is at `/api/accounts` and `/api/accounts/{id}`
