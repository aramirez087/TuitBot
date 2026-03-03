# Runtime Backend Plan — Local No-Key Mode

## Overview

Session 03 wired `provider_backend = "scraper"` into the product runtime so the CLI, server, and automation loops honor the selected backend without requiring X API credentials.

## Runtime Behavior Matrix

| Operation | Official (`x_api`) | Scraper (mutations off) | Scraper (mutations on) |
|-----------|-------------------|------------------------|----------------------|
| **Startup** | OAuth tokens required, tier detection, `get_me()` | No tokens, synthetic `Basic` tier, no `get_me()` |  Same |
| **Discovery loop** | Search via X API | Spawned (transport stub returns error) | Same |
| **Mentions loop** | Runs if Pro tier | Skipped (`capabilities.mentions = false`) | Same |
| **Target loop** | Runs if Pro tier | Skipped (follows mentions gate) | Same |
| **Analytics loop** | Runs if mentions cap | Skipped (`capabilities.mentions = false`) | Same |
| **Content loop** | Generates + posts tweets | Generates tweets, posts fail with `ScraperMutationBlocked` | Generates tweets, posts fail with `ScraperTransportUnavailable` |
| **Thread loop** | Generates + posts threads | Same as content loop | Same as content loop |
| **Posting queue** | Executes via `XApiHttpClient` | Queue processes, execution returns scraper errors | Same |
| **Approval poster** | Posts approved items | Posts fail with clear scraper error message | Same |
| **Token refresh** | Periodic refresh loop | Skipped (no `TokenManager`) | Same |
| **`post_tweet()`** | Posts via X API v2 | `ScraperMutationBlocked` | `ScraperTransportUnavailable` |
| **`reply_to_tweet()`** | Posts via X API v2 | `ScraperMutationBlocked` | `ScraperTransportUnavailable` |
| **`get_me()`** | Returns authenticated user | `FeatureRequiresAuth` | `FeatureRequiresAuth` |
| **`search_tweets()`** | Searches via X API v2 | `ScraperTransportUnavailable` | `ScraperTransportUnavailable` |
| **`upload_media()`** | Uploads via X API v1.1 | `MediaUploadError` | `MediaUploadError` |

## Error Handling Semantics

Three new `XApiError` variants:

| Variant | When | User Action |
|---------|------|-------------|
| `ScraperMutationBlocked` | Write method called with `scraper_allow_mutations = false` | Enable mutations in config or switch to `x_api` |
| `ScraperTransportUnavailable` | Any method called but scraper transport not yet implemented | Wait for transport implementation (future session) |
| `FeatureRequiresAuth` | Auth-gated method called in scraper mode | Switch to `x_api` backend |

All three variants map to `LoopError::Other` in the automation adapters, causing loops to log and continue without crashing.

## Configuration Contract

```toml
[x_api]
provider_backend = "scraper"       # "" | "x_api" | "scraper"
scraper_allow_mutations = false    # default: false
```

- Validation (Session 02): `scraper` rejected in `deployment_mode = "cloud"`
- Env vars: `TUITBOT_X_API__PROVIDER_BACKEND`, `TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS`

## Client Selection

Factory function `create_local_client(config: &XApiConfig)`:
- Returns `Some(Arc<dyn XApiClient>)` when `provider_backend = "scraper"`
- Returns `None` for `""` or `"x_api"` (caller constructs `XApiHttpClient` with OAuth tokens)

CLI `RuntimeDeps::init()` branches early:
- `init_official_mode()`: existing OAuth → tier detection → `get_me()` path
- `init_scraper_mode()`: `LocalModeXClient` → synthetic capabilities → empty user ID

## Loop Availability

| Loop | Gate | Scraper Mode |
|------|------|--------------|
| Discovery | `capabilities.discovery` | Enabled (search stub will fail gracefully) |
| Mentions | `capabilities.mentions && !is_composer` | Disabled |
| Target | `capabilities.mentions && !is_composer` | Disabled |
| Analytics | `capabilities.mentions` | Disabled |
| Content | `!is_composer` | Enabled (post attempt will fail with scraper error) |
| Thread | `!is_composer` | Enabled (post attempt will fail with scraper error) |
| Posting queue | Always | Enabled (execution errors are logged) |
| Approval poster | Always | Enabled (post errors are logged) |
| Token refresh | `token_manager.is_some()` | Disabled |
| Status reporter | `effective_interval > 0` | Enabled |

## Server Integration

- `AppState` gains `provider_backend: String` field
- `GET /api/runtime/status` includes `"provider_backend"` in response JSON
- Server startup requires no X API credentials regardless of backend
- Frontend can check `provider_backend` to adapt UI behavior

## Backward Compatibility

- Official X API path is unchanged — `RuntimeDeps::init()` dispatches to `init_official_mode()` which is the original code extracted verbatim
- `token_manager` and `x_client` are now `Option<_>` on `RuntimeDeps`, but the official path sets `Some(...)` for both
- `dyn_client: Arc<dyn XApiClient>` is a new field — in official mode it's cast from `Arc<XApiHttpClient>`, in scraper mode it's `Arc<LocalModeXClient>`
- All existing tests pass without behavioral changes (server tests updated to include `provider_backend: String::new()` in AppState construction)

## Future Transport Integration Points

When actual scraper transport is implemented:
1. Replace `ScraperTransportUnavailable` stubs in `LocalModeXClient` read methods with real HTTP/browser calls
2. Replace `ScraperTransportUnavailable` stubs in write methods (when `allow_mutations = true`) with real posting
3. Add health check mechanism for transport reliability
4. Consider circuit breaker integration for scraper transport failures
