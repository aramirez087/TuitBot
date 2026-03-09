# Session 07 Handoff

## What Changed

Implemented server-side X profile provisioning during `init_settings` and idempotent 409 recovery on the frontend, so a newly onboarded user lands in a configured Tuitbot starter state with their X identity active on the default account — no extra steps, no dead ends.

### Modified Files

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/routes/settings.rs` | Added `XProfileData` struct; extract and remove `x_profile` from init body before TOML conversion; after token migration, update default account with X identity fields (non-fatal on failure) |
| `dashboard/src/lib/stores/onboarding.ts` | Added `x_user_id`, `x_username`, `x_display_name`, `x_avatar_url` fields to `OnboardingData` interface and both initial/reset defaults |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | On successful OAuth polling, persist X user data (`id`, `username`, `name`, `profile_image_url`) into the onboarding data store for inclusion in init payload |
| `dashboard/src/routes/onboarding/+page.svelte` | Include `x_profile` in init payload when X identity is available; handle 409 ("already exists") by redirecting to `/` instead of showing an error; handle 409 during already-claimed retry as well |
| `dashboard/src/lib/stores/accounts.ts` | Added `updateAccountLocally()` helper for updating account fields in the store without an API call |

### New Files

| File | Purpose |
|------|---------|
| `crates/tuitbot-server/tests/onboarding_provisioning.rs` | 8 integration tests covering X profile provisioning, token migration, idempotent 409 behavior, claim+profile combo, and invalid payload rejection |
| `docs/roadmap/x-profile-prefill-onboarding/session-07-handoff.md` | This handoff |

## Key Decisions Made

### D1: X profile passed in init payload, not re-fetched
The frontend already has X user data from the OAuth callback polling response. Passing it as an `x_profile` field in the init body avoids a separate `sync-profile` API call that could fail or race. Server extracts it before TOML conversion (same pattern as `claim`).

### D2: Profile population is non-fatal
If `update_account` fails during init (e.g. unexpected DB state), a warning is logged but config creation succeeds. The existing `syncCurrentProfile()` in `(app)/+layout.svelte` serves as a fallback.

### D3: 409 recovery redirects to home
When the frontend gets a 409 from `init_settings` (double-submit, browser back/forward), it redirects to `/` instead of showing an error. The first request already provisioned everything.

### D4: Invalid x_profile fails fast with 400
Malformed `x_profile` returns 400 before any config file or passphrase is created. This prevents partial provisioning.

### D5: x_avatar_url is optional in XProfileData
The `x_avatar_url` field defaults to `None` if not provided. Users without a profile image (or scraper-mode users who skip OAuth) still provision cleanly.

### D6: updateAccountLocally helper for post-init use
A new `updateAccountLocally()` function in the accounts store allows updating account fields in the client-side store without an API call — useful after init when the server already wrote the data and the frontend just needs to reflect it.

## Test Coverage

All 8 new tests in `onboarding_provisioning.rs`:

| Test | Validates |
|------|-----------|
| `init_with_x_profile_populates_account` | X identity written to default account row |
| `init_without_x_profile_leaves_account_empty` | Backward compat — no X fields when not provided |
| `init_with_x_profile_and_claim` | Both claim (passphrase+session) and X profile work together |
| `init_migrates_onboarding_tokens` | `onboarding_tokens.json` moved to default account path |
| `token_migration_missing_source_is_noop` | No error when no onboarding tokens exist |
| `double_init_returns_409` | Second init call returns 409 Conflict |
| `double_init_with_x_profile_first_wins` | First profile persists; second call is rejected |
| `init_with_invalid_x_profile_returns_400` | Malformed x_profile fails before config write |

## Open Issues

1. **No re-analysis trigger** — When a user configures LLM in Settings after skipping during onboarding, there's no "Analyze Profile" button. Carried from Session 05/06.

2. **Tier-gated empty states not implemented** — Discovery, Targets, and other pages don't yet show tier-aware empty states. Carried from Session 06.

3. **No first-draft experience on home page** — DraftStudioShell doesn't have a first-run empty state. Carried from Session 06.

4. **Checklist deep-link scroll verification** — Hash-anchor scrolling may not work in SPA routing. Carried from Session 06.

5. **Measurement hooks not wired** — No analytics events for provisioning funnel, checklist interactions, or tier transitions. Session 08 scope.

6. **Stale onboarding tokens cleanup** — Old `onboarding_tokens.json` files from abandoned flows are never cleaned up. Carried from Session 03/04.

7. **Analyze endpoint unauthenticated** — Carried from Session 03/04.

8. **BusinessStep.svelte still unused** — Carried from Session 04. Can be deleted in cleanup.

## What Session 08 Should Do

### Instrumentation & Polish
1. **Wire measurement events** — Emit analytics/telemetry for: onboarding completion (with/without X auth), provisioning success/failure, checklist item clicks, tier transitions.
2. **Add "Analyze Profile" button to Settings** — When LLM is configured post-onboarding, allow triggering profile analysis from the Business Profile or LLM Provider section.
3. **Tier-gated empty states** — Discovery, Targets, and other pages should show helpful prompts when the user lacks the required tier, linking to the relevant Settings section.
4. **First-draft experience** — DraftStudioShell should guide `profile_ready` users toward their first action instead of a blank interface.
5. **Checklist deep-link scroll** — Verify hash-anchor scrolling works with SPA routing; add `scrollIntoView` if needed.
6. **Cleanup** — Delete unused `BusinessStep.svelte`, add stale onboarding token cleanup.

### Verification
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
cd dashboard && npm run build
```
