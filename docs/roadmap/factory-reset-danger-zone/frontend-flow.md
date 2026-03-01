# Factory Reset -- Frontend UX Flow

## User Journey

1. Navigate to **Settings** (sidebar or direct URL).
2. Scroll to the bottom or click **Danger** in the sticky section nav.
3. The **Danger Zone** section is visually distinct: red-tinted border,
   red icon background, and a warning triangle icon.
4. Read the warning text and the two-column list (deleted vs preserved).
5. Type the exact phrase `RESET TUITBOT` in the confirmation input.
6. The **Factory Reset** button enables (red background, white text).
7. Click the button. A spinner replaces the button text ("Resetting...").
8. On success, the app redirects to `/onboarding` immediately.

## Confirmation UX

The confirmation uses an explicit typed phrase, not a timer-based double
click. The button remains disabled until the input value matches
`RESET TUITBOT` exactly (case-sensitive, no trimming). This prevents
accidental triggers and satisfies the charter requirement for explicit
typed confirmation.

## Auth Mode Handling

### Bearer / Tauri Mode

- The `request()` helper sends `Authorization: Bearer <token>`.
- On success, the server clears all sessions from the DB and emits a
  `Set-Cookie` header (harmless -- Tauri doesn't use cookies).
- `clearSession()` sets the auth store's `authMode` to `'none'` but does
  NOT touch the api module's internal `token` or `authMode` (`'bearer'`).
  The bearer token and `api_token` file survive the reset.
- After `goto('/onboarding')`, the layout's existing boot logic re-detects
  Tauri context and re-sets `authMode` to `'tauri'` in the store.
- Re-onboarding works because `/api/settings/init` is auth-exempt and
  subsequent API calls use the surviving bearer token.

### Cookie / Web Mode

- The `request()` helper sends `credentials: 'include'` and the
  `X-CSRF-Token` header for the POST.
- On success, the server's `Set-Cookie` header clears the session cookie.
- `clearSession()` clears the CSRF token and sets `authMode` to `'none'`.
- After `goto('/onboarding')`, the onboarding page shows the claim step
  (passphrase was deleted). After claiming, a new session is established.

## State Cleanup Sequence

On successful reset response:

1. `clearSession()` -- clears CSRF token, sets auth store to `'none'`.
2. `resetStores()` -- nulls out `config`, `defaults`, `draft`, resets
   `loading` to `true`, clears errors and validation state.
3. `disconnectWs()` -- closes the WebSocket and cancels the reconnect
   timer. Without this, the exponential-backoff reconnect loop would run
   indefinitely (sessions are deleted, so WS auth fails).
4. `goto('/onboarding')` -- SvelteKit client-side navigation to the
   onboarding page. Falls back to `window.location.href` if `goto` throws.

## Known Limitations

### WebSocket in Other Tabs

If the user has multiple browser tabs open, only the tab that initiated
the reset performs cleanup. Other tabs' WebSocket connections will fail
on the next reconnect attempt (sessions are cleared). The exponential
backoff will keep retrying until the user refreshes. On the next full
navigation, the layout boot check detects `configured=false` and
redirects to `/onboarding`.

A cross-tab notification via `BroadcastChannel` API is out of scope for
this session.

### Server Subsystem Hot-Restart

The server does not hot-restart runtimes or the watchtower after reset.
After re-onboarding, the user must manually start the runtime (via the
dashboard play button or API call). This is consistent with the initial
onboarding flow, which also requires explicit runtime activation.

### Multi-Tab Stale Data

Other open tabs will display stale data (cached analytics, targets, etc.)
until the user refreshes or navigates. The `+layout.svelte` boot logic
will detect the unconfigured state on the next navigation and redirect
to `/onboarding`.

### Other Stores

Stores outside of `settings` (analytics, approval, activity, etc.) are
not explicitly cleared. They will fail gracefully on their next API call
since the config no longer exists. This is acceptable because the user
is redirected to `/onboarding` and won't access those pages until after
re-onboarding.
