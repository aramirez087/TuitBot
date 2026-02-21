# Implementation Plan: ReplyGuy — Autonomous X Growth Assistant

**Branch**: `001-replyguy-autonomous-x-growth-assistant` | **Date**: 2026-02-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md`

## Summary

ReplyGuy is a cross-platform Rust CLI that autonomously grows a user's X (Twitter) presence by discovering relevant tweets, replying with valuable content, posting educational tweets, and publishing weekly threads. It runs as a lightweight background daemon with four concurrent automation loops, driven by a business profile in `config.toml`.

The implementation uses a Cargo workspace (library + binary), a thin reqwest-based wrapper for X API v2 and LLM providers (OpenAI/Anthropic/Ollama), SQLx with embedded SQLite for persistence, and a purely heuristic scoring engine for tweet evaluation.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: Tokio (async), Clap (CLI), Reqwest (HTTP), Serde + TOML (config), SQLx (database), Tracing (logging), OAuth2 (auth), thiserror/anyhow (errors)
**Storage**: SQLite via SQLx (WAL mode, embedded migrations, connection pool of 4)
**Testing**: cargo test + wiremock for HTTP fixtures; trait-based mocks for unit tests
**Target Platform**: Cross-platform — Linux, macOS, Windows 10+
**Project Type**: Cargo workspace — `replyguy-core` (library) + `replyguy-cli` (binary)
**Performance Goals**: <50MB memory, near-zero idle CPU, <1s startup for local operations
**Constraints**: Single static binary per platform, no external runtime dependencies
**Scale/Scope**: Single-user CLI, ~2K lines core library, ~500 lines CLI

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Constitution Requirement | Status | Notes |
|---|---|---|
| Rust 1.75+ (edition 2021) | PASS | Workspace uses edition 2021 |
| Tokio async runtime | PASS | All async code runs on Tokio |
| Clap for CLI parsing | PASS | Used in replyguy-cli |
| Reqwest for HTTP | PASS | X API + LLM providers |
| SQLx + SQLite | PASS | Embedded migrations, WAL mode |
| Serde for serialization | PASS | Config + JSON parsing |
| Tracing for logging | PASS | Structured logs, configurable verbosity |
| OAuth2 crate for PKCE | PASS | X API authentication |
| cargo test + integration tests | PASS | Unit + integration with wiremock |
| 100% coverage for critical paths | PASS | Auth, API, persistence targeted |
| No unwrap() in production | PASS | thiserror in library, anyhow in binary |
| No unnecessary unsafe | PASS | No unsafe code needed |
| cargo clippy -D warnings | PASS | CI quality gate |
| cargo fmt --check | PASS | CI quality gate |
| cargo audit | PASS | CI quality gate |
| Public APIs have /// doc comments | PASS | All pub items documented |
| Cross-platform (Linux, macOS, Windows) | PASS | No platform-specific code |
| Single static binary | PASS | SQLite embedded via SQLx |

All constitution checks pass. No violations or exceptions needed.

## Project Structure

### Documentation (this feature)

```
kitty-specs/001-replyguy-autonomous-x-growth-assistant/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research findings
├── data-model.md        # Entity definitions and schema
├── quickstart.md        # Developer quickstart guide
├── meta.json            # Feature metadata
├── contracts/
│   └── cli-interface.md # CLI commands and config contract
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Work packages (created by /spec-kitty.tasks)
```

### Source Code (repository root)

```
ReplyGuy/
├── Cargo.toml                          # Workspace definition
├── crates/
│   ├── replyguy-core/                  # Library — all business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs                # thiserror error types
│   │       ├── config/
│   │       │   ├── mod.rs              # Config types + layered loading
│   │       │   └── defaults.rs         # Built-in defaults
│   │       ├── x_api/
│   │       │   ├── mod.rs              # XApiClient trait
│   │       │   ├── client.rs           # Reqwest implementation
│   │       │   ├── auth.rs             # OAuth 2.0 PKCE (manual + callback)
│   │       │   ├── types.rs            # Tweet, User, Mention, Metrics
│   │       │   └── tier.rs             # API tier detection
│   │       ├── llm/
│   │       │   ├── mod.rs              # LlmProvider trait
│   │       │   ├── openai_compat.rs    # OpenAI + Ollama
│   │       │   ├── anthropic.rs        # Anthropic native API
│   │       │   └── factory.rs          # Provider factory
│   │       ├── scoring/
│   │       │   ├── mod.rs              # ScoringEngine
│   │       │   └── signals.rs          # Scoring signal functions
│   │       ├── content/
│   │       │   └── generator.rs        # ContentGenerator
│   │       ├── storage/
│   │       │   ├── mod.rs              # Database init + pool
│   │       │   ├── tweets.rs           # Discovered tweet operations
│   │       │   ├── replies.rs          # Reply operations
│   │       │   ├── threads.rs          # Thread operations
│   │       │   ├── rate_limits.rs      # Rate limit tracking
│   │       │   ├── action_log.rs       # Action log operations
│   │       │   └── cleanup.rs          # Retention cleanup
│   │       ├── automation/
│   │       │   ├── mod.rs              # Runtime orchestrator
│   │       │   ├── mentions_loop.rs
│   │       │   ├── discovery_loop.rs
│   │       │   ├── content_loop.rs
│   │       │   ├── thread_loop.rs
│   │       │   └── scheduler.rs        # Interval + jitter
│   │       └── safety/
│   │           ├── mod.rs              # Rate limiter
│   │           └── dedup.rs            # Duplicate prevention
│   │
│   └── replyguy-cli/                   # Binary — CLI entry point
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                 # Clap app + dispatch
│           └── commands/
│               ├── mod.rs
│               ├── run.rs
│               ├── auth.rs
│               ├── test.rs
│               ├── discover.rs
│               ├── mentions.rs
│               ├── post.rs
│               ├── thread.rs
│               └── score.rs
│
├── migrations/
│   └── 20260221000000_initial_schema.sql
│
└── tests/
    ├── integration/
    │   ├── discovery_test.rs
    │   ├── mentions_test.rs
    │   ├── scoring_test.rs
    │   ├── content_test.rs
    │   └── storage_test.rs
    └── fixtures/
        ├── tweets.json
        └── config.toml
```

**Structure Decision**: Cargo workspace with two crates. `replyguy-core` contains all business logic behind traits (testable with mocks). `replyguy-cli` is a thin layer that parses CLI args and delegates to core. This separation enables independent testing and keeps the binary crate minimal.

## Architecture

### Module Dependency Graph

```
replyguy-cli
  └── replyguy-core
        ├── config        (standalone — no deps on other modules)
        ├── error         (standalone)
        ├── x_api         (depends on: config, error)
        ├── llm           (depends on: config, error)
        ├── storage       (depends on: config, error, x_api::types)
        ├── scoring       (depends on: config, x_api::types)
        ├── content       (depends on: config, llm)
        ├── safety        (depends on: config, storage)
        └── automation    (depends on: all above)
```

### Data Flow

```
config.toml ──► Config ──► Runtime Orchestrator
                              │
                ┌─────────────┼─────────────────┐
                │             │                 │
          Mentions Loop   Discovery Loop    Content/Thread Loop
                │             │                 │
                │         ┌───┴───┐             │
                │      X Search  Scoring     LLM Provider
                │         │    Engine           │
                │         └───┬───┘             │
                │             │                 │
                └─────────────┼─────────────────┘
                              │
                    Safety / Rate Limiter
                              │
                         X API Client
                              │
                      SQLite (persist)
```

### Key Design Decisions

1. **Trait-based abstractions**: `XApiClient`, `LlmProvider`, and storage operations are behind traits. Production uses reqwest/SQLx implementations; tests use mocks.

2. **Two LLM implementations, not three**: OpenAI and Ollama share the OpenAI-compatible endpoint format. Only Anthropic needs a separate native implementation.

3. **Heuristic scoring only**: No LLM calls in the scoring path. Scores are computed from keyword relevance, author follower count, tweet recency, and engagement rate — all available from the X API response.

4. **Serialized posting**: All loops feed actions through a shared posting queue to prevent race conditions and ensure rate limits are respected across loops.

5. **Graceful tier degradation**: At startup, the agent probes the X API to determine the user's tier. On Free tier, the discovery loop is disabled and the user is informed. All other loops continue to function.

6. **Config layering**: CLI flags > env vars (prefixed `REPLYGUY_`) > `config.toml` > built-in defaults. Secrets can live in env vars to avoid committing them.
