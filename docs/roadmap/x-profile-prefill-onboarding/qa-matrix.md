# QA Validation Matrix — Session 09

Systematic code-path tracing of every onboarding and activation scenario. Validated by reading source files and confirming code paths exist with correct event properties.

## 1. Scenario Coverage

| # | Scenario | Code Path | Events Traced | Verdict |
|---|----------|-----------|---------------|---------|
| 1 | **Happy path: API + LLM** | Welcome → X Access (clientId entered) → Connect with X (OAuth poll) → LLM config → Analyze (auto-runs) → Profile (prefilled) → Language → Vault → Validate → Review → submit() | `started(api)` → `step-entered` × N → `x-auth-started` → `x-auth-success(username)` → `analysis-started(has_llm:true)` → `analysis-success(ok)` → `submitted(has_x_auth,has_llm,tier)` → `completed(tier,claimed)` | PASS |
| 2 | **Happy path: Scraper mode** | Welcome → X Access (select scraper) → Profile (manual) → LLM → Language → Vault → Validate → Review → submit() | `started(scraper)` → `scraper-selected` → `step-entered` × N → `submitted(...)` → `completed(...)` | PASS |
| 3 | **Skip optional steps** | From Profile or any optional step → `skipToFinish()` → Review | `step-skipped(from_step, skipped[])` fires with correct array of skipped step names | PASS |
| 4 | **OAuth timeout** | `startPolling()` → 5-minute `setTimeout` clears interval → sets error | `x-auth-error(timeout)` fires; error message shown with Retry button | PASS |
| 5 | **Server down during submit** | `submit()` → `TypeError: Failed to fetch` → friendly message + retry | `error(network, step:submit)` fires; error banner shows "Can't reach the Tuitbot server" with RefreshCw retry button | PASS |
| 6 | **Double submit (409)** | `submit()` → error message includes "already exists" → silent redirect | `409-recovery(already_exists)` fires; `onboardingData.reset()` + `goto('/')` | PASS |
| 7 | **Already-claimed race (409)** | `submit()` → "already claimed" → retry without claim → redirect to `/login` | `409-recovery(already_claimed)` fires; retries init without claim object; handles nested 409 | PASS |
| 8 | **Sparse X profile** | `analyzeProfile()` returns `status: "partial"` → `sparseProfile = true` | `analysis-success(status:partial)` fires; sparse notice rendered; profile form shows in normal mode | PASS |
| 9 | **LLM error during analysis** | `analyzeProfile()` returns `status: "llm_error"` with partial profile → auto-continue | `analysis-success(status:llm_error_partial)` fires with `fields_inferred` count; `oncomplete()` called after 600ms delay | PASS |
| 10 | **X API error during analysis** | `analyzeProfile()` returns `status: "x_api_error"` → error card shown | `analysis-error(x_api_error)` fires; retry + "Continue manually" buttons shown | PASS |
| 11 | **Network error during analysis** | `TypeError: Failed to fetch` in `runAnalysis()` → friendly message | `analysis-error("Can't reach the server...")` fires; retry + skip buttons shown | PASS |
| 12 | **Analysis skipped manually** | User clicks "Continue manually" → `continueManually()` | `analysis-skipped(had_error)` fires; `oncomplete()` called | PASS |
| 13 | **Profile field editing** | User types in any field → `trackFieldEdit()` → fires once per field | `profile-edited(field)` fires on first input per field; `editedFields` Set prevents duplicates | PASS |
| 14 | **First draft created** | Content page → `createDraftAndRedirect()` → checks `$calendarItems.length === 0` | `first-draft-created(source)` fires only when no existing calendar items; source can be `calendar-slot`, `calendar-day`, `calendar-button`, `calendar-empty` | PASS |
| 15 | **Tier change** | `$capabilityTier` changes → `$effect` detects `previousTier !== current` | `tier-changed(from, to)` fires; checklist dismissal resets | PASS |
| 16 | **Checklist viewed** | Home page mount → capabilities loaded → not fully activated | `checklist-viewed(tier, completed, total)` fires once per mount (viewTracked guard) | PASS |
| 17 | **Checklist item clicked** | User clicks a checklist item link | `checklist-item-clicked(item_id, completed)` fires | PASS |
| 18 | **Checklist dismissed** | User clicks X button | `checklist-dismissed(tier)` fires; `dismissed = true` hides card | PASS |
| 19 | **Deployment mode guard** | On mount → `configStatus()` → already configured → redirect to `/` | Redirect fires; unsupported modes show fallback banner with `tuitbot init` guidance | PASS |
| 20 | **Web mode claim step** | `showClaimStep = !isTauri` → Secure step appended | Passphrase ≥ 8 chars required; `beforeunload` guard prevents accidental navigation; claim included in submit payload | PASS |

## 2. Event Coverage Matrix

All 21 events from the Session 08 inventory verified against source code.

### Onboarding Events (16)

| Event | File | Line Range | Properties Verified |
|-------|------|-----------|---------------------|
| `onboarding:started` | `+page.svelte` | 88–90 | `mode: 'scraper' \| 'api'` — derived from `isScraperMode` |
| `onboarding:step-entered` | `+page.svelte` | 93–97 | `step: currentStepName`, `index: currentStep` |
| `onboarding:step-skipped` | `+page.svelte` | 190–193 | `from_step`, `skipped[]` array of optional step names |
| `onboarding:x-auth-started` | `XApiStep.svelte` | 68–70 | `mode: 'scraper' \| 'api'` |
| `onboarding:x-auth-success` | `XApiStep.svelte` | 95 | `username` from user response |
| `onboarding:x-auth-error` | `XApiStep.svelte` | 82, 118 | `error: msg \| 'timeout'` |
| `onboarding:scraper-selected` | `XApiStep.svelte` | 56 | No properties |
| `onboarding:analysis-started` | `ProfileAnalysisState.svelte` | 38 | `has_llm: boolean` |
| `onboarding:analysis-success` | `ProfileAnalysisState.svelte` | 73–76, 95–99 | `status`, `fields_inferred`, `warnings` |
| `onboarding:analysis-error` | `ProfileAnalysisState.svelte` | 62, 116 | `error: 'x_api_error' \| message` |
| `onboarding:analysis-skipped` | `ProfileAnalysisState.svelte` | 122 | `had_error: boolean` |
| `onboarding:profile-edited` | `PrefillProfileForm.svelte` | 72 | `field: string` (once per field) |
| `onboarding:submitted` | `+page.svelte` | 208–213 | `has_x_auth`, `has_llm`, `has_vault`, `tier` |
| `onboarding:completed` | `+page.svelte` | 301–304 | `tier`, `claimed: showClaimStep` |
| `onboarding:error` | `+page.svelte` | 316, 349 | `error`, `step` |
| `onboarding:409-recovery` | `+page.svelte` | 322, 328 | `reason: 'already_exists' \| 'already_claimed'` |

### Activation Events (5)

| Event | File | Line Range | Properties Verified |
|-------|------|-----------|---------------------|
| `activation:checklist-viewed` | `ActivationChecklist.svelte` | 44–51 | `tier`, `completed`, `total` |
| `activation:checklist-item-clicked` | `ActivationChecklist.svelte` | 72 | `item_id`, `completed` |
| `activation:checklist-dismissed` | `ActivationChecklist.svelte` | 67 | `tier` |
| `activation:tier-changed` | `ActivationChecklist.svelte` | 37 | `from`, `to` |
| `activation:first-draft-created` | `content/+page.svelte` | 64 | `source` |

**Result: 21/21 events implemented with correct properties.**

## 3. Contract Alignment (Frontend ↔ Server)

### Analyze Profile Response

| Field | Rust (onboarding.rs) | Frontend (ProfileAnalysisState.svelte) | Aligned? |
|-------|---------------------|---------------------------------------|----------|
| `status` | `"ok" \| "partial" \| "x_api_error" \| "llm_error"` | Checks all four statuses | YES |
| `profile` | `InferredProfile` (serde snake_case) | Accessed as `result.profile` with `InferredField<T>` shape | YES |
| `warnings` | `Vec<String>` | `result.warnings ?? []` | YES |
| `error` | `String` (on x_api_error) | Not used directly; status check sufficient | YES |

Note: Rust `analyze_profile` returns `"llm_error"` in warnings but status stays `"partial"` when LLM fails with a provider config error. The frontend checks `result.status === 'llm_error'` which handles the case where the `create_provider` call succeeds but `enrich_with_llm` fails. When `create_provider` itself fails, status remains `"partial"` with a warning — the frontend handles this via the non-`llm_error` path, auto-continuing to profile form. **Functionally correct.**

### Init Settings Payload

| Frontend Field (submit body) | Server Field (init_settings) | Handling |
|------------------------------|------------------------------|----------|
| `x_api.provider_backend: 'scraper'` | Parsed via `json_to_toml` → TOML | Config field |
| `x_api.client_id` | Config field | Config field |
| `x_api.client_secret` | Config field (optional) | Config field |
| `business.*` | Config fields | Config field |
| `llm.*` | Config fields (conditional) | Config field |
| `content_sources.*` | Config fields (conditional) | Config field |
| `approval_mode` | Config field | Config field |
| `x_profile` | Extracted before TOML conversion | `XProfileData` struct → `update_account()` |
| `claim` | Extracted before TOML conversion | `ClaimRequest` → `create_passphrase_hash()` |

**Extraction order confirmed:** `claim` removed first (line 200–205), then `x_profile` removed (line 207–213), remaining body converted to TOML. **Correct — no config pollution from non-config fields.**

### 409 Responses

| Condition | Server Response | Frontend Handling |
|-----------|----------------|-------------------|
| Config already exists | `ApiError::Conflict("configuration already exists...")` | Error message includes "already exists" → `goto('/')` |
| Instance already claimed | `ApiError::Conflict("instance already claimed")` | Error message includes "already claimed" → retry without claim → `goto('/login')` |

## 4. Desktop vs Self-Host Divergence Audit

| Aspect | Desktop (Tauri) | Self-Host (Web) | Source |
|--------|----------------|-----------------|--------|
| Claim step | Skipped (`showClaimStep = !isTauri`) | Shown as final step | `+page.svelte:47` |
| Mode selector | X API / Scraper toggle shown | Same | `XApiStep.svelte:137` |
| Cloud mode guard | N/A | Mode selector hidden if `isCloud` | `XApiStep.svelte:137` |
| Deployment mode guard | Checks `configStatus()` | Same | `+page.svelte:68–81` |
| Token migration | Same `onboarding_tokens.json` path | Same | `settings.rs:272–283` |
| X profile provisioning | Same `update_account()` call | Same | `settings.rs:286–303` |
| Auth middleware exemptions | Not applicable (bearer mode) | All onboarding routes exempt | `middleware.rs:58–65` |
| Post-submit redirect | `goto('/')` | `goto('/')` or `goto('/login')` if already claimed | `+page.svelte:306–310` |

**Divergence is intentional and limited to the Claim step.** All other behavior is shared.

## 5. Checklist Deep-Link Verification

### Section IDs in Settings Page

| Checklist `href` | Section Component | `id` Attribute | Found? |
|------------------|-------------------|----------------|--------|
| `/settings#business` | `BusinessProfileSection.svelte` | `id="business"` | YES |
| `/settings#xapi` | `XApiSection.svelte` | `id="xapi"` | YES |
| `/settings#llm` | `LlmProviderSection.svelte` | `id="llm"` | YES |
| `/settings#sources` | `ContentSourcesSection.svelte` | `id="sources"` | YES |

### SPA Hash Navigation Behavior

The settings page uses `IntersectionObserver` to track visible sections (`+page.svelte:77–98`) and has a `scrollToSection(id)` helper using `el.scrollIntoView()` (`+page.svelte:126–131`). The sidebar nav calls `scrollToSection()` for in-page navigation.

For cross-page navigation (e.g., from `/` to `/settings#xapi`), SvelteKit with `adapter-static` in SPA mode handles the hash after client-side routing completes. The browser's native hash-scroll behavior fires after the page renders. Since settings sections render synchronously after `loadSettings()` resolves, the target element will exist when the browser processes the hash.

**Verdict: Deep-links work for same-page navigation. Cross-page hash navigation relies on browser native behavior, which is reliable but may not scroll smoothly in all cases. A `$effect` that reads `window.location.hash` on mount and calls `scrollToSection()` would make this more robust — recommended as a P2 post-launch improvement.**

## 6. Error State Coverage

| Error Condition | Handler Location | User-Facing Message | Recovery Action | Event |
|----------------|-----------------|---------------------|-----------------|-------|
| Network failure (submit) | `+page.svelte:314–317` | "Can't reach the Tuitbot server..." | Retry button | `onboarding:error(network)` |
| Network failure (analysis) | `ProfileAnalysisState:111–112` | "Can't reach the server..." | Retry / Continue manually | `onboarding:analysis-error` |
| OAuth timeout (5 min) | `XApiStep:113–124` | "Connection timed out..." | Retry button | `onboarding:x-auth-error(timeout)` |
| OAuth error (generic) | `XApiStep:81–83` | Raw error message | Retry button | `onboarding:x-auth-error(msg)` |
| X API error (analysis) | `ProfileAnalysisState:60–65` | "Couldn't access your X account..." | Retry / Continue manually | `onboarding:analysis-error(x_api_error)` |
| LLM error (analysis) | `ProfileAnalysisState:67–83` | None (auto-continues) | N/A | `onboarding:analysis-success(llm_error_partial)` |
| Expired tokens (analysis) | `onboarding.rs:258–263` | "X tokens expired..." via status | Error card shown | `analysis-error(x_api_error)` |
| Config validation failure | `+page.svelte:290–292` | `field: message` joined | Fix fields and retry | N/A |
| Double submit (409) | `+page.svelte:321–326` | None (silent redirect) | N/A | `onboarding:409-recovery` |
| Already claimed (409) | `+page.svelte:327–347` | None (retry without claim) | N/A | `onboarding:409-recovery` |
| Unsupported deployment mode | `+page.svelte:356–370` | "This deployment mode is not supported" | Use `tuitbot init` CLI | N/A |

## 7. Auth Middleware Exemptions

All onboarding and setup endpoints are exempt from authentication:

| Endpoint | Exempt? | Rationale |
|----------|---------|-----------|
| `POST /api/onboarding/x-auth/start` | Yes | Pre-account OAuth initiation |
| `POST /api/onboarding/x-auth/callback` | Yes | OAuth code exchange |
| `GET /api/onboarding/x-auth/status` | Yes | Polling during OAuth flow |
| `POST /api/onboarding/analyze-profile` | Yes | Profile analysis before config exists |
| `GET /api/settings/status` | Yes | Config existence check |
| `POST /api/settings/init` | Yes | First-time config creation (409 if exists) |
| `POST /api/settings/test-llm` | Yes | LLM connectivity test during onboarding |

**Risk assessment:** The analyze-profile endpoint requires valid OAuth tokens stored in `onboarding_tokens.json` to call the X API. Without tokens, it returns `x_api_error`. Abuse potential is limited for self-hosted/desktop deployments where the attacker would need network access to the local server. **Acceptable for launch; add auth post-launch as a hardening measure.**
