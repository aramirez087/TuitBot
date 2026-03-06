# Runtime Isolation Plan

## Overview

Runtime, assist, and discovery services are now keyed by account rather than using default-account singletons. Each account loads its own effective config (base `config.toml` merged with DB overrides) and maintains isolated credential and generator caches.

## Architecture

### Effective Config Resolution

`AppState::load_effective_config(account_id)` is the single source of truth:

- **Default account**: reads `config.toml` directly (backward compatible).
- **Non-default account**: merges `config.toml` base with `config_overrides` JSON from the `accounts` table.

`routes::content::read_effective_config` delegates to this method, mapping errors to `ApiError`.

### Content Generator Lifecycle

`AppState::get_or_create_content_generator(account_id)` provides lazy per-account init:

1. Check `content_generators` cache (fast path).
2. Load effective config for the account.
3. Create LLM provider via `create_provider(&config.llm)`.
4. Construct `ContentGenerator::new(provider, config.business)`.
5. Cache and return.

Used by `assist.rs` and `discovery.rs` `get_generator()` helpers.

### Runtime Start/Stop/Status

- `runtimes` HashMap is keyed by `account_id` (established in Session 2).
- `status` endpoint now computes `provider_backend` from the account's effective config instead of a global `AppState.provider_backend` field (removed).
- `can_post` is computed per-account via credential file checks (established in Session 3).
- Start/stop only affect the specified account's runtime entry.

### What Was Removed

- `AppState.provider_backend: String` field (global singleton). All consumers now use per-account effective config.

## Boundaries

### In Scope (This Session)
- Per-account effective config for runtime status, assist, and discovery endpoints.
- Lazy content generator creation per account.
- `provider_backend` derived from effective config, not global state.

### Out of Scope
- **Watchtower scoping**: Content sources remain instance-level (Session 6 per charter, S5 priority).
- **Automation loop spawning**: The `start` endpoint creates an empty `Runtime`. Full loop setup (discovery, content, mentions) requires wiring up XApiClient and LLM provider adapters in the server crate, which is future work.
- **WebSocket account filtering**: Events are still broadcast globally (Session 5).
- **Config hot-reload**: Running runtimes don't pick up config changes after `PATCH /api/settings`.

## Cache Invalidation

Content generators are cached until the server restarts. If config changes (LLM provider, business profile), the cached generator will be stale. This is the same behavior as `TokenManager` caching. Future work: invalidate generator cache on `PATCH /api/settings` or account config update.

## Test Coverage

- `runtime_status_per_account_provider_backend`: Two accounts see different `provider_backend` values.
- `runtime_isolation_start_stop`: Start/stop for account A does not affect account B.
- `content_generator_lazy_init_per_account`: Generators are lazily created and cached per account.
- `load_effective_config_per_account`: Config merging produces correct per-account overrides.
