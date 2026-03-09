# Session 05 Handoff

## What Changed

Implemented progressive activation so the onboarding wizard no longer blocks on every advanced credential. Users can now complete a short X-first setup and enter a safe starter mode without filling every advanced field. Missing capabilities are represented as explicit, resumable state.

### New Files

| File | Purpose |
|------|---------|
| `crates/tuitbot-core/src/config/capability.rs` | `CapabilityTier` enum (5 tiers), `compute_tier()` function, tier metadata methods (`label()`, `description()`, `missing_for_next()`), 11 unit tests |
| `dashboard/src/lib/stores/capability.ts` | Frontend derived stores: `capabilityTier`, `canExplore`, `canGenerate`, `canPublish`, plus `tierRank()` and `tierLabel()` helpers |
| `docs/roadmap/x-profile-prefill-onboarding/progressive-activation.md` | Design doc for the progressive activation model |
| `docs/roadmap/x-profile-prefill-onboarding/session-05-handoff.md` | This handoff |

### Modified Files

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/config/mod.rs` | Added `pub mod capability;` and re-export of `CapabilityTier` and `compute_tier` |
| `crates/tuitbot-core/src/config/validation.rs` | Added `validate_minimum()` method — checks only profile fields and structural requirements, skips LLM and X API checks |
| `crates/tuitbot-server/src/routes/settings.rs` | Changed `init_settings` to use `validate_minimum()` instead of `validate()`; added `capability_tier` to `config_status` response |
| `crates/tuitbot-server/src/routes/runtime.rs` | Added `capability_tier` to runtime status response |
| `dashboard/src/lib/api/types.ts` | Added `CapabilityTier` type alias; added `capability_tier` field to `RuntimeStatus` and `ConfigStatus` interfaces |
| `dashboard/src/lib/stores/runtime.ts` | Extended `RuntimeCapabilities` interface and all data paths to include `capability_tier` |
| `dashboard/src/lib/stores/onboarding.ts` | Added `isMinimalComplete()` export |
| `dashboard/src/routes/onboarding/+page.svelte` | Refactored step flow: LLM is now optional, "Skip optional steps" button after Profile, skipped steps tracked and shown with dashed progress dots, submit omits LLM section when unconfigured, API mode skips Analyze when LLM not configured |
| `dashboard/src/lib/components/onboarding/ValidationStep.svelte` | Accepts `hasLlmConfig` prop; shows info card when LLM missing instead of failing test; only runs test when LLM is configured |
| `dashboard/src/lib/components/onboarding/ReviewStep.svelte` | Accepts `skippedSteps` prop; shows tier indicator card (amber for deferred, green for full); each section shows "Configured" or "Set up later" badges; deferred items listed explicitly |

## Key Decisions Made

### D1: Tiers are computed, never stored
`compute_tier(config, can_post)` is a pure function called on every request. No database column or file field for the tier. This prevents stale-state bugs when config changes outside the app.

### D2: `validate_minimum()` is additive — `validate()` unchanged
The existing `validate()` method is untouched. `validate_minimum()` only relaxes LLM API key and X API client_id requirements. All structural validations (db_path, schedule, content sources) remain.

### D3: LLM step is optional in both modes
In API mode, skipping LLM also skips the Analyze step (no LLM means no profile analysis). In scraper mode, LLM was already after Profile so this is natural.

### D4: Submit omits LLM section when not configured
When `llm_provider` is empty or has no API key (non-ollama), the `submit()` function omits the `llm` section entirely from the init payload. The server's `validate_minimum()` accepts this.

### D5: ReviewStep computes tier client-side for display
Rather than making an extra API call, the ReviewStep derives the expected tier from the onboarding store state. This gives immediate feedback without a round-trip.

### D6: Skipped steps use dashed borders in progress dots
Visual distinction between completed (green check), skipped (dashed border, dash mark), and pending (solid border, number) steps. This keeps the progress bar informative without being confusing.

## Capability Tier Model

| Tier | Name | Required | Unlocks |
|------|------|----------|---------|
| 0 | `unconfigured` | Nothing | Redirect to onboarding |
| 1 | `profile_ready` | Business profile | Dashboard, settings |
| 2 | `exploration_ready` | + X credentials | Discovery, scoring |
| 3 | `generation_ready` | + LLM config | AI drafts, composition |
| 4 | `posting_ready` | + posting tokens | Autopilot, scheduling |

## Open Issues

1. **No dashboard tier prompt yet** — After onboarding completes with deferred capabilities, the dashboard doesn't yet show prompts to complete setup. Session 06 should add first-run guidance.

2. **No re-analysis after LLM setup in Settings** — If a user skips LLM during onboarding and later configures it in Settings, there's no way to trigger profile analysis from Settings. Consider adding an "Analyze Profile" button to the Settings page.

3. **BusinessStep.svelte still unused** — Carried from Session 04. Can be deleted in cleanup.

4. **Carried from Session 03/04**: Analyze endpoint is unauthenticated, no token refresh, stale onboarding tokens cleanup.

5. **Tier-gated UI not implemented** — The `canExplore`, `canGenerate`, `canPublish` stores exist but no dashboard components use them yet to gate features or show upgrade prompts.

## What Session 06 Must Do

### Mission
Build first-run guidance and dashboard tier awareness so users who completed the shortened onboarding know what to do next and can progressively unlock capabilities.

### Key Points
- After onboarding submit, users land at `/content?compose=true`. Session 06 should add empty states and tier-aware prompts.
- The capability stores (`canExplore`, `canGenerate`, `canPublish`) are ready for use in dashboard components.
- Consider adding "Complete Setup" cards or banners that link to Settings for each missing capability.
- The `compute_tier()` function on the server and `tierLabel()`/`tierRank()` on the frontend are ready for use.

### Verification
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
cd dashboard && npm run build
```
