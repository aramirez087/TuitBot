# Session 04 Handoff

## What Changed

Replaced the multi-step onboarding wizard's Profile step with a single-screen editable prefill experience. After X OAuth and LLM setup, users see all inferred fields pre-populated with confidence/provenance metadata. The step flow is now mode-aware: API mode inserts an analysis step that auto-triggers profile inference before the profile form; scraper mode skips analysis and shows a manual-entry form.

### New Files

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/onboarding/ProfileAnalysisState.svelte` | Loading screen for the Analyze step: animated 3-phase progress UI, calls `analyzeProfile()` on mount, handles success/partial/error, auto-advances on completion |
| `dashboard/src/lib/components/onboarding/PrefillProfileForm.svelte` | Single-screen editable form replacing BusinessStep: shows confidence dots + provenance labels per field when inference data exists, connected-account card, warning/info banners |
| `docs/roadmap/x-profile-prefill-onboarding/session-04-handoff.md` | This handoff |

### Modified Files

| File | Change |
|------|--------|
| `dashboard/src/lib/stores/onboarding.ts` | Added `prefillFromInference(profile)` method that maps `InferredProfile` into store fields with brand_voice mapping (professional→balanced, casual→bold, formal→conservative, witty→bold) |
| `dashboard/src/routes/onboarding/+page.svelte` | Rewrote step flow to use step-name-based routing instead of index-based; conditional step list (API mode adds LLM→Analyze→Profile, scraper mode keeps Profile→LLM); hides Next button and Back during Analyze; skips Analyze step when navigating back from Profile |

## Key Decisions Made

### D1: Step-name-based routing instead of index-based
The `currentStepName` derivation (`steps[currentStep]`) drives all step rendering and `canAdvance()` logic via string switch/case. This eliminates brittle index arithmetic when steps shift between API and scraper modes.

### D2: Single PrefillProfileForm for both inferred and manual modes
Rather than maintaining BusinessStep (manual) and a separate review component (inferred), one `PrefillProfileForm` handles both. When no inference data exists, confidence badges and provenance tags don't render — the form behaves like the old BusinessStep.

### D3: Analyze step is hidden from the progress bar
The Analyze step appears in the step array for routing purposes but is excluded from the progress dots (filtered out in the `{#each}` loop). During analysis, the Profile dot shows as active. This prevents visual clutter from a transient auto-advancing step.

### D4: Back from Profile skips Analyze
When navigating back from Profile in API mode, `back()` decrements by 2 (skipping the Analyze step) to land on LLM. This prevents the user from re-triggering analysis unnecessarily.

### D5: Brand voice mapping lives in the store
The `prefillFromInference()` method on the `onboardingData` store handles the brand_voice mapping (professional→balanced, casual→bold, formal→conservative, witty→bold, fallback→balanced). This centralizes the mapping logic and keeps components simple.

### D6: Prefill happens in ProfileAnalysisState, not PrefillProfileForm
`ProfileAnalysisState` calls `onboardingData.prefillFromInference(profile)` after receiving the API response, before auto-advancing. PrefillProfileForm just reads from `$onboardingData` (already populated) and `$onboardingSession.inferred_profile` (for metadata display only).

### D7: Derived field metadata avoids @const placement issues
Svelte 5 restricts `{@const}` to specific block contexts (`{#if}`, `{#each}`, etc.), not plain `<div>` elements. Field metadata (confidence color, provenance label) is computed as `$derived` variables in the script block instead.

## Step Flow

### API Mode
```
0: Welcome → 1: X Access → 2: LLM → 3: Analyze (auto) → 4: Profile → 5: Language → 6: Vault → 7: Validate → 8: Review [→ 9: Secure]
```
Progress bar shows: Welcome, X Access, LLM, Profile, Language, Vault, Validate, Review [, Secure]
(Analyze is hidden from progress dots but exists in routing)

### Scraper Mode
```
0: Welcome → 1: X Access → 2: Profile → 3: LLM → 4: Language → 5: Vault → 6: Validate → 7: Review [→ 8: Secure]
```
(Same as before Session 04, but BusinessStep replaced by PrefillProfileForm)

## Payload Contract

The form produces the same payload shape that `submit()` sends to `api.settings.init()`. No payload changes — `onboardingData` store fields are populated either from inference prefill or manual entry, and the submit function reads them identically.

## Open Issues

1. **BusinessStep.svelte retained but unused** — The old component still exists in the codebase. It's no longer imported anywhere. Can be deleted in a cleanup pass.

2. **Analyze step re-triggers on back+forward** — If a user navigates back past the Analyze step and forward again, analysis re-runs. The `done` state is component-local and resets on remount. This is intentional (fresh analysis) but could be optimized to skip if `$onboardingSession.inferred_profile` already exists.

3. **No LLM config forwarding for scraper mode** — Scraper mode puts LLM after Profile. If a scraper user goes back from LLM to Profile, there's no analysis to trigger. This is correct (scraper users don't have X OAuth tokens), but the info banner could be more specific.

4. **Carried from Session 03**: Analyze endpoint is unauthenticated, no token refresh, stale onboarding tokens cleanup.

## What Session 05 Must Do

### Mission
Build the progressive activation experience after onboarding completes — first-run guidance, compose drawer integration, and any remaining post-submit routing.

### Key Points
- After `submit()` succeeds, the user lands at `/content?compose=true`. Session 05 should add first-run guidance or empty states that orient the user.
- The onboarding payload is unchanged — `api.settings.init()` receives the same shape regardless of whether inference was used.
- `BusinessStep.svelte` can be deleted (no longer imported).
- Consider adding a "Re-analyze" button on the Profile step for users who want to retry with different LLM settings after going back to the LLM step.

### Verification
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
cd dashboard && npm run build
```
