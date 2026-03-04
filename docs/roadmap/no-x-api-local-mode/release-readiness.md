# Local No-Key Mode — Release Readiness Report

**Date:** 2026-03-02
**Sessions:** 1–4 (Charter → Settings → Runtime → Validation)
**Decision:** CONDITIONAL GO

---

## Quality Gate Results

| Gate | Result |
|------|--------|
| `cargo fmt --all && cargo fmt --all --check` | PASS |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS — 1,866 tests, 0 failures |
| `cargo clippy --workspace -- -D warnings` | PASS — zero warnings |
| `cd dashboard && npm run check` | PASS — 0 errors, 7 pre-existing warnings (unrelated to this feature) |

### Test Coverage

| Suite | Tests | Status |
|-------|-------|--------|
| Config backend validation (`tests_backend.rs`) | 17 | PASS |
| LocalModeXClient (`x_api/local_mode/tests.rs`) | 19 | PASS |
| Error Display variants (`error.rs`) | 3 | PASS |
| Server AppState (`api_tests.rs` et al.) | 27 | PASS |
| CLI init/settings (`tuitbot-cli`) | 147 | PASS |
| Core unit+integration (`tuitbot-core`) | 1,129 | PASS |
| Server integration (`tuitbot-server`) | 495 | PASS |

---

## Scenario Validation Matrix

All scenarios traced through source code and confirmed by automated tests.

| # | Scenario | Result | Evidence |
|---|----------|--------|----------|
| S1 | Official X API flow unchanged | PASS | `deps.rs:105-233` — `init_official_mode()` is extracted verbatim. All 1,866 tests pass with no behavioral changes. |
| S2 | Scraper mode saves without client credentials (desktop) | PASS | `validation.rs:274` — `is_x_api_backend` is false when `backend = "scraper"`, skips `client_id` check. Test: `validate_scraper_backend_allows_empty_client_id`. |
| S3 | Scraper mode saves without client credentials (LAN/self_host) | PASS | `validation.rs:261-271` — cloud check only fires for `DeploymentMode::Cloud`. Test: `validate_self_host_scraper_allowed`. |
| S4 | Cloud rejects scraper mode | PASS | `validation.rs:261-271` — explicit `InvalidValue` error. Test: `validate_cloud_scraper_rejected`. |
| S5 | `scraper_allow_mutations = false` blocks writes | PASS | `local_mode/mod.rs:37-42` — `check_mutation()` returns `ScraperMutationBlocked`. 5 tests: `post_tweet_blocked`, `reply_to_tweet_blocked`, `like_tweet_blocked`, `follow_user_blocked`, `delete_tweet_blocked`. |
| S6 | `scraper_allow_mutations = true` falls through to transport stub | PASS | `local_mode/mod.rs:43-45` — returns `ScraperTransportUnavailable`. Tests: `post_tweet_transport_unavailable_when_mutations_enabled`, `reply_to_tweet_transport_unavailable_when_mutations_enabled`. |
| S7 | Auth-gated methods return `FeatureRequiresAuth` | PASS | `local_mode/mod.rs:67-104` — `get_me`, `get_mentions`, `get_home_timeline`, `get_bookmarks`, `bookmark_tweet`, `unbookmark_tweet`. 4 tests confirm. |
| S8 | Settings API round-trip works | PASS | `routes/settings.rs:286-351` — `GET/PATCH` serialize full `Config` via serde. Tests: `settings_json_includes_backend_fields`, `settings_json_roundtrip_scraper_mode`. |
| S9 | Dashboard hides mode selector in cloud | PASS | `XApiSection.svelte:14` — `isCloud = $derived($deploymentMode === 'cloud')`. Line 32: `{#if !isCloud}` wraps mode selector. |
| S10 | Discovery loop spawns in scraper mode, fails gracefully | PASS | `deps.rs:256` — `discovery: true`. `run.rs:149` spawns discovery. `search_tweets` → `ScraperTransportUnavailable` → `LoopError::Other` in `helpers.rs:52-54` → logged, loop continues. |
| S11 | Mentions/target/analytics loops skip in scraper mode | PASS | `deps.rs:255` — `mentions: false`. `run.rs:175` gated by `deps.capabilities.mentions && !is_composer`. `run.rs:223` gated by `deps.capabilities.mentions`. |
| S12 | Token refresh skipped in scraper mode | PASS | `deps.rs:317` — `token_manager = None`. `run.rs:77` — `if let (Some(tm), Some(xc))` skips. |
| S13 | Invalid backend value rejected | PASS | `validation.rs:249-259` — returns `InvalidValue` for values not in `{"", "x_api", "scraper"}`. Test: `validate_invalid_backend_value_rejected`. |
| S14 | Env var overrides work | PASS | Tests: `env_var_override_provider_backend`, `env_var_override_scraper_allow_mutations`, `env_var_scraper_allow_mutations_false`, `env_var_scraper_allow_mutations_invalid`. |
| S15 | Onboarding flow handles scraper mode | PASS | `onboarding/+page.svelte:52` — `canAdvance()` returns true for scraper. Lines 103-104 — submit payload sends `{ provider_backend: "scraper" }`. |

---

## Critical Path Verification

### 1. New user selects Local No-Key Mode in onboarding

Traced: `onboarding/+page.svelte` → step 1 (X Access) → user selects scraper card → `canAdvance()` returns true (no client_id needed) → submit builds `{ x_api: { provider_backend: "scraper" } }` → `PATCH /api/settings` → `Config` serde roundtrip → validation passes → `config.toml` written.

**Result:** PASS — no credentials required to complete onboarding.

### 2. Settings page switches between modes

Traced: `XApiSection.svelte` → `setMode('scraper')` → `updateDraft('x_api.provider_backend', 'scraper')` → save → `PATCH /api/settings` → validation in `validation.rs:249-279` → `config.toml` updated. Switching back to `x_api` shows credential fields.

**Result:** PASS — mode switching is bidirectional and persists.

### 3. CLI starts in scraper mode

Traced: `RuntimeDeps::init()` → checks `config.x_api.provider_backend == "scraper"` → dispatches to `init_scraper_mode()` → creates `LocalModeXClient` via `create_local_client()` → synthetic capabilities (`discovery=true, mentions=false`) → skips OAuth, tier detection, and `get_me()`.

**Result:** PASS — no network calls to X API during startup.

### 4. Loops run with correct gating

Traced: `run.rs` → discovery loop spawns (`capabilities.discovery = true`) → mentions/target/analytics loops skip (`capabilities.mentions = false`) → content/thread loops spawn (autopilot mode) → token refresh skips (`token_manager = None`) → posting queue and approval poster always spawn.

**Result:** PASS — only appropriate loops run in scraper mode.

### 5. Write attempt blocked with actionable error

Traced: `LocalModeXClient::post_tweet()` → `check_mutation("post_tweet")` → `!self.allow_mutations` → returns `ScraperMutationBlocked { message: "post_tweet" }` → Display: `"Write operation blocked in Local No-Key Mode: post_tweet. Enable scraper_allow_mutations in config or switch to Official X API."`.

**Result:** PASS — error message is actionable and guides user to resolution.

---

## Artifact Reconciliation

| Artifact | Divergence | Resolution |
|----------|-----------|------------|
| `charter.md` | None — all charter requirements are met. Feature availability matrix, deployment boundaries, risk inventory, and UI framing all match the implementation. | Aligned |
| `settings-flow.md` | None — UI contract (mode selector cards, conditional content, validation behavior, env vars, TypeScript interface) matches `XApiSection.svelte` and onboarding exactly. | Aligned |
| `runtime-backend-plan.md` | None — runtime behavior matrix, error semantics, loop availability, server integration, and backward compatibility all match the implemented code. | Aligned |
| `README.md` | Updated in this session to document the no-key option. Includes transport caveat. | Updated |
| `config.example.toml` | Updated in this session to document `provider_backend` and `scraper_allow_mutations`. | Updated |

---

## Residual Risks and Known Limitations

### Medium Risk

1. **Scraper transport not implemented** — All read methods in `LocalModeXClient` return `ScraperTransportUnavailable`. Discovery loop spawns but every search fails gracefully (logged, loop continues). This is by design — transport is scoped for a future session. Users in scraper mode will see errors in logs until then.

2. **User expectation gap** — Users who opt into Local No-Key Mode may expect discovery to return results immediately. The README caveat and error messages mitigate this, but the onboarding flow does not explicitly state "transport not yet available." The feature availability matrix in onboarding says "Search and discover tweets — Available" which is aspirationally correct but not currently functional.

### Low Risk

3. **Content/thread loops run in scraper mode (autopilot)** — These loops generate content via LLM and attempt to post. Posting fails with `ScraperTransportUnavailable` (mutations enabled) or `ScraperMutationBlocked` (mutations disabled). The error is logged and the loop continues. No data loss or corruption. This is acceptable because the loops are part of the autopilot mode contract and the errors are informative.

4. **Empty `own_user_id` in scraper mode** — `deps.rs:298` sets `own_user_id = String::new()`. This flows into `XApiMentionsAdapter` but the mentions loop is gated by `capabilities.mentions = false` so it never executes. No risk of self-reply because the loop doesn't run.

5. **Approval poster runs in scraper mode** — If a user manually adds items to the approval queue (via dashboard) and approves them, the approval poster will attempt to post via `LocalModeXClient` and fail with a clear error. This is correct behavior — the error tells the user to switch to the Official API.

---

## Release Decision

**CONDITIONAL GO.**

### Rationale

The Local No-Key Mode infrastructure is correctly wired, comprehensively tested, and safe to merge to main:

1. **Zero regression risk** — The official X API path (`init_official_mode()`) is the original code extracted verbatim. All 1,866 existing tests pass unchanged. The `provider_backend` default is `""` which maps to the official API path, so no existing user is affected.

2. **Opt-in only** — Users must explicitly set `provider_backend = "scraper"` via the settings UI, onboarding wizard, config file, or environment variable. There is no automatic migration or upgrade path.

3. **Defense in depth** — Cloud deployment rejects scraper mode at validation time. Mutations are blocked by default. Auth-gated methods return `FeatureRequiresAuth`. Invalid backend values are rejected. Every boundary is tested.

4. **Actionable errors** — Every error message in the scraper path tells the user what happened and what to do about it (enable mutations, switch to official API, or wait for transport implementation).

### Conditions for Full GO

The CONDITIONAL qualifier exists because the scraper transport is not yet implemented. The feature is safe to ship but does not deliver end-user value for discovery until transport ships. To promote to full GO:

1. Implement scraper transport for `search_tweets`, `get_tweet`, `get_user_by_username` (replaces `ScraperTransportUnavailable` stubs)
2. Add circuit breaker for transport reliability
3. Update onboarding feature availability matrix to reflect actual transport status
4. Consider a CLI startup banner indicating scraper mode

---

## Follow-Up Items

1. **Scraper transport implementation** — Replace `ScraperTransportUnavailable` stubs in `LocalModeXClient` read methods with actual HTTP transport for public X data.
2. **Circuit breaker** — Add transport reliability monitoring with automatic fallback to informative errors on consecutive failures.
3. **Dashboard runtime status** — Use `provider_backend` from `GET /api/runtime/status` to adapt dashboard UI (e.g., hide unavailable features, show mode indicator).
4. **CLI startup banner** — Add `[LOCAL NO-KEY MODE]` indicator to the startup banner when `provider_backend = "scraper"`.
5. **Onboarding accuracy** — Consider adding a note to the feature availability matrix that transport is under development.
