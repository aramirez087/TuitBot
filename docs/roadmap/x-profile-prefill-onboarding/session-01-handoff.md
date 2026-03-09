# Session 01 Handoff

## What Changed

Created four planning documents defining the deployment-unified onboarding flow:

| File | Purpose |
|------|---------|
| `epic-charter.md` | Implementation charter: scope, shared/divergent steps, architecture decisions, risk register, session breakdown |
| `target-flow.md` | Step-by-step target journey with data flow, state additions, and scraper mode fallback |
| `inference-contract.md` | Full `InferredProfile` type contract (TypeScript + Rust), inference rules per field, LLM prompt template, confidence logic |
| `session-01-handoff.md` | This file |

No code was modified.

## Key Decisions Made

1. **AD-1: X Sign-In as Step 1 Phase B** — OAuth PKCE happens immediately after Client ID entry, during onboarding (not post-setup). Tokens stored temporarily until account creation.

2. **AD-2: LLM before Profile Analysis** — Step order changed to Welcome > X Access+Auth > LLM > Analyze > Review > Language > Sources > Validate > Review > Secure. LLM is needed to power inference.

3. **AD-3: Extended User type** — `description`, `location`, `url`, `entities` added to X API User struct. `USER_FIELDS` constant updated.

4. **AD-4: Inference in core::workflow** — `profile_inference` module in Layer 2, not in server handlers or frontend.

5. **AD-5: Single analysis endpoint** — `POST /api/onboarding/analyze-profile` does fetch + inference in one call.

6. **AD-6: Single-screen editable prefill** — All inferred fields on one screen with confidence badges and provenance tags.

7. **AD-7: Scraper mode graceful degradation** — Scraper users get current manual flow, no regression.

## Open Issues

1. **Temporary token storage path** — During onboarding, OAuth tokens need a temporary home because no account exists yet. Proposed: `data_dir/onboarding_tokens.json`, migrated to account token path after `POST /api/settings/init`. Session 02 must decide the exact path and cleanup strategy.

2. **Onboarding OAuth endpoint design** — Current `x_auth.rs` endpoints are account-scoped (`/api/accounts/{id}/x-auth/*`). Onboarding needs pre-account OAuth. Two options:
   - New `/api/onboarding/x-auth/*` endpoints that don't require an account ID
   - Reuse existing endpoints with the default account ID (`DEFAULT_ACCOUNT_ID`)
   Session 02 should choose based on implementation complexity.

3. **LLM step ordering user experience** — Moving LLM to step 2 (before profile analysis) is the simplest implementation. If early user feedback shows this is confusing (users expect profile setup before AI config), implement the two-pass fallback: heuristic extraction from bio first, LLM enrichment after LLM is configured.

4. **Non-English profile handling** — The LLM prompt is English-focused. For non-English bios/tweets, the LLM should still extract meaningful fields, but inference quality may vary. Not a blocker for MVP; monitor and iterate.

## What Session 02 Must Do

### Mission
Implement the X API User type extension and onboarding OAuth flow.

### Files to Modify

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/x_api/types.rs` | Add `description: Option<String>`, `location: Option<String>`, `url: Option<String>` to `User` struct |
| `crates/tuitbot-core/src/x_api/client/mod.rs` | Update `USER_FIELDS` constant to include `description,location,url` |
| `crates/tuitbot-server/src/routes/x_auth.rs` | Add onboarding-specific OAuth endpoints (or decide to reuse account-scoped ones with default account) |
| `crates/tuitbot-server/src/routes/mod.rs` | Register new onboarding OAuth routes |
| `crates/tuitbot-server/src/lib.rs` | Wire new routes into Axum router |
| `dashboard/src/lib/stores/onboarding.ts` | Add `x_connected`, `x_username`, `x_display_name`, `x_avatar_url`, `oauth_state` fields |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | Add Phase B: OAuth connect button, status polling, connected state display |
| `dashboard/src/routes/onboarding/+page.svelte` | Update step order constants and `canAdvance()` logic for new OAuth requirement |

### Decisions Session 02 Must Make

1. Choose between new `/api/onboarding/x-auth/*` endpoints vs reusing existing account-scoped endpoints with `DEFAULT_ACCOUNT_ID`.
2. Define exact temporary token storage path and cleanup strategy.
3. Decide whether to add `entities` field to User struct (complex nested type) or just `description`, `location`, `url` (simpler, covers 95% of inference value).

### Verification

After Session 02:
- `cargo fmt --all && cargo fmt --all --check` passes
- `RUSTFLAGS="-D warnings" cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` passes
- `get_me()` returns `User` with populated `description` field (verifiable with existing tests or a new fixture test)
- Onboarding XApiStep shows OAuth connect button after Client ID entry
- OAuth flow completes and shows connected avatar + username in onboarding UI
