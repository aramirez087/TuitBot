# Session 08 Handoff

## What Changed

Instrumented the onboarding funnel end-to-end, hardened failure states across the wizard and analysis flow, swept copy for consistency with the product context, and produced measurement and experiment documentation.

### New Files

| File | Purpose |
|------|---------|
| `dashboard/src/lib/analytics/funnel.ts` | Lightweight funnel event tracking utility — logs structured JSON to `console.info('[tuitbot:funnel]', ...)` |
| `docs/roadmap/x-profile-prefill-onboarding/funnel-metrics.md` | Funnel stages, key activation metrics, drop-off points, measurement method |
| `docs/roadmap/x-profile-prefill-onboarding/experiment-backlog.md` | 10 prioritized experiments (E1–E10) for funnel optimization |
| `docs/roadmap/x-profile-prefill-onboarding/session-08-handoff.md` | This handoff |

### Modified Files

| File | Changes |
|------|---------|
| `dashboard/src/routes/onboarding/+page.svelte` | Funnel events (started, step-entered, step-skipped, submitted, completed, error, 409-recovery); deployment mode guard; network error handling with user-friendly message; error retry button |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | Events for x-auth-started, x-auth-success, x-auth-error, scraper-selected; timeout tracking |
| `dashboard/src/lib/components/onboarding/ProfileAnalysisState.svelte` | Events for analysis-started, analysis-success, analysis-error, analysis-skipped; error differentiation (x_api_error vs llm_error vs network); sparse profile notice; copy update |
| `dashboard/src/lib/components/onboarding/PrefillProfileForm.svelte` | Profile-edited events (once per field); low-confidence dashed border styling; manual entry mode copy when all fields are low confidence |
| `dashboard/src/lib/components/onboarding/WelcomeStep.svelte` | Copy rewrite: removed marketing-speak ("autonomous growth co-pilot"), updated features list, changed hint to reflect optional LLM |
| `dashboard/src/lib/components/onboarding/ReviewStep.svelte` | Copy update: "Review your setup. You can change any of these in Settings later."; fixed Generation Ready tier description |
| `dashboard/src/lib/components/onboarding/ActivationChecklist.svelte` | Events for checklist-viewed, checklist-item-clicked, checklist-dismissed, tier-changed |
| `dashboard/src/lib/stores/capability.ts` | Updated checklist item descriptions to be action-oriented |
| `dashboard/src/routes/(app)/content/+page.svelte` | First-draft-created event tracking |
| `docs/getting-started.md` | Updated Desktop section to mention profile pre-fill; noted LLM is optional during onboarding; added progressive enrichment note about dashboard onboarding |

## Key Decisions Made

### D1: Client-side console.info tracking, no external SDK
Tuitbot is self-hosted/desktop with no cloud telemetry. Events go to `console.info('[tuitbot:funnel]', JSON.stringify(event))` — grep-able from Tauri logs or DevTools. No bundle size impact from analytics SDKs.

### D2: Event taxonomy uses `domain:action` kebab-case
All events follow `onboarding:*` or `activation:*` naming convention with structured JSON properties.

### D3: Network errors get user-friendly messages
`TypeError: Failed to fetch` in submit() shows "Can't reach the Tuitbot server" instead of the raw error. Same pattern in ProfileAnalysisState.

### D4: LLM errors auto-continue with partial results
When analysis returns `llm_error` status but has partial profile data, the flow auto-advances rather than blocking. Users get whatever was extracted from the X API alone.

### D5: Low-confidence fields use dashed amber border
Only Low confidence fields get visual treatment (dashed border). Medium and High are neutral. This avoids cognitive overload while flagging fields that need attention.

### D6: Deployment mode guard added to onboarding
On mount, the wizard checks `configStatus()`. If already configured, redirects to `/`. If deployment mode is unrecognized, shows a fallback message directing users to `tuitbot init`.

### D7: Error banner includes retry button
The error banner in the onboarding wizard now has a Retry button that re-calls `submit()`. The existing 409 recovery handles double-submit safely.

## Event Inventory

### Onboarding Events
| Event | Properties |
|-------|------------|
| `onboarding:started` | `mode` |
| `onboarding:step-entered` | `step`, `index` |
| `onboarding:step-skipped` | `from_step`, `skipped[]` |
| `onboarding:x-auth-started` | `mode` |
| `onboarding:x-auth-success` | `username` |
| `onboarding:x-auth-error` | `error` |
| `onboarding:scraper-selected` | — |
| `onboarding:analysis-started` | `has_llm` |
| `onboarding:analysis-success` | `status`, `fields_inferred`, `warnings` |
| `onboarding:analysis-error` | `error` |
| `onboarding:analysis-skipped` | `had_error` |
| `onboarding:profile-edited` | `field` |
| `onboarding:submitted` | `has_x_auth`, `has_llm`, `has_vault`, `tier` |
| `onboarding:completed` | `tier`, `claimed` |
| `onboarding:error` | `error`, `step` |
| `onboarding:409-recovery` | `reason` |

### Activation Events
| Event | Properties |
|-------|------------|
| `activation:checklist-viewed` | `tier`, `completed`, `total` |
| `activation:checklist-item-clicked` | `item_id`, `completed` |
| `activation:checklist-dismissed` | `tier` |
| `activation:tier-changed` | `from`, `to` |
| `activation:first-draft-created` | `source` |

## Open Issues

1. **No re-analysis trigger** — When a user configures LLM in Settings after skipping during onboarding, there's no "Analyze Profile" button. Carried from Session 05/06. See experiment E3.

2. **Tier-gated empty states not implemented** — Discovery, Targets, and other pages don't yet show tier-aware empty states. Carried from Session 06.

3. **No first-draft experience on home page** — DraftStudioShell doesn't have a first-run empty state. Carried from Session 06.

4. **Checklist deep-link scroll verification** — Hash-anchor scrolling may not work in SPA routing. Carried from Session 06.

5. **Stale onboarding tokens cleanup** — Old `onboarding_tokens.json` files from abandoned flows are never cleaned up. Carried from Session 03/04.

6. **Analyze endpoint unauthenticated** — Carried from Session 03/04.

7. **BusinessStep.svelte still unused** — Carried from Session 04. Can be deleted in cleanup.

8. **Server-side telemetry endpoint** — Currently events are client-side only. A `POST /api/telemetry/funnel` endpoint would enable persistent storage and dashboarding. Post-epic scope.

## What Session 09 Should Do

### Validation & Polish
1. **Manual funnel walkthrough** — Complete the full onboarding flow while monitoring `[tuitbot:funnel]` events in DevTools. Verify every stage fires, properties are correct, and timestamps are sequential.
2. **Error state testing** — Simulate: server down during submit, OAuth window closed, expired tokens during analysis, empty X profile (no bio, no tweets).
3. **Copy review at 320px** — Verify all copy changes render correctly at narrow mobile viewports.
4. **Checklist deep-link scroll** — Test hash-anchor navigation from checklist items to Settings sections.

### Remaining Feature Work
5. **"Analyze Profile" button in Settings** — Add server endpoint and Settings UI for re-triggering profile analysis when LLM is configured post-onboarding.
6. **Tier-gated empty states** — Discovery, Targets pages show helpful prompts when user lacks the required tier.
7. **First-draft experience** — DraftStudioShell guides `profile_ready` users toward their first action.

### Cleanup
8. **Delete unused BusinessStep.svelte**
9. **Stale onboarding tokens cleanup** — Add periodic or on-init cleanup of abandoned `onboarding_tokens.json` files.
10. **Analyze endpoint auth** — Add authentication to the analyze-profile endpoint.

### Verification Matrix

| Scenario | Expected Behavior | Event(s) |
|----------|-------------------|----------|
| Complete happy path (API mode + LLM) | All steps, analysis, review, submit | started → step-entered (each) → x-auth-success → analysis-success → submitted → completed |
| Complete happy path (scraper mode) | Skip analysis, manual profile | started → scraper-selected → submitted → completed |
| Skip optional steps | Jump to Review | step-skipped with skipped[] |
| OAuth timeout | Error after 5min, retry available | x-auth-error(timeout) |
| Server down during submit | User-friendly error + retry button | error(network) |
| Double submit (409) | Silent redirect to / | 409-recovery(already_exists) |
| Sparse X profile | Sparse notice, manual entry mode | analysis-success(status:partial) |
| LLM error during analysis | Auto-continue with partial data | analysis-success(status:llm_error_partial) |
| First draft created | Tracked on content page | first-draft-created |
| Tier change | Logged when capability tier updates | tier-changed(from, to) |
