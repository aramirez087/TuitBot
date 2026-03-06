# Settings Override UX Contract

Defines how the settings page communicates account-scoped vs instance-scoped fields and provides override management affordances.

## Scope Model

Every settings section is classified as either **account-scoped** or **instance-scoped**, matching the backend's `ACCOUNT_SCOPED_KEYS` in `merge.rs` and the `settings-scope-matrix.md` contract.

The frontend constant `SECTION_SCOPE` in `dashboard/src/lib/stores/settingsScope.ts` maps each section `id` to its scope type and corresponding top-level config key(s).

| Section | Scope | Config Key(s) |
|---------|-------|---------------|
| Business Profile | account | `business` |
| Content Persona | account | `business` |
| Scoring Engine | account | `scoring` |
| Safety & Limits | account | `limits` |
| Schedule | account | `schedule` |
| X Access | account | `x_api` |
| Content Sources | account | `content_sources` |
| LLM Provider | instance | `llm` |
| Storage | instance | `storage` |
| LAN Access | instance | (none) |
| Danger Zone | instance | (none) |
| Workspace | (none) | (none) -- local pref |
| Accounts | (none) | (none) -- CRUD section |

## Badge Semantics

Badges appear **only when viewing a non-default account**. The default account always renders sections normally with no badges.

### Account-Scoped Sections

- **"Overridden" badge** (accent-colored): The section's config key appears in the `_overrides` array from `GET /api/settings`. A "Reset to base" button appears in the section header.
- **"Inherited" badge** (muted): The section's config key is NOT in `_overrides`. The account uses the base config value for this section.

### Instance-Scoped Sections

- A warning banner appears: "Shared across all accounts. Switch to the default account to edit."
- Section content is visually dimmed (`opacity: 0.45`) and non-interactive (`pointer-events: none`).
- No badge is shown since the scope classification itself is the signal.

## Reset to Base

The "Reset to base" button sends `PATCH /api/settings` with `{ [key]: null }`. Per the `merge_overrides` contract, `null` removes the key from the account's `config_overrides`, causing it to fall back to the base config.

After reset, the settings are reloaded and the section transitions from "Overridden" to "Inherited".

### Shared Key Caveat

Business Profile and Content Persona both map to the `business` config key. Resetting either section resets the entire `business` override, affecting both sections. This matches the merge granularity (top-level key replace).

## Dirty Draft Protection

When switching accounts while the settings form has unsaved changes:

1. The draft is automatically discarded (`resetDraft()`).
2. The SaveBar briefly shows "Unsaved changes were discarded" for 3 seconds.
3. New settings for the switched-to account are loaded.

This prevents cross-account leakage where one account's draft could be saved to another account's overrides.

### Limitation

There is no pre-switch confirmation modal. The switch happens immediately (via `switchAccount()` in `AccountSwitcher.svelte`) before the settings page can intercept. A future enhancement could add a pre-switch confirmation by checking dirty state in the `AccountSwitcher` component itself.

## Implementation Details

### SettingsSection Component

`SettingsSection.svelte` accepts two optional props:
- `scope?: 'account' | 'instance'` -- the section's scope classification
- `scopeKey?: string` -- the top-level config key for override detection

When both are omitted, the component renders identically to its pre-session-7 behavior (backward compatible).

### Store Dependencies

- `overriddenKeys` (from `settings.ts`) -- populated by `loadSettings()` from the `_overrides` envelope field
- `currentAccountId` (from `accounts.ts`) -- used to detect non-default account
- `resetSectionToBase(key)` (from `settings.ts`) -- PATCHes null and reloads settings
