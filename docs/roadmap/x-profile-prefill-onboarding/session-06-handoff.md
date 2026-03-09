# Session 06 Handoff

## What Changed

Built the post-onboarding activation checklist that shows users which capabilities are unlocked, which are deferred, and what the next best action is. The checklist reuses existing Settings sections via hash-anchor deep links instead of rebuilding credential flows.

### New Files

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/onboarding/ActivationChecklist.svelte` | Full and compact checklist component — tier badge, progress bar, next-step links, available-now actions |
| `docs/roadmap/x-profile-prefill-onboarding/activation-checklist.md` | Design doc for checklist model, items, filtering, deep-link strategy |
| `docs/roadmap/x-profile-prefill-onboarding/session-06-handoff.md` | This handoff |

### Modified Files

| File | Change |
|------|--------|
| `dashboard/src/lib/stores/capability.ts` | Added `tierColor()`, `ChecklistItem` interface, `TierAction` interface, `computeChecklistItems()`, `currentTierActions()` |
| `dashboard/src/routes/(app)/+page.svelte` | Renders `ActivationChecklist` above `DraftStudioShell` when tier < posting_ready |
| `dashboard/src/routes/(app)/content/+page.svelte` | Renders compact `ActivationChecklist` banner above calendar; removed `?compose=true` onboarding redirect handler and unused `page` import |
| `dashboard/src/routes/onboarding/+page.svelte` | Changed post-submit redirect from `/content?compose=true` to `/` |

## Key Decisions Made

### D1: Checklist items link to Settings sections via hash anchors
Each item links to `/settings#business`, `/settings#xapi`, `/settings#llm`, or `/settings#sources`. These IDs already exist on `<SettingsSection>` elements. No mini-wizards or duplicated credential flows.

### D2: Checklist state is derived, never persisted
Items compute from `capabilityTier` store. No localStorage, no API state. Auto-hides at tier 4. Follows Session 05's D1 principle (tiers are computed, never stored).

### D3: Dismissal resets on tier change
A `$state` boolean tracks per-session dismissal. An `$effect` resets it when `capabilityTier` changes, so users see the checklist again when they unlock a new tier.

### D4: Post-onboarding redirect changed to `/`
Users now land on the home page (where the full checklist is) instead of `/content?compose=true`. The `?compose=true` handler was removed from the content calendar since it's no longer needed.

### D5: Knowledge vault is always shown as optional
The vault item has `optional: true` and never blocks tier progression. It uses a sparkle icon and amber "optional" badge to distinguish from required items.

### D6: Compact vs full rendering via `compact` prop
Home page shows the full two-column card (next steps + available actions). Content calendar shows a slim one-line banner. Same component, different layouts.

## Capability Checklist Items

| Item | Completes at | Links to | Optional |
|------|-------------|----------|----------|
| Business profile | tier >= 1 | `/settings#business` | no |
| X credentials | tier >= 2 | `/settings#xapi` | no |
| LLM provider | tier >= 3 | `/settings#llm` | no |
| Posting access | tier >= 4 | `/settings#xapi` | no |
| Knowledge vault | never (always optional) | `/settings#sources` | yes |

## Open Issues

1. **No re-analysis trigger** — When a user configures LLM in Settings after skipping during onboarding, there's no "Analyze Profile" button. Carried from Session 05.

2. **Tier-gated empty states not implemented** — Discovery, Targets, and other pages don't yet show tier-aware empty states or prompts when the user lacks the required tier. Session 07 scope.

3. **No first-draft experience on home page** — DraftStudioShell doesn't have a first-run empty state that guides profile_ready users toward their first action. Session 07 scope.

4. **Carried from Session 03/04**: Analyze endpoint is unauthenticated, no token refresh, stale onboarding tokens cleanup.

5. **BusinessStep.svelte still unused** — Carried from Session 04. Can be deleted in cleanup.

6. **Measurement hooks not wired** — The checklist doesn't yet emit analytics events for item clicks or tier transitions. Measurement plan documented in `activation-checklist.md` for Session 08.

## What Session 07 Must Do

### Mission
Implement provisioning actions and first-value experiences so users at each tier have meaningful things to do, not just a checklist to stare at.

### Key Points
1. **Add "Analyze Profile" button to Settings** — When LLM is configured post-onboarding, allow triggering profile analysis from `BusinessProfileSection` or `LlmProviderSection`.
2. **Tier-gated empty states** — Discovery (`/discovery`), Targets (`/targets`), and other pages should show helpful empty states when the user lacks the required tier, with links to the relevant Settings section.
3. **First-draft experience** — DraftStudioShell should guide `profile_ready` users toward their first meaningful action (browse settings, view profile, etc.) instead of showing a blank draft interface.
4. **Checklist deep-link scroll verification** — Verify that navigating to `/settings#llm` from the checklist correctly scrolls to the LLM section. If native hash scrolling doesn't work due to SPA routing, add `scrollIntoView` logic.

### Verification
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
cd dashboard && npm run build
```
