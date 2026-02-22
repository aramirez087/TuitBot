# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ReplyGuy is a cross-platform Rust CLI autonomous growth assistant for X (Twitter).
It helps founders and indie hackers grow their accounts organically by discovering
relevant conversations, replying with valuable content, posting educational tweets,
and publishing weekly threads.

## Build & Development Commands

```bash
# Build
cargo build                              # debug build
cargo build --release                    # release build (binary: target/release/replyguy)

# Test
cargo test                               # run all tests
cargo test -p replyguy-core              # test core crate only
cargo test -p replyguy-cli               # test CLI crate only
cargo test -p replyguy-core scoring      # test a specific module
cargo test -p replyguy-core -- --test-threads=1  # serial (for env-var tests)

# Lint & Format
cargo clippy --workspace                 # lint all crates
cargo fmt --all -- --check               # check formatting
cargo fmt --all                          # auto-format

# Run
cargo run -p replyguy-cli -- init        # run a subcommand via cargo
cargo install --path crates/replyguy-cli # install binary as `replyguy`
```

## Mandatory CI Parity Checklist (for agents)

Before handing off any Rust code change, always run these commands locally and fix failures:

```bash
cargo fmt --all
cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Additional rules:

- Do not manually format Rust code and assume it is correct; always run `cargo fmt --all`.
- Treat all warnings as release blockers (CI uses denied warnings).
- Do not finish the task until the checklist passes.

## Architecture

### Workspace Layout

- **`crates/replyguy-core/`** — Library crate containing all business logic
- **`crates/replyguy-cli/`** — Thin binary crate (CLI parsing, logging init, dispatch to core)
- **`migrations/`** — SQLite migrations (embedded at compile time via `sqlx::migrate!("../../migrations")`)
- **`config.example.toml`** — Full configuration reference

### Core Modules (`replyguy-core/src/`)

| Module | Purpose |
|--------|---------|
| `config/` | Three-layer config: defaults → TOML file → env vars (`REPLYGUY_` prefix, `__` separator) |
| `error.rs` | Per-domain error enums: `ConfigError`, `XApiError`, `LlmError`, `StorageError`, `ScoringError` |
| `x_api/` | X API v2 client behind `XApiClient` trait; OAuth 2.0 PKCE auth; tier detection |
| `llm/` | `LlmProvider` trait with `OpenAiCompatProvider` (OpenAI + Ollama) and `AnthropicProvider` |
| `scoring/` | Purely heuristic scoring (no LLM) — keyword relevance, follower count, recency, engagement |
| `storage/` | SQLite via SQLx — WAL mode, pool of 4, embedded migrations, `init_test_db()` for tests |
| `safety/` | Deduplication, rate limit enforcement |
| `content/` | Content generation orchestration using LLM providers |
| `automation/` | Runtime with `CancellationToken` + concurrent loops (discovery, mentions, content, threads) |
| `startup.rs` | Agent startup orchestration |

### Key Patterns

- **Trait-based testing**: All external services (`XApiClient`, `LlmProvider`) are behind `async_trait` traits. Tests use trait-based mocks + `wiremock` for HTTP fixtures.
- **Error handling**: `thiserror` in core library (typed enums per domain), `anyhow` in CLI binary.
- **LLM providers**: Thin reqwest wrappers — `openai_compat` serves both OpenAI and Ollama via compatible API, `anthropic` uses native Messages API.
- **Config layering**: CLI flags > env vars (`REPLYGUY_X_API__CLIENT_ID`) > `config.toml` > built-in defaults. The `init` subcommand is handled before config loading since the file may not exist yet.
- **Automation runtime**: `Runtime` struct spawns tokio tasks sharing a `CancellationToken`. Graceful shutdown on SIGTERM/Ctrl+C with 30s timeout. Four concurrent loops: discovery, mentions, content posting, thread publishing.
- **Storage**: SQLite WAL mode, pool of 4, `sqlx::migrate!()` for embedded migrations, 90-day configurable retention, dedup records are never deleted. Tests use `storage::init_test_db()` for in-memory SQLite.
- **Build script**: `crates/replyguy-core/build.rs` watches `migrations/` directory for recompilation.

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
