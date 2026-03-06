# Account Management Flow

Documents the UX contract for managing the account roster from the dashboard.

## Surface Location

Settings page > Accounts section (first section, `id="accounts"`). Reachable via:
- Settings left-nav "Accounts" button
- AccountSwitcher dropdown "Add Account" button (navigates to `/settings#accounts`)

## Account Roster

Each account row displays:
- **Avatar** (from `x_avatar_url`) or placeholder icon
- **Identity**: `@username` if linked, otherwise the label
- **Label tag**: shown when both username and label exist and differ
- **Badges**: Default (blue), Active (green), credential status (green/yellow)
- **Actions**: Sync Profile, Rename, Archive (conditionally shown)

### Credential Status

Fetched from `GET /api/accounts/{id}/x-auth/status` on section mount for all accounts.

Badge logic:
- **Linked** (green) - `has_credentials && !oauth_expired`
- **Token expired** (yellow) - `has_credentials && oauth_expired`
- **No credentials** (yellow) - `!has_credentials`

A detail grid at the bottom shows per-account OAuth and scraper status.

## Create Flow

1. User types a label in the inline text input at the bottom of the account list
2. Clicks "Add Account" or presses Enter
3. `POST /api/accounts` creates the account
4. Store refreshes, auto-switches to the new account via `switchAccount()`
5. Auth statuses are re-fetched to include the new account
6. Error is displayed inline below the form if creation fails

## Rename Flow

1. User clicks the pencil icon on an account row
2. Label becomes an editable input with save/cancel buttons
3. User edits and presses Enter or clicks the check icon
4. `PATCH /api/accounts/{id}` updates the label
5. Store updates in-place, row returns to display mode
6. Escape or cancel button aborts without saving

## Archive Flow

1. Archive button only appears for accounts that are:
   - Not the default account (backend also enforces this)
   - Not the currently active account (must switch away first)
2. Clicking archive reveals a confirmation input
3. User types `ARCHIVE` to confirm
4. `DELETE /api/accounts/{id}` soft-deletes the account
5. Store refreshes; if the archived account was somehow still active, falls back to default
6. Auth statuses are re-fetched

## Sync Profile Flow

1. User clicks the refresh icon on any account row
2. `POST /api/accounts/{id}/sync-profile` fetches latest X profile data
3. Store updates the account in-place with new avatar/username/display name
4. Spinner shows during the request

## Guards

| Guard | Enforced by |
|-------|-------------|
| Cannot archive default account | Frontend (button hidden) + Backend (rejects DELETE) |
| Cannot archive active account | Frontend (button hidden) |
| Cannot create with empty label | Frontend (button disabled) |
| Cannot rename to empty label | Frontend (save disabled) |

## Error Handling

All errors are shown as inline red text below the relevant action. Errors are cleared when the user starts a new action. No toast notifications or modals.

## Empty State

When only the default account exists, a centered prompt encourages users to add their first account with a brief description of the next steps.
