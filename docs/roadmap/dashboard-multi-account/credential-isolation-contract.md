# Credential Isolation Contract

## Per-Account File Layout

Each non-default account stores credentials under:

```
data_dir/
  tokens.json                    # default account OAuth tokens
  scraper_session.json           # default account scraper session
  accounts/
    {account_id}/
      tokens.json                # non-default account OAuth tokens
      scraper_session.json       # non-default account scraper session
```

Path helpers in `tuitbot_core::storage::accounts`:
- `account_data_dir(data_dir, account_id)` -> root for default, `accounts/{id}/` for others
- `account_token_path(data_dir, account_id)` -> `{account_data_dir}/tokens.json`
- `account_scraper_session_path(data_dir, account_id)` -> `{account_data_dir}/scraper_session.json`

## Token Format

Token files use the `tuitbot_core::x_api::auth::Tokens` struct:

```rust
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,       // required (not Optional)
    pub expires_at: DateTime<Utc>,   // required (not Optional)
    pub scopes: Vec<String>,
}
```

This is the format expected by `auth::load_tokens` and `AppState::get_x_access_token`. The `StoredTokens` type from `startup.rs` (which has Optional fields) is used only during the OAuth exchange flow and is immediately converted to `auth::Tokens` before saving.

## can_post Computation

The `can_post` flag is computed per-account via `content::can_post_for(state, account_id)`:

| Provider Backend | Condition for `can_post = true` |
|---|---|
| `x_api` | `account_token_path(data_dir, account_id)` exists |
| `scraper` | `account_scraper_session_path(data_dir, account_id)` exists |
| other/empty | always `false` |

The `provider_backend` is resolved from the effective config (base config merged with account's `config_overrides`). For the default account, this is the raw `config.toml` value. For non-default accounts, the `x_api` section can be overridden per-account.

## X Auth Linking Flow

### Endpoints

| Method | Path | Description |
|---|---|---|
| POST | `/api/accounts/{id}/x-auth/start` | Generate PKCE challenge, return auth URL |
| POST | `/api/accounts/{id}/x-auth/callback` | Exchange code for tokens, save to account path |
| GET | `/api/accounts/{id}/x-auth/status` | Check OAuth + scraper credential status |

### PKCE State Management

The `PendingOAuth` struct now includes an `account_id` field:

```rust
pub struct PendingOAuth {
    pub code_verifier: String,
    pub created_at: Instant,
    pub account_id: String,  // which account initiated the flow
}
```

- The `start` endpoint stores the account ID with the PKCE state.
- The `callback` endpoint validates that the state's `account_id` matches the URL path `{id}`.
- Entries expire after 10 minutes (consistent with connector OAuth).
- Connector OAuth flows set `account_id: String::new()` (not account-scoped).

### Token Save + Evict Pattern

On successful token exchange:
1. Convert `StoredTokens` -> `auth::Tokens` (fill defaults for missing optional fields).
2. Save to `account_token_path(data_dir, account_id)` via `auth::save_tokens` (creates parent dir, sets 0600 permissions).
3. Evict the existing `TokenManager` from `state.token_managers` for this account (forces fresh load on next API call).

### Status Response

```json
{
    "oauth_linked": true,
    "oauth_expired": false,
    "oauth_expires_at": "2026-03-05T14:00:00+00:00",
    "scraper_linked": false,
    "has_credentials": true
}
```

## Scraper Session Isolation

Already implemented in Session 2. All three scraper session endpoints (`get`, `import`, `delete`) accept `AccountContext` and use `account_scraper_session_path()`. The `import` handler creates the parent directory for non-default accounts.

## Effective Config Resolution

The `read_effective_config(state, account_id)` async helper:
- Default account: reads `config.toml` directly (backward compat, no DB query).
- Non-default account: reads `config.toml` as base, fetches account's `config_overrides` from DB, merges via `effective_config()`.

This is used by all `can_post`, `require_post_capable`, `read_approval_mode`, and posting code paths.
