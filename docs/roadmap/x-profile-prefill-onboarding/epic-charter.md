# Epic Charter: X Profile Prefill Onboarding

## Problem Statement

The current onboarding wizard requires users to manually type 6+ business profile fields (name, description, audience, keywords, topics) before Tuitbot delivers any value. Users must also create an X Developer App and paste a Client ID without immediate feedback. The combined friction produces abandonment before first value.

## Goal

Reduce time-to-first-value by connecting the user's X account early, fetching their profile and recent tweets, and using LLM inference to prefill the business profile. Every inferred value remains editable so the user stays in control.

## Scope

- **In scope:** Desktop and SelfHost onboarding flows in the dashboard UI, server endpoints for profile analysis, `tuitbot-core` inference module, extended X API `User` type.
- **Future-compatible:** Cloud mode can skip Client ID entry (managed by platform) and ClaimStep (managed auth), but the profile inference flow is identical. No cloud-specific code ships in this epic.
- **Out of scope:** CLI onboarding (`tuitbot init`), post-onboarding settings pages, any changes to approval-mode safeguards or auth protections.

## Shared vs Divergent Steps

| Step | Desktop | SelfHost | Notes |
|------|---------|----------|-------|
| Welcome | Shared | Shared | Updated copy mentioning X sign-in |
| X Access + Auth | Shared | Shared | Client ID entry + OAuth PKCE sign-in |
| LLM Setup | Shared | Shared | Moved before analysis to power inference |
| Profile Analysis | Shared | Shared | Server fetches profile + tweets, runs LLM inference |
| Editable Prefill | Shared | Shared | Single-screen review of all inferred fields |
| Language & Brand | Shared | Shared | `brand_voice` may be pre-suggested from tweet tone |
| Content Sources | Shared | Shared | Defaults differ by capability (local_fs vs google_drive), already handled |
| Validation | Shared | Shared | LLM connection test |
| Review | Shared | Shared | Summary + approval_mode toggle |
| Secure (ClaimStep) | Skipped | Present | Web-only passphrase setup, already gated by `!isTauri` |

**Divergence summary:** Only ClaimStep diverges (web-only, already implemented). Content source defaults are capability-gated (already works). All new work is shared.

## Architecture Decisions

### AD-1: X Sign-In Becomes the Second Step

Move OAuth PKCE sign-in into the X Access step. After entering Client ID, the user immediately clicks "Connect Account" which opens the OAuth flow. On success, the UI shows their avatar + @username as confirmation.

**Rationale:** Sign-in gives us identity + profile data for prefill. The current flow asks for Client ID but doesn't use it until after onboarding completes.

### AD-2: LLM Setup Moves Before Profile Analysis

Reorder the wizard so LLM configuration (step 3) comes before profile analysis (step 4). The inference prompt needs an LLM to run.

**Rationale:** Without an LLM, we cannot run structured inference. Heuristic-only extraction is too fragile for production quality. The step ordering becomes: Welcome > X Access+Auth > LLM > Analyze > Review Prefill > Language/Brand > Sources > Validate > Review > Secure.

**Fallback:** If user testing shows LLM-first is confusing, implement a two-pass approach: heuristic extraction from bio fields first (no LLM needed), then offer "re-analyze with AI" after LLM is configured.

### AD-3: Extend `User` Type with Bio Fields

Add `description`, `location`, `url`, `entities` to `x_api::types::User` and update `USER_FIELDS` constant to request `description,location,url,entities` from the X API v2 `/users/me` endpoint.

**Current state:** `USER_FIELDS = "username,name,public_metrics,profile_image_url"` in `crates/tuitbot-core/src/x_api/client/mod.rs:34`. The `User` struct has only `id`, `username`, `name`, `profile_image_url`, `public_metrics`.

**Rationale:** The bio is the primary input for inferring product_description, target_audience, and keywords. These fields are available from X API v2 when requested.

### AD-4: Profile Inference Lives in `tuitbot-core::workflow`

Create `core::workflow::profile_inference` module that takes a `User` (with bio) + recent tweets and calls the LLM with a structured prompt, returning `InferredProfile` with per-field confidence and provenance.

**Rationale:** Per `docs/architecture.md`, workflow modules combine toolkit functions with DB and LLM. Profile inference is a stateful composite operation — it calls toolkit (`get_me`, `get_user_tweets`) and uses LLM. It belongs in Layer 2.

### AD-5: Single Server Endpoint for Analysis

New `POST /api/onboarding/analyze-profile` endpoint that:
1. Uses OAuth tokens from the just-completed auth flow
2. Calls `get_me` to fetch profile with expanded fields
3. Calls `get_user_tweets` for recent 10-20 tweets
4. Runs `profile_inference` with the configured LLM
5. Returns `InferredProfile` JSON

**Rationale:** Keeps the server thin (routing + delegation). A single endpoint avoids exposing raw tweet data to the frontend and reduces round trips.

### AD-6: Single-Screen Editable Prefill

Replace the multi-field BusinessStep with a single review screen showing all inferred fields. Each field displays its inferred value (editable), confidence indicator (high/medium/low), and source provenance ("from bio", "from tweets", "inferred").

**Rationale:** Users should see all inferences at once and correct anything wrong, rather than stepping through fields one by one. This also reduces the step count.

### AD-7: Graceful Degradation for Scraper Mode

For scraper/no-key mode users, profile inference is unavailable (no OAuth = no `get_me`). They get the current manual entry flow as fallback. The UI detects `provider_backend === 'scraper'` and skips analysis steps.

**Rationale:** No regression for scraper users. They see the same flow they have today.

## Non-Goals

- No changes to CLI onboarding (`tuitbot init`)
- No changes to post-onboarding settings screens
- No cloud mode implementation
- No changes to approval-mode safeguards, auth protections, or deployment capability gating
- No changes to account isolation rules

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| X API rate limits during onboarding | Analysis fails, user stuck | Low (single user, fresh tokens) | Cache profile data; graceful fallback to manual entry |
| LLM unavailable when inference runs | No prefill | Medium (user may misconfigure) | LLM health check runs before analysis; clear error directing user to fix LLM config |
| Bio too short or empty | Poor inference quality | Medium | Low-confidence badges on affected fields; manual entry fallback for empty fields |
| User has no tweets | No tweet-based inference | Low-Medium | Bio-only inference; confidence downgraded; keywords/topics flagged for manual entry |
| OAuth tokens expire during analysis | API calls fail | Very Low (tokens just issued) | Tokens are fresh from just-completed OAuth; no mitigation needed |
| Scraper users feel left out | Perception of two-tier experience | Low | Clear messaging about API mode benefits; manual entry is identical to current flow |
| LLM step reorder confuses users | Drop-off at LLM step | Medium | Copy explains "we need AI to analyze your profile"; fallback AD-2 available |

## Session Breakdown

| Session | Title | Scope |
|---------|-------|-------|
| 1 | Charter & Target Flow | This session — planning documents |
| 2 | Unified Entry & X Auth Bootstrap | Extend `User` type, update `USER_FIELDS`, wire onboarding OAuth flow, frontend combined ClientID + OAuth step |
| 3 | Profile Bootstrap & Analysis | `core::workflow::profile_inference` module, LLM prompt, `POST /api/onboarding/analyze-profile` endpoint, integration tests |
| 4 | Single-Screen Onboarding UI | `ProfileReviewStep.svelte`, onboarding store inference state, wire analyze endpoint, scraper mode degradation |
| 5 | Progressive Activation & Gating | Step reordering (LLM before analysis), deployment mode branching validation, end-to-end Desktop vs SelfHost vs scraper flows |
| 6 | First-Run Checklist & Unlocks | Post-onboarding activation state, first-value guidance UI |
| 7 | Provisioning & First Value Launch | Config materialization from inferred+edited profile, runtime start verification |
| 8 | Measurement, Copy & Fallbacks | Funnel instrumentation, error handling, fallback flows, copy polish |
| 9 | End-to-End Validation & Launch | Integration tests, manual QA checklist, launch readiness doc |

## Test Strategy

- **Unit tests:** `profile_inference` module with fixture bio/tweet data covering: business account, individual account, empty bio, no tweets, non-English content.
- **Integration tests:** `analyze-profile` endpoint with mocked X API responses and mocked LLM provider.
- **Frontend tests:** Onboarding store state transitions, inference field rendering, confidence badge display.
- **Manual QA matrix:** Desktop OAuth flow, SelfHost OAuth flow, scraper fallback, LLM failure fallback, empty bio handling, non-Latin bio handling.
