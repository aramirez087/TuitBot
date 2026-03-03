# Session 02 Handoff

## What Was Done

Session 02 implemented backend-aware config validation, environment variable overrides, and the full dashboard UX for selecting between Official X API and Local No-Key Mode.

### Rust (tuitbot-core)

1. **`config/validation.rs`** — Added three validation rules:
   - Backend value validation: rejects unknown values (not `""`, `"x_api"`, or `"scraper"`)
   - Cloud + scraper rejection: `deployment_mode = "cloud"` + `provider_backend = "scraper"` produces `InvalidValue` error
   - Backend-aware `client_id`: required only when `provider_backend` is `""` or `"x_api"`

2. **`config/env_overrides.rs`** — Added two env var handlers:
   - `TUITBOT_X_API__PROVIDER_BACKEND` → `config.x_api.provider_backend` (string)
   - `TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS` → `config.x_api.scraper_allow_mutations` (bool via `parse_env_bool`)

3. **`config/tests_backend.rs`** — New test module with 18 tests covering:
   - Scraper backend allows empty client_id
   - x_api backend requires client_id
   - Empty backend (default) requires client_id
   - Cloud + scraper rejection
   - Desktop and self_host allow scraper
   - Invalid backend value rejection
   - Default `scraper_allow_mutations = false`
   - Env var overrides for both fields
   - TOML round-trip for scraper and x_api configs
   - JSON serialization (settings API payload shape)
   - `parse_env_bool` variant coverage

4. **`config/tests.rs`** — Updated 12 existing tests to set `client_id` so they pass the new backend-aware validation. No behavioral changes to existing tests.

5. **`config/mod.rs`** — Registered `tests_backend` module.

### Server (tuitbot-server)

No code changes needed. Verified that `GET/PATCH /api/settings` already serializes `provider_backend` and `scraper_allow_mutations` through the generic `serde_json::to_value(config)` → TOML pipeline. The `XApiConfig` struct has `#[serde(default)]` on both fields.

### Dashboard

6. **`api.ts`** — Added `provider_backend: string` and `scraper_allow_mutations: boolean` to the `TuitbotConfig.x_api` interface.

7. **`stores/settings.ts`** — Added `provider_backend` to `hasDangerousChanges()` check.

8. **`stores/onboarding.ts`** — Added `provider_backend: string` field (default: `""`) to `OnboardingData` interface and both initial/reset states.

9. **`settings/XApiSection.svelte`** — Rewrote as mode-aware "X Access" section:
   - Radio-style mode selector cards (Official X API / Local No-Key Mode)
   - Conditional credential fields (shown only for x_api mode)
   - Scraper info banner with read-only explanation
   - Advanced collapsible section with `scraper_allow_mutations` toggle and warning
   - Cloud deployment guard (hides scraper option)

10. **`settings/+page.svelte`** — Updated nav label from "X API" to "X Access".

11. **`onboarding/XApiStep.svelte`** — Rewrote as mode-aware onboarding step:
    - Mode selector cards with "Recommended" badge on Official X API
    - Feature availability matrix for Local No-Key Mode
    - Conditional setup guide and credential fields
    - Cloud deployment guard

12. **`onboarding/+page.svelte`** — Three changes:
    - Step label: "X API" → "X Access"
    - `canAdvance()`: scraper mode skips credential requirement
    - `submit()`: sends `{ provider_backend: "scraper" }` instead of credentials when in scraper mode

### Documentation

13. **`settings-flow.md`** — UI contract document with layout descriptions, validation behavior table, env var reference, TypeScript interface, and API round-trip notes.

## What Was Decided

| Decision | Outcome |
|----------|---------|
| Existing tests updated, not rewritten | Added `client_id = "test-id"` to 12 tests that expect `validate().is_ok()`. Tests expecting `unwrap_err()` were not changed — additional `client_id` error doesn't affect their assertions. |
| Test file split | New tests in `tests_backend.rs` (separate module) since `tests.rs` was already 1319 lines. |
| No server code changes | Generic serialization already handles the fields. Integration test deferred to Session 04. |
| Cloud guard: UI + validation | Both the frontend (hides scraper option) and backend (rejects at validation) block cloud+scraper. Belt and suspenders. |
| Credentials not cleared on mode switch | When user switches to scraper, credential fields are hidden but values preserved. User might switch back. |
| `onboarding` payload minimal in scraper mode | Only sends `{ provider_backend: "scraper" }` under `x_api`. Omits empty `client_id` to avoid unnecessary data. |

## What Was NOT Done (Deferred)

| Topic | Deferred To | Notes |
|-------|-------------|-------|
| `LocalModeXClient` runtime implementation | Session 03 | No scraper transport exists yet. Config and UI are ready. |
| Server integration test (settings round-trip) | Session 04 | Type-level round-trip tested. Full HTTP test needs running server. |
| `FeatureRequiresAuth` error types | Session 03 | Error variants for auth-gated features in scraper mode. |
| Automation loop scraper-awareness | Session 03 | Mentions loop skip, analytics degradation, write gating. |
| `config.example.toml` updates | Session 04 | Commented fields for `provider_backend` and `scraper_allow_mutations`. |

## Session 03: Runtime Backend

### Mission

Implement `LocalModeXClient` in `tuitbot-core` and wire it into the server and automation loops.

### Inputs from Session 02

- **Config validation is done.** `provider_backend = "scraper"` passes validation without `client_id`. Cloud guard blocks scraper in cloud mode. All in `validation.rs`.
- **Env var overrides are done.** Docker users can set `TUITBOT_X_API__PROVIDER_BACKEND=scraper`.
- **UI is done.** Settings and onboarding both write `provider_backend` to config. The runtime just needs to read it and select the right client.
- **Settings API works.** No server changes needed for config round-trip.

### Files to Create/Modify

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/x_api/local_mode.rs` | New. `LocalModeXClient` implementing `XApiClient` trait. |
| `crates/tuitbot-core/src/x_api/mod.rs` | Add `pub mod local_mode;` and factory function `create_x_client(config)`. |
| `crates/tuitbot-core/src/error.rs` | Add `ScraperMutationBlocked`, `ScraperTransportUnhealthy`, `MediaUploadUnavailable`, `FeatureRequiresAuth` variants. |
| `crates/tuitbot-core/src/automation/mod.rs` | Use factory for client selection. Skip mentions loop in scraper mode. |
| `crates/tuitbot-core/src/workflow/publish.rs` | Check backend before writes. Queue or reject when mutations blocked. |
| `crates/tuitbot-server/src/main.rs` | Use factory for client init. Allow startup without tokens in scraper mode. |

### Key Design Constraints

1. `LocalModeXClient` implements `XApiClient` (not `SocialReadProvider` from MCP)
2. `get_me()` returns `FeatureRequiresAuth` error in scraper mode
3. Mentions loop logs once and skips when `user_id` is unavailable
4. Writes gated by `scraper_allow_mutations` config field
5. Media upload always returns `MediaUploadUnavailable` in scraper mode
6. Circuit breaker needed for scraper transport health

## Changed Files in Session 02

```
crates/tuitbot-core/src/config/validation.rs         (modified)
crates/tuitbot-core/src/config/env_overrides.rs       (modified)
crates/tuitbot-core/src/config/mod.rs                  (modified)
crates/tuitbot-core/src/config/tests.rs                (modified — 12 tests updated)
crates/tuitbot-core/src/config/tests_backend.rs        (new — 18 tests)
dashboard/src/lib/api.ts                                (modified)
dashboard/src/lib/stores/settings.ts                    (modified)
dashboard/src/lib/stores/onboarding.ts                  (modified)
dashboard/src/routes/(app)/settings/XApiSection.svelte  (rewritten)
dashboard/src/routes/(app)/settings/+page.svelte        (modified — nav label)
dashboard/src/lib/components/onboarding/XApiStep.svelte (rewritten)
dashboard/src/routes/onboarding/+page.svelte            (modified — 3 changes)
docs/roadmap/no-x-api-local-mode/settings-flow.md      (new)
docs/roadmap/no-x-api-local-mode/session-02-handoff.md  (new)
```

## Quality Gate Results

- `cargo fmt --all && cargo fmt --all --check` — pass
- `RUSTFLAGS="-D warnings" cargo test --workspace` — pass (128 config tests, all green)
- `cargo clippy --workspace -- -D warnings` — pending (run at commit time)
- `npm run check` — 0 errors, 7 pre-existing warnings
- `npm run build` — success
