# Session 03 Handoff

## What Changed

Built the profile-analysis pipeline that reads the connected X account's profile and recent tweets, runs a two-pass inference pipeline (deterministic heuristics → optional LLM enrichment), and returns normalized `InferredProfile` suggestions with confidence and provenance metadata.

### New Files

| File | Purpose |
|------|---------|
| `crates/tuitbot-core/src/toolkit/profile_inference/mod.rs` | Types: `Confidence`, `Provenance`, `InferredField<T>`, `InferredProfile`, `ProfileInput`, `compute_base_confidence()` |
| `crates/tuitbot-core/src/toolkit/profile_inference/heuristics.rs` | Deterministic extraction: account type detection, product name/description/URL extraction, hashtag-based keywords |
| `crates/tuitbot-core/src/toolkit/profile_inference/llm_enrichment.rs` | LLM enrichment: prompt building, JSON response parsing (with markdown fence stripping), confidence-aware merge into heuristic baseline |
| `crates/tuitbot-core/src/toolkit/profile_inference/tests.rs` | 18 unit tests covering rich/sparse/individual/business profiles, confidence computation, LLM response parsing, serialization |
| `crates/tuitbot-server/tests/onboarding_analysis.rs` | 6 integration tests: rich profile, sparse profile, no tweets, low confidence, serialization roundtrip, response shape |
| `docs/roadmap/x-profile-prefill-onboarding/session-03-handoff.md` | This handoff |

### Modified Files

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/toolkit/mod.rs` | Added `pub mod profile_inference;` |
| `crates/tuitbot-server/src/routes/onboarding.rs` | Added `POST /api/onboarding/analyze-profile` handler with `AnalyzeProfileRequest` and `LlmConfigInput` types |
| `crates/tuitbot-server/src/lib.rs` | Wired `/onboarding/analyze-profile` route |
| `crates/tuitbot-server/src/auth/middleware.rs` | Added 2 auth exemptions for analyze-profile endpoint |
| `dashboard/src/lib/api/types.ts` | Added `Confidence`, `InferenceProvenance`, `InferredField<T>`, `InferredProfile`, `AnalyzeProfileResponse` types |
| `dashboard/src/lib/api/client.ts` | Added `api.onboarding.analyzeProfile()` method |
| `dashboard/src/lib/stores/onboarding-session.ts` | Added `inferred_profile`, `analyzing`, `analysis_warnings` fields + `setAnalyzing()`, `setInferredProfile()` helpers |

## Key Decisions Made

### D1: Two-pass architecture (heuristics first, LLM second)
Heuristics extract `account_type`, `product_name`, `product_description`, `product_url`, and `product_keywords` from bio text and profile fields. LLM enrichment (optional) upgrades `target_audience`, `industry_topics`, `brand_voice`, and refines other fields. If no LLM is configured, the endpoint returns `status: "partial"` with heuristic-only results.

### D2: Module placed in `toolkit/` layer (not `workflow/`)
Profile inference is a stateless function: `(User, Vec<Tweet>, Option<LlmProvider>) → InferredProfile`. No DB dependency, no persistence. This matches the toolkit layer's charter per the dependency rule.

### D3: No persistence of inference results
The `InferredProfile` lives only in the API response and the client-side `onboarding-session` store. Re-calling the endpoint re-analyzes from fresh X data. Final values are persisted only after user review (Session 04 concern).

### D4: LLM config comes from the request body
The analyze endpoint accepts optional `{ llm: { provider, api_key?, model, base_url? } }` in the request body. This decouples it from server-side config that may not exist during onboarding. If absent, heuristic-only analysis runs.

### D5: Sparse-account fallback strategy
- **No bio + no tweets**: All defaults with `confidence: low`, `provenance: default`. Status: `partial`.
- **Bio only**: Heuristic extraction from bio. `brand_voice` and `industry_topics` empty with low confidence.
- **Tweets only**: LLM enrichment uses tweet text. `product_url` defaults to null.
- **Auth failure/expired tokens**: Returns `status: "x_api_error"` immediately.

### D6: Tweet fetch failure is non-fatal
If `get_user_tweets` fails (rate limit, permissions), the endpoint continues with profile-only data and adds a warning. This prevents tweet API issues from blocking the entire flow.

### D7: OnceLock for regex patterns (MSRV 1.75 compatibility)
`std::sync::LazyLock` requires Rust 1.80+. Used `OnceLock` pattern consistent with the rest of the codebase.

## API Contract

### `POST /api/onboarding/analyze-profile`
**Auth:** Exempt (pre-account endpoint)
**Request:**
```json
{
  "llm": {
    "provider": "openai",
    "api_key": "sk-...",
    "model": "gpt-4o-mini",
    "base_url": null
  }
}
```
(The `llm` field is optional; send `null` or omit for heuristic-only analysis.)

**Success Response:**
```json
{
  "status": "ok",
  "profile": {
    "account_type": { "value": "business", "confidence": "high", "provenance": "bio_and_tweets" },
    "product_name": { "value": "Acme Tools", "confidence": "high", "provenance": "bio" },
    "product_description": { "value": "Developer productivity suite", "confidence": "high", "provenance": "bio_and_tweets" },
    "product_url": { "value": "https://acme.dev", "confidence": "high", "provenance": "profile_url" },
    "target_audience": { "value": "Software engineers", "confidence": "high", "provenance": "bio_and_tweets" },
    "product_keywords": { "value": ["devtools", "productivity", "api"], "confidence": "high", "provenance": "bio_and_tweets" },
    "industry_topics": { "value": ["developer tools", "SaaS", "open source"], "confidence": "medium", "provenance": "bio_and_tweets" },
    "brand_voice": { "value": "professional", "confidence": "medium", "provenance": "tweets" }
  },
  "warnings": []
}
```

**Partial Response (no LLM):**
```json
{
  "status": "partial",
  "profile": { ... },
  "warnings": ["No LLM configured. Using heuristic analysis only (limited accuracy)."]
}
```

**Error Response:**
```json
{
  "status": "x_api_error",
  "error": "X tokens expired. Please re-authenticate."
}
```

## Open Issues

1. **Analyze endpoint is unauthenticated** — Anyone with access to the server can call it if `onboarding_tokens.json` exists. Low risk (tokens are short-lived, endpoint is read-only), but consider IP rate limiting in future.

2. **Hashtag-only keyword extraction** — Heuristic keywords come only from hashtags. Many profiles and tweets don't use hashtags. LLM enrichment is the primary source of quality keywords.

3. **No token refresh in analyze endpoint** — If the access token is about to expire but hasn't yet, the X API calls may fail mid-flight. The endpoint returns `x_api_error` and the user can re-authenticate. A future improvement could use the refresh token.

4. **Stale onboarding tokens cleanup** — Carried from Session 02. Still not implemented; tokens expire in 2 hours.

## What Session 04 Must Do

### Mission
Build the single-screen profile review UI that consumes the analysis API and lets users edit inferred values before completing setup.

### How to Consume the API

1. **Trigger analysis** after X OAuth completes:
   ```typescript
   onboardingSession.setAnalyzing(true);
   const result = await api.onboarding.analyzeProfile(llmConfig);
   if (result.profile) {
     onboardingSession.setInferredProfile(result.profile, result.warnings ?? []);
   }
   ```

2. **Read from store**:
   ```typescript
   $onboardingSession.inferred_profile  // InferredProfile | null
   $onboardingSession.analyzing         // boolean
   $onboardingSession.analysis_warnings // string[]
   ```

3. **Field mapping** — Each `InferredField` has:
   - `.value` — pre-fill the form field
   - `.confidence` — show badge (high=green, medium=yellow, low=gray)
   - `.provenance` — tooltip ("Inferred from bio", "Inferred from tweets", etc.)
   - All fields are editable — inference is a suggestion, not a commitment

4. **Fallback** — If `status === "partial"` or `status === "x_api_error"`, show manual entry (existing BusinessStep behavior) with a notice explaining why prefill didn't work.

5. **LLM config** — If the user has already entered LLM settings in an earlier onboarding step, pass them to `analyzeProfile()`. Otherwise pass `null` for heuristic-only analysis.

### Files to Create/Modify

| File | Change |
|------|--------|
| `dashboard/src/lib/components/onboarding/ProfileReviewStep.svelte` | New component: editable prefill review with confidence badges per field |
| `dashboard/src/routes/onboarding/+page.svelte` | Integrate ProfileReviewStep into the step flow, auto-trigger analysis after OAuth |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | Add auto-trigger for `analyzeProfile()` after successful connection |

### Verification

After Session 04:
- `cargo fmt --all && cargo fmt --all --check` passes
- `RUSTFLAGS="-D warnings" cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cd dashboard && npm run check` passes
- Profile analysis auto-triggers after X OAuth and shows prefilled fields
- All inferred fields are editable on a single review screen
- Low-confidence and partial-analysis cases show appropriate UI warnings
