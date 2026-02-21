# Research: ReplyGuy — Autonomous X Growth Assistant

**Feature**: `001-replyguy-autonomous-x-growth-assistant`
**Date**: 2026-02-21

## 1. X API v2 Integration

### Decision: Use X API v2 with OAuth 2.0 PKCE

**Rationale**: X API v2 is the current supported API. V1.1 endpoints are deprecated and being phased out.

**Alternatives considered**:
- X API v1.1 — deprecated, limited availability
- Third-party X API wrappers — adds dependency risk, may violate ToS

### Endpoints Required

| Operation | Endpoint | Method |
|---|---|---|
| Search tweets | `GET /2/tweets/search/recent` | Bearer / OAuth |
| Get mentions | `GET /2/users/{id}/mentions` | Bearer / OAuth |
| Post tweet | `POST /2/tweets` | OAuth (write) |
| Reply to tweet | `POST /2/tweets` (with `reply.in_reply_to_tweet_id`) | OAuth (write) |
| Post thread | Sequential `POST /2/tweets` chaining reply IDs | OAuth (write) |

### Required OAuth Scopes

`tweet.read tweet.write users.read offline.access`

### Rate Limits by Tier

| Operation | Free Tier | Basic Tier ($200/mo) |
|---|---|---|
| Tweet search | NOT AVAILABLE | 300 req/15min (user), 15K reads/mo |
| Mentions | NOT AVAILABLE | 180 req/15min (user), 15K reads/mo |
| Post tweet | ~17 req/24hr, 500 posts/mo | 100 req/24hr, 50K posts/mo |

### Token Management

- Access tokens expire every **2 hours** (7200 seconds)
- Refresh tokens are long-lived (requires `offline.access` scope)
- Token refresh: `POST /2/oauth2/token` with `grant_type=refresh_token`
- Rate limit headers: `x-rate-limit-remaining`, `x-rate-limit-reset` (UTC epoch)

### Thread Creation

No dedicated thread endpoint. Chain sequential `POST /2/tweets` calls, each using the previous response's `id` as `reply.in_reply_to_tweet_id`. All tweets in the thread share the first tweet's `conversation_id`.

## 2. LLM Provider Integration

### Decision: Thin reqwest wrapper with two implementations

**Rationale**: ReplyGuy only needs `POST /chat/completions` (or equivalent) — one HTTP endpoint per provider. A thin wrapper is ~200-300 lines total, avoids three heavy dependencies, and gives full control over retry/backoff logic.

**Alternatives considered**:
- `async-openai` + Anthropic crate + `ollama-rs` — three different abstractions, error types, and update cadences; still need an adapter trait
- `genai` crate (multi-provider, 660 GitHub stars) — adds dependency, 0.x maturity, less control over error recovery

### Provider Implementation Map

| Provider | Implementation | Base URL | Auth |
|---|---|---|---|
| OpenAI | OpenAI-compat | `https://api.openai.com/v1` | `Authorization: Bearer <key>` |
| Ollama | OpenAI-compat (same code) | `http://localhost:11434/v1` | `Authorization: Bearer ollama` |
| Anthropic | Native Messages API | `https://api.anthropic.com/v1/messages` | `x-api-key` header + `anthropic-version` header |

**Key insight**: Ollama fully supports the OpenAI-compatible endpoint (`/v1/chat/completions`), so only two HTTP implementations are needed: OpenAI-compat (covers OpenAI + Ollama) and Anthropic native.

### Trait Design

A single `LlmProvider` trait with one method (`complete`) plus `health_check`. Higher-level content generation (reply, tweet, thread) lives in a `ContentGenerator` struct that takes a `Box<dyn LlmProvider>`.

## 3. SQLite Storage with SQLx

### Decision: SQLx with embedded migrations, WAL mode, connection pool

**Rationale**: `sqlx::migrate!()` compiles migrations into the binary (zero user friction), WAL mode enables concurrent reads/writes for the daemon's multiple loops, and a small pool (4 connections) handles concurrency without complexity.

**Alternatives considered**:
- `rusqlite` — synchronous API, doesn't integrate with tokio natively
- `diesel` — heavier ORM, code generation, more boilerplate for simple queries
- `refinery` for migrations — separate runtime dependency; sqlx's built-in migrate is sufficient

### Connection Configuration

- Pool: 4 max connections (1 writer + readers)
- Journal mode: WAL (`SqliteJournalMode::Wal`)
- Synchronous: Normal (safe with WAL, better performance)
- Busy timeout: 5 seconds
- Foreign keys: enabled
- `optimize_on_close`: enabled

### Data Types

- Tweet IDs: `TEXT` (64-bit IDs can exceed i64 range)
- Timestamps: `TEXT` in ISO-8601 format (lexicographic sort, maps to chrono in Rust)
- Metadata: `TEXT` storing serialized JSON
- Booleans: `INTEGER` (0/1)

### Cleanup Strategy

- Periodic background task (every 6 hours)
- Configurable retention period (default 90 days)
- Discovered tweets (unreplied): 7 days
- Replied-to tweets and reply records: 90 days
- Action log: 14 days
- Rate limit counters: never deleted
- `VACUUM` only after large deletions (>1000 rows)

## 4. Scoring Engine

### Decision: Purely heuristic scoring, no LLM calls

**Rationale**: Fast, free, predictable. Scoring runs on every discovered tweet — using LLM calls would be slow and expensive. Configurable weights let users tune the algorithm.

**Scoring Signals**:
- **Keyword relevance** (0-40 pts): How many configured keywords appear in the tweet, weighted by specificity
- **Author follower count** (0-20 pts): Logarithmic scale — replying to accounts with larger audiences has more growth potential
- **Tweet recency** (0-15 pts): Newer tweets get higher scores — early replies get more visibility
- **Engagement rate** (0-25 pts): Likes + retweets relative to author's follower count — high engagement rate signals viral potential

All weights configurable in `config.toml` under `[scoring]`.

## 5. Error Handling

### Decision: `thiserror` in library + `anyhow` in binary

**Rationale**: Typed errors in `replyguy-core` enable pattern matching and proper recovery in the automation loops. `anyhow` in `replyguy-cli` provides ergonomic error reporting at the user-facing boundary.

### Error Categories in Library

- `XApiError` — rate limited, auth expired, account restricted, network failure
- `LlmError` — provider unreachable, rate limited, parse failure, not configured
- `StorageError` — database connection, migration, query failure
- `ConfigError` — missing fields, invalid values, file not found
- `ScoringError` — invalid tweet data

## 6. Configuration Layering

### Decision: CLI flags > env vars > config.toml

**Rationale**: Full layering supports development (`config.toml`), server deployment (env vars for secrets), and one-off overrides (CLI flags).

**Implementation**: Use `clap` for CLI args, manually layer env vars (prefixed `REPLYGUY_`), then load `config.toml` with `toml` crate. Merge in precedence order.

## 7. Crate Structure

### Decision: Cargo workspace — library + binary

**Rationale**: Clean separation of core logic (testable, reusable) from CLI entry point. Library crate can be tested independently. Binary crate is a thin CLI layer.

```
Cargo.toml          (workspace root)
crates/
  replyguy-core/    (library — all business logic)
  replyguy-cli/     (binary — CLI entry point, clap, anyhow)
```
