# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tuitbot is a cross-platform Rust CLI autonomous growth assistant for X (Twitter).
It helps founders and indie hackers grow their accounts organically by discovering
relevant conversations, building relationships with target accounts, replying with
varied and authentic content, posting educational tweets, publishing weekly threads,
and tracking analytics to optimize strategy over time. Supports optional human-in-the-loop
approval mode for reviewing all generated content before posting.

## Build & Development Commands

```bash
# Build
cargo build                              # debug build
cargo build --release                    # release build (binary: target/release/tuitbot)

# Test
cargo test                               # run all tests
cargo test -p tuitbot-core              # test core crate only
cargo test -p tuitbot-cli               # test CLI crate only
cargo test -p tuitbot-core scoring      # test a specific module
cargo test -p tuitbot-core -- --test-threads=1  # serial (for env-var tests)

# Lint & Format
cargo clippy --workspace                 # lint all crates
cargo fmt --all -- --check               # check formatting
cargo fmt --all                          # auto-format

# Run
cargo run -p tuitbot-cli -- init        # run a subcommand via cargo
cargo install --path crates/tuitbot-cli # install binary as `tuitbot`
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

## Crates.io Release Discipline (Mandatory)

When changing any crate under `crates/`, keep the workspace publishable to crates.io:

- Keep `Cargo.lock` committed and current.
- Every local `path` dependency between workspace crates must also include an explicit `version`.
- Keep package metadata present in every published crate:
  - `description`
  - `license`
  - `repository`
  - `homepage`
  - `documentation`
  - `keywords`
- Do not change tag conventions in `release-plz.toml` without explicit approval:
  - `tuitbot-core-vX.Y.Z`
  - `tuitbot-mcp-vX.Y.Z`
  - `tuitbot-cli-vX.Y.Z`
- Keep `release_always = false` except for one-time bootstrap scenarios.

Before handing off publish-affecting changes, run:

```bash
release-plz update --config release-plz.toml --allow-dirty
cargo package --workspace --allow-dirty
```

If either command fails, treat it as a release blocker and fix before handoff.

## Architecture

### Workspace Layout

- **`crates/tuitbot-core/`** — Library crate containing all business logic
- **`crates/tuitbot-cli/`** — Thin binary crate (CLI parsing, logging init, dispatch to core)
- **`migrations/`** — SQLite migrations (embedded at compile time via `sqlx::migrate!("../../migrations")`)
- **`config.example.toml`** — Full configuration reference

### Core Modules (`tuitbot-core/src/`)

| Module | Purpose |
|--------|---------|
| `config/` | Three-layer config: defaults → TOML file → env vars (`TUITBOT_` prefix, `__` separator) |
| `error.rs` | Per-domain error enums: `ConfigError`, `XApiError`, `LlmError`, `StorageError`, `ScoringError` |
| `x_api/` | X API v2 client behind `XApiClient` trait; OAuth 2.0 PKCE auth; tier detection |
| `llm/` | `LlmProvider` trait with `OpenAiCompatProvider` (OpenAI + Ollama) and `AnthropicProvider` |
| `scoring/` | 6-signal heuristic scoring — keyword relevance, targeted followers (bell curve), recency, engagement, reply count, content type |
| `storage/` | SQLite via SQLx — WAL mode, pool of 4, embedded migrations, `init_test_db()` for tests. Submodules: `analytics`, `approval_queue`, `author_interactions`, `target_accounts`, `replies`, `threads`, `tweets`, `action_log`, `rate_limits`, `cleanup` |
| `safety/` | Deduplication, rate limit enforcement, banned phrase filtering, per-author reply limits, self-reply prevention |
| `content/` | Content generation via LLM + `frameworks.rs` (reply archetypes, tweet formats, thread structures with weighted random selection) |
| `automation/` | Runtime with `CancellationToken` + 6 concurrent loops: discovery, mentions, content, threads, target monitoring, analytics. Also: posting queue with optional approval mode, `schedule.rs` (timezone-aware active hours gating via `chrono-tz`), `scheduler.rs` (`LoopScheduler` with jitter for human-like timing) |
| `startup.rs` | Agent startup orchestration |

### Key Patterns

- **Trait-based testing**: All external services (`XApiClient`, `LlmProvider`) are behind `async_trait` traits. Tests use trait-based mocks + `wiremock` for HTTP fixtures.
- **Error handling**: `thiserror` in core library (typed enums per domain), `anyhow` in CLI binary.
- **LLM providers**: Thin reqwest wrappers — `openai_compat` serves both OpenAI and Ollama via compatible API, `anthropic` uses native Messages API.
- **Config layering**: CLI flags > env vars (`TUITBOT_X_API__CLIENT_ID`) > `config.toml` > built-in defaults. The `init`, `upgrade`, and `settings` subcommands are handled before general config loading since they manage their own config lifecycle.
- **Automation runtime**: `Runtime` struct spawns tokio tasks sharing a `CancellationToken`. Graceful shutdown on SIGTERM/Ctrl+C with 30s timeout. Six concurrent loops: discovery, mentions, content, threads, target monitoring, analytics. All posting loops use `LoopScheduler` for jittered intervals and `schedule_gate()` for timezone-aware active hours (analytics loop runs 24/7, no schedule gate).
- **Content frameworks**: `ReplyArchetype` (weighted random), `TweetFormat` (avoid-recent), `ThreadStructure` (random). Each provides prompt fragments injected into LLM generation. Persona opinions/experiences/content pillars enrich prompts.
- **Analytics feedback loop**: Hourly follower snapshots, 24h engagement measurement on posted content, performance scoring formula `(likes*3 + replies*5 + retweets*4) / max(impressions,1) * 1000`, epsilon-greedy topic selection (80% exploit / 20% explore).
- **Approval queue**: When `approval_mode = true`, posting queue routes actions to `approval_queue` table instead of X API. `tuitbot approve` provides interactive CLI review.
- **Storage**: SQLite WAL mode, pool of 4, `sqlx::migrate!()` for embedded migrations, 90-day configurable retention, dedup records are never deleted. Tests use `storage::init_test_db()` for in-memory SQLite.
- **Build script**: `crates/tuitbot-core/build.rs` watches `migrations/` directory for recompilation.
