# X Access Contract — v1

This document locks in the configuration, validation, and runtime behavior contract for the no-X-API-key local mode. Sessions 02-04 implement against this spec.

## Decision 1: Reuse `provider_backend` — No New Config Fields

The `XApiConfig` struct already has both fields needed:

```rust
// crates/tuitbot-core/src/config/types.rs
pub struct XApiConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub provider_backend: String,        // "" | "x_api" | "scraper"
    pub scraper_allow_mutations: bool,   // default: false
}
```

The UI wraps these fields with user-friendly labels:

| Config Value | UI Label |
|-------------|----------|
| `provider_backend = "x_api"` (or empty) | "Official X API" |
| `provider_backend = "scraper"` | "Local No-Key Mode" |

**Rationale:** Adding a separate `x_access_mode` field would create a sync problem between two representations of the same concept. The MCP crate already has a `ProviderBackend` enum and `parse_backend()` that reads this field. All runtime code reads `config.x_api.provider_backend` — no translation layer needed.

**Note on the `ProviderBackend` enum:** Currently defined only in `tuitbot-mcp` (`crates/tuitbot-mcp/src/provider/mod.rs`). The product runtime (`tuitbot-core`, `tuitbot-server`) uses string matching against `config.x_api.provider_backend`. Session 03 may optionally move the enum to `tuitbot-core` for type safety, but this is not required for v1. The MCP and product runtime use separate provider trait hierarchies by design (`SocialReadProvider` in MCP vs. `XApiClient` in core).

## Decision 2: Credential Rules by Backend

| Backend | `client_id` | `client_secret` | OAuth tokens | `tuitbot auth` |
|---------|------------|-----------------|--------------|-----------------|
| `x_api` (default) | Required (validation error if empty) | Optional (for confidential clients) | Required (obtained via `tuitbot auth`) | Required |
| `scraper` | Ignored (may be empty) | Ignored (may be empty) | Not needed | Not needed |

**Current state:** Validation (`validation.rs`) does not check `client_id` at all — it defaults to an empty string and the auth flow fails later at runtime. Session 02 makes validation backend-aware: `client_id` is required only when `provider_backend` is `"x_api"` or empty.

**Backward compatibility:** The default `provider_backend` is `""` (empty string), which `parse_backend()` maps to `XApi`. Existing configs without `provider_backend` set are unaffected — they continue to require `client_id` and work exactly as before.

## Decision 3: Safe Write Behavior in Scraper Mode

All write operations (post, reply, like, retweet, follow, media upload) follow fail-closed semantics in scraper mode.

### When `scraper_allow_mutations = false` (default)

| Context | Behavior |
|---------|----------|
| Direct write call (server/CLI) | Returns `ScraperMutationBlocked` error with guidance: *"Posting is disabled in Local No-Key Mode. Enable scraper mutations in Settings > X Access > Advanced, or switch to the Official X API for full posting capabilities."* |
| Automation loop (autopilot) | Write step is skipped. Draft is routed to the approval queue with status `blocked` and reason `scraper_mutations_disabled`. The queue item shows actionable guidance in the dashboard. |
| MCP tool call | Existing `scraper_mutation_guard()` in `x_actions/mod.rs` blocks the call and returns the error message. No changes needed. |

### When `scraper_allow_mutations = true`

| Context | Behavior |
|---------|----------|
| Direct write call | Attempts the write via scraper transport. If transport health check fails (circuit breaker open, consecutive errors), the write is queued instead of posted, with status `transport_unhealthy`. |
| Automation loop | Same as direct, but subject to all existing safety limits (rate limits, per-author limits, banned phrases). |
| MCP tool call | Existing `scraper_mutation_guard()` allows the call. `ProviderCapabilities::scraper(true)` reports `mutations_available: true`. |
| Media upload | Always returns `MediaUploadUnavailable` error. Media upload is not supported via scraper transport regardless of the mutations flag. |

### Error types

Session 03 adds these error variants to the appropriate error enums in `tuitbot-core`:

| Error | When | Guidance |
|-------|------|----------|
| `ScraperMutationBlocked` | Write attempted with `scraper_allow_mutations = false` | Enable mutations or switch to official API |
| `ScraperTransportUnhealthy` | Write attempted but circuit breaker is open | Retry later or switch to official API |
| `MediaUploadUnavailable` | Media upload attempted in scraper mode | Switch to official API for media uploads |
| `FeatureRequiresAuth` | Auth-gated read (mentions, timeline, bookmarks, get_me) attempted in scraper mode | Switch to official API |

## Decision 4: Validation Rules

Session 02 adds these rules to `Config::validate()` in `validation.rs`:

### Rule 1: Cloud + scraper rejection

```
if deployment_mode == Cloud && provider_backend == "scraper" {
    error: "Local No-Key Mode is not available in cloud deployment.
            Use the Official X API (provider_backend = \"x_api\")."
}
```

### Rule 2: Backend-aware client_id

```
if provider_backend is "" or "x_api" {
    if client_id is empty {
        error: "x_api.client_id is required when using the Official X API backend.
                Get your Client ID from https://developer.x.com, or switch to
                Local No-Key Mode (provider_backend = \"scraper\") to use Tuitbot
                without API credentials."
    }
}
// When provider_backend == "scraper", client_id is not required.
```

### Rule 3: Backend value validation

```
if provider_backend is not "" and not "x_api" and not "scraper" {
    error: "x_api.provider_backend must be 'x_api' or 'scraper', got '{value}'"
}
```

## Decision 5: Environment Variable Support

Session 02 adds these handlers to `apply_env_overrides()` in `env_overrides.rs`:

```
TUITBOT_X_API__PROVIDER_BACKEND   → config.x_api.provider_backend (string)
TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS → config.x_api.scraper_allow_mutations (bool)
```

This is required for Docker/self-host users who configure entirely via env vars.

## Decision 6: Dashboard TypeScript Types

Session 02 updates the `TuitbotConfig` interface in `dashboard/src/lib/api.ts`:

```typescript
x_api: {
    client_id: string;
    client_secret: string | null;
    provider_backend: string;          // added
    scraper_allow_mutations: boolean;   // added
};
```

No server-side changes are needed — the generic `GET/PATCH /api/settings` endpoint already serializes the full `Config` struct. Adding the fields to the TypeScript types makes them flow through the existing settings round-trip.

## Decision 7: config.example.toml Updates

Session 04 adds commented fields under `[x_api]`:

```toml
[x_api]
client_id = "your-client-id-here"
# client_secret = "your-client-secret-here"

# X access mode: "x_api" (default, requires credentials above) or "scraper"
# (Local No-Key Mode, no credentials needed, read-only by default).
# Local No-Key Mode is only available in desktop and self_host deployment modes.
# provider_backend = "x_api"

# Allow write operations (post, reply, like, follow) in Local No-Key Mode.
# Only meaningful when provider_backend = "scraper". Default: false.
# WARNING: Posting via scraper transport carries elevated risk of account restrictions.
# scraper_allow_mutations = false
```

## Automation Loop Behavior by Backend

| Loop | `x_api` | `scraper` |
|------|---------|-----------|
| Discovery (keyword search) | Runs normally | Runs normally (public data) |
| Target monitoring | Runs normally | Runs normally (public profiles) |
| Mentions check | Runs normally | Skipped (auth-gated). Logs once: "Mentions loop disabled in Local No-Key Mode." |
| Content posting | Runs normally | Gated by `scraper_allow_mutations`. When disabled, drafts queue as `blocked`. |
| Thread posting | Runs normally | Gated by `scraper_allow_mutations`. When disabled, plans queue as `blocked`. |
| Analytics snapshot | Runs normally | Degraded: skips own-profile stats. Logs once: "Analytics limited in Local No-Key Mode." |

## Data Flow Diagram

```
User selects mode in Settings
         │
         ▼
┌─────────────────────┐
│  provider_backend    │
│  stored in config    │
└─────────┬───────────┘
          │
    ┌─────┴──────┐
    │            │
    ▼            ▼
 "x_api"     "scraper"
    │            │
    ▼            ▼
 XApiClient   LocalModeXClient (Session 03)
 (existing)   implements XApiClient trait
    │            │
    ▼            ▼
 Full API     Public reads: scraper transport
 access       Auth-gated reads: typed errors
              Writes: gated by scraper_allow_mutations
```

## MCP Alignment

The MCP crate already implements the scraper safety model:

- `ProviderBackend` enum with `XApi` / `Scraper` variants (`provider/mod.rs`)
- `scraper_mutation_guard()` blocks writes when mutations disabled (`x_actions/mod.rs`)
- `ProviderCapabilities::scraper(allow_mutations)` reports backend status (`capabilities.rs`)
- `ScraperReadProvider` stubs all methods with appropriate errors (`scraper.rs`)

The product runtime (core + server) builds a parallel but separate implementation:

- `LocalModeXClient` implements the `XApiClient` trait (not `SocialReadProvider`)
- Uses the same scraper transport under the hood
- Shares the same safety semantics (mutation guard, auth-gated errors)
- Does not share code with the MCP provider (different trait hierarchies)

This separation is intentional and correct — MCP tools and product workflows use different abstraction layers.
