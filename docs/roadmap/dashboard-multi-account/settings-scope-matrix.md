# Settings Scope Matrix

Defines which top-level `Config` keys are **account-scoped** (overridable per-account) vs **instance-scoped** (shared across all accounts).

Enforced at compile time via `ACCOUNT_SCOPED_KEYS` in `crates/tuitbot-core/src/config/merge.rs`.

## Account-Scoped Keys

These keys can appear in `accounts.config_overrides`. When present, they override the corresponding section from `config.toml` for that account only.

| Key | Type | Description |
|-----|------|-------------|
| `mode` | `OperatingMode` | Autopilot vs composer mode |
| `x_api` | `XApiConfig` | X API credentials and provider backend |
| `business` | `BusinessProfile` | Product name, keywords, voice, persona |
| `scoring` | `ScoringConfig` | Scoring weights and threshold |
| `limits` | `LimitsConfig` | Rate limits, banned phrases, mention ratio |
| `intervals` | `IntervalsConfig` | Automation loop intervals |
| `approval_mode` | `bool` | Whether posts require approval |
| `max_batch_approve` | `usize` | Max items in batch approve |
| `schedule` | `ScheduleConfig` | Active hours, preferred times, timezone |
| `targets` | `TargetsConfig` | Target accounts to monitor |
| `content_sources` | `ContentSourcesConfig` | Content source configuration |

## Instance-Scoped Keys

These keys are shared across all accounts and cannot be overridden per-account. Attempts to include them in `config_overrides` or PATCH them for a non-default account will be rejected.

| Key | Type | Rationale |
|-----|------|-----------|
| `server` | `ServerConfig` | Single server instance, one bind address |
| `storage` | `StorageConfig` | Shared database, shared retention policy |
| `logging` | `LoggingConfig` | Shared observability infrastructure |
| `llm` | `LlmConfig` | Single LLM provider/key (single-user product) |
| `deployment_mode` | `DeploymentMode` | Install-wide capability set |
| `connectors` | `ConnectorConfig` | Application-level OAuth credentials |
| `circuit_breaker` | `CircuitBreakerConfig` | Shared rate-limit protection |
| `mcp_policy` | `McpPolicyConfig` | Shared MCP security policy |
| `auth` | `AuthConfig` | Authentication mode (install-wide) |

## Merge Semantics

- **RFC 7396 JSON merge-patch**: Objects recurse, scalars replace, arrays replace entirely.
- **`null` removes**: Sending `null` for an account-scoped key removes the override, causing the account to fall back to the base config value.
- **Validation**: After merging overrides into the base config, the resulting effective config must pass `Config::validate()`.

## API Behavior by Account

| Endpoint | Default Account | Non-Default Account |
|----------|----------------|---------------------|
| `GET /api/settings` | Returns raw `config.toml` as JSON | Returns effective config (base + overrides) with `_overrides` metadata |
| `PATCH /api/settings` | Writes to `config.toml` | Rejects instance-scoped keys (403); persists account-scoped changes to `accounts.config_overrides` |
| `POST /api/settings/validate` | Validates against `config.toml` merge | Validates effective config (base + merged overrides) |

## Default Account Backward Compatibility

The default account (`00000000-0000-0000-0000-000000000000`) always reads/writes `config.toml` directly. When the `X-Account-Id` header is missing, the `AccountContext` extractor defaults to this account, preserving exact pre-multi-account behavior.
