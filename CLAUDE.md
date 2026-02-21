# ReplyGuy - Project Context

## Project Overview

ReplyGuy is a cross-platform Rust CLI autonomous growth assistant for X (Twitter).
It helps founders and indie hackers grow their accounts organically by discovering
relevant conversations, replying with valuable content, posting educational tweets,
and publishing weekly threads.

## Architecture

- **Workspace**: Cargo workspace with two crates
  - `crates/replyguy-core/` - Library crate (all business logic)
  - `crates/replyguy-cli/` - Binary crate (CLI entry point)
- **Error handling**: `thiserror` in library, `anyhow` in binary
- **Testing**: Trait-based mocks + `wiremock` for HTTP fixtures

<!-- SPEC-KITTY:AUTO:START -->
## Active Technologies

- Rust 1.75+ (edition 2021)
- Tokio (async runtime)
- Clap (CLI parsing)
- Reqwest (HTTP client)
- SQLx + SQLite (persistence, WAL mode, embedded migrations)
- Serde + TOML (configuration)
- Tracing (structured logging)
- OAuth2 (X API authentication, PKCE flow)
- thiserror / anyhow (error handling)
- wiremock (HTTP test fixtures)

- Rust 1.75+ (edition 2021) + Tokio (async), Clap (CLI), Reqwest (HTTP), Serde + TOML (config), SQLx (database), Tracing (logging), OAuth2 (auth), thiserror/anyhow (errors) (001-replyguy-autonomous-x-growth-assistant)
- SQLite via SQLx (WAL mode, embedded migrations, connection pool of 4) (001-replyguy-autonomous-x-growth-assistant)
## Recent Changes
- 001-replyguy-autonomous-x-growth-assistant: Added Rust 1.75+ (edition 2021) + Tokio (async), Clap (CLI), Reqwest (HTTP), Serde + TOML (config), SQLx (database), Tracing (logging), OAuth2 (auth), thiserror/anyhow (errors)
- Initial project planning complete (2026-02-21)
<!-- SPEC-KITTY:AUTO:END -->
<!-- MANUAL ADDITIONS -->
## Key Patterns

- All external services (X API, LLM providers) are behind traits for testability
- LLM providers: thin reqwest wrapper (OpenAI-compat for OpenAI+Ollama, native for Anthropic)
- Scoring engine: purely heuristic (no LLM calls) - keyword relevance, follower count, recency, engagement rate
- Config layering: CLI flags > env vars > config.toml > defaults
- SQLite: WAL mode, pool of 4, `sqlx::migrate!()` for embedded migrations
- Data retention: 90 days configurable, periodic cleanup, never delete dedup records
<!-- END MANUAL ADDITIONS -->
