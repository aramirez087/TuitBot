# Frontend Account Switching Flow

Documents the bootstrap sequence, switch flow, store invalidation pattern, WebSocket filtering, and AccountSwitcher states for the dashboard multi-account shell.

## Bootstrap Sequence

On app mount (`(app)/+layout.svelte`), `initAccounts()` runs before any page renders:

1. Set `bootstrapState = 'loading'` — layout shows a spinner instead of page content.
2. Read persisted account ID from `localStorage` (`tuitbot-account-id` key).
3. Set the HTTP client's `X-Account-Id` header to the persisted ID.
4. Fetch `GET /api/accounts` to get the full account list.
5. Validate the persisted ID against the list:
   - **Found**: Keep it. No switch needed.
   - **Not found** (deleted account, stale localStorage): Fall back to the default account (`00000000-...`) or the first account in the list.
   - **Empty list** (accounts endpoint not available): Keep persisted ID for backward compat.
6. Set `bootstrapState = 'ready'` — layout renders children.
7. Sync the current account's X profile data (avatar, display name).

If `initAccounts()` throws, `bootstrapState` is set to `'ready'` anyway so the app is never stranded.

## Account Switch Flow

When `switchAccount(id)` is called:

1. Update the `currentAccountId` Svelte store.
2. Update the HTTP client's `X-Account-Id` header via `setAccountId()`.
3. Persist the new ID to `localStorage`.
4. Flush the WebSocket events buffer (`clearEvents()`), resetting `runtimeRunning` to false.
5. Dispatch a `tuitbot:account-switched` CustomEvent on `window`.

## Store Invalidation Pattern

Each account-scoped page listens for the `tuitbot:account-switched` event:

```typescript
import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

onMount(() => {
    loadPageData();
    const handler = () => loadPageData();
    window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
    return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
});
```

Pages that hold local state (e.g., `schedule = null` on the home page) reset it before refetching to prevent flash of stale data.

### Pages with switch listeners

| Page | Refetch on switch |
|------|-------------------|
| Home | `loadHomeSurface()`, `api.content.schedule()`, `api.runtime.status()` |
| Activity | `loadActivity(true)` |
| Approval | `loadItems(true)`, `loadStats()` |
| Content | `loadSchedule()`, `loadCalendar()` |
| Costs | `loadCosts(days)` |
| Discovery | `loadFeed()`, `loadKeywords()` |
| Drafts | `loadDrafts()` |
| MCP | `loadMcpData(hours)` |
| Observability | `loadAll()` |
| Settings | `loadSettings()` |
| Strategy | `loadStrategy()` |
| Targets | `loadTargets()`, `loadSettings()` |

## WebSocket Filtering

### Backend

All `WsEvent` variants are wrapped in `AccountWsEvent`:

```rust
pub struct AccountWsEvent {
    pub account_id: String,
    #[serde(flatten)]
    pub event: WsEvent,
}
```

The `#[serde(flatten)]` attribute means the JSON shape is backward compatible — just one new `account_id` field alongside the existing `type` discriminator.

### Frontend

In `websocket.ts`, `onmessage` filters events before adding to the store:

```typescript
const activeAccountId = getAccountId();
if (event.account_id && event.account_id !== activeAccountId) {
    return; // Skip events from other accounts
}
```

Events without `account_id` (e.g., lag errors) pass through.

## AccountSwitcher States

The `AccountSwitcher` component is always rendered (no `{#if accounts.length > 1}` guard):

| State | Behavior |
|-------|----------|
| **0 accounts** | Shows "Default" with User icon |
| **1 account** | Shows account identity (avatar + @username). Dropdown has "Add Account" only |
| **Multiple accounts** | Full dropdown with all accounts + "Add Account" |
| **Collapsed sidebar** | Shows avatar-only (or User icon). Dropdown accessible via click |

Avatar images from `x_avatar_url` are shown when available; falls back to `User` icon.

"Add Account" navigates to `/settings` (placeholder for future credential linking flow).
