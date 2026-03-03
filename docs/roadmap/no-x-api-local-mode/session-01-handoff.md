# Session 01 Handoff

## What Was Done

Session 01 produced three documentation artifacts defining the product, config, and safety contract for the no-X-API-key local mode:

1. **`charter.md`** — Problem statement, user promise, non-goals, deployment boundaries, feature matrix, risk inventory, and success criteria.
2. **`x-access-contract.md`** — Seven binding decisions covering config field reuse, credential rules, write behavior, validation rules, env var support, TypeScript types, and automation loop behavior.

No production code was changed. This session was documentation-only.

## What Was Decided

| Decision | Outcome |
|----------|---------|
| Config field strategy | Reuse `provider_backend` and `scraper_allow_mutations` on `XApiConfig`. No new config fields. |
| UI framing | "Official X API" and "Local No-Key Mode". No user-facing use of the word "scraper". |
| Default write behavior | `scraper_allow_mutations = false`. All writes fail closed with actionable error. |
| Cloud guard | `deployment_mode = "cloud"` + `provider_backend = "scraper"` rejected at validation. |
| Credential rules | `client_id` required only when `provider_backend` is `"x_api"` or empty. Ignored in scraper mode. |
| Auth-gated features | Mentions, home timeline, bookmarks, `get_me` return typed `FeatureRequiresAuth` errors in scraper mode. |
| Media upload | Always unavailable in scraper mode regardless of mutation flag. |
| MCP alignment | MCP and product runtime use separate provider traits by design. Both implement the same safety semantics. |

## What Was NOT Decided (Deferred)

| Topic | Deferred To | Notes |
|-------|------------|-------|
| Scraper transport implementation | Session 03 | Which scraper library/approach to use for `LocalModeXClient`. |
| Circuit breaker parameters | Session 03 | Failure thresholds, recovery timing, health check interval. |
| Onboarding flow UX details | Session 02 | Exact copy, layout, and progressive disclosure for the mode selector in `XApiStep.svelte`. |
| Migration path for `ProviderBackend` enum | Future | Whether to move the enum from `tuitbot-mcp` to `tuitbot-core`. Not required for v1. |
| `scraper_allow_mutations` advanced toggle copy | Session 02 | Exact warning text shown when user enables mutations. |

## Session 02: Settings & Validation

### Mission

Implement backend-aware config validation, env var overrides, and the dashboard settings/onboarding UI for the mode selector.

### Files to Touch

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/config/validation.rs` | Add three validation rules: cloud+scraper rejection, backend-aware `client_id`, backend value validation. |
| `crates/tuitbot-core/src/config/env_overrides.rs` | Add `TUITBOT_X_API__PROVIDER_BACKEND` and `TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS` handlers. |
| `crates/tuitbot-core/src/config/tests.rs` | Add tests: scraper+empty client_id=valid, x_api+empty client_id=error, cloud+scraper=error, env var overrides, backend value validation. |
| `dashboard/src/lib/api.ts` | Add `provider_backend: string` and `scraper_allow_mutations: boolean` to `TuitbotConfig.x_api`. |
| `dashboard/src/routes/(app)/settings/XApiSection.svelte` | Replace fixed credential form with mode-aware X Access UX: backend selector, conditional credential fields, mutations toggle. |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | Add backend selector. Make credential fields conditional. Add tradeoff info copy. |

### Inputs from Session 01

- **`charter.md`**: Feature matrix and UI framing (use "Official X API" / "Local No-Key Mode" labels).
- **`x-access-contract.md`**: Exact validation rules (Decisions 4), env var names (Decision 5), TypeScript types (Decision 6).

### Deliverable

- `docs/roadmap/no-x-api-local-mode/session-02-handoff.md`

## Session 03: Runtime Backend

### Mission

Implement `LocalModeXClient` in `tuitbot-core` and wire it into the server and automation loops.

### Files to Touch

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/x_api/local_mode.rs` | New file. `LocalModeXClient` implementing `XApiClient`. |
| `crates/tuitbot-core/src/x_api/mod.rs` | Add `pub mod local_mode;` and factory function `create_x_client(config) -> Box<dyn XApiClient>`. |
| `crates/tuitbot-core/src/automation/mod.rs` | Use factory to select client. Handle scraper mode gracefully (no user_id for mentions loop). |
| `crates/tuitbot-core/src/workflow/publish.rs` | Check backend before write. Queue or reject when mutations blocked. |
| `crates/tuitbot-server/src/main.rs` | Use factory for X client initialization. Allow startup without tokens in scraper mode. |

### Inputs from Session 01

- **`x-access-contract.md`**: Error types (Decision 3), automation loop behavior, write gating semantics, MCP alignment notes.

### Deliverable

- `docs/roadmap/no-x-api-local-mode/session-03-handoff.md`

## Session 04: Validation & Docs

### Mission

End-to-end validation, `config.example.toml` updates, README updates, and release readiness assessment.

### Files to Touch

| File | Change |
|------|--------|
| `config.example.toml` | Add commented `provider_backend` and `scraper_allow_mutations` fields. |
| `README.md` | Add "No API Key" section under Getting Started. |
| All quality gates | Full CI checklist + `npm run check`. |

### Inputs from Session 01

- **`x-access-contract.md`**: config.example.toml field format (Decision 7).
- **`charter.md`**: User promise and feature matrix for README copy.

### Deliverable

- `docs/roadmap/no-x-api-local-mode/release-readiness.md`

## Known Risks for Subsequent Sessions

1. **`client_id` validation change:** Session 02 makes `client_id` conditionally required. Existing users who have `client_id = ""` and no `provider_backend` set will now get a validation error — this is correct behavior (they couldn't use the API anyway), but test thoroughly.

2. **No `LocalModeXClient` exists yet:** Session 03 must create this from scratch in `tuitbot-core`. The MCP `ScraperReadProvider` is a stub and implements a different trait (`SocialReadProvider`), so it cannot be reused directly.

3. **Automation loop startup:** Session 03 must handle the case where `user_id` is unknown in scraper mode. The mentions loop, analytics snapshots, and any loop that calls `get_me()` must degrade gracefully.

4. **Frontend settings round-trip:** Session 02 should verify that `PATCH /api/settings` correctly persists `provider_backend` and `scraper_allow_mutations` by adding an integration test or manual verification step.

## Changed Files in Session 01

```
docs/roadmap/no-x-api-local-mode/charter.md          (new)
docs/roadmap/no-x-api-local-mode/x-access-contract.md (new)
docs/roadmap/no-x-api-local-mode/session-01-handoff.md (new)
```

No production code was modified.
