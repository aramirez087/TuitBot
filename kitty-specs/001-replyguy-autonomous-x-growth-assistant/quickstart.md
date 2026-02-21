# Developer Quickstart: ReplyGuy

**Feature**: `001-replyguy-autonomous-x-growth-assistant`
**Date**: 2026-02-21

## Prerequisites

- Rust 1.75+ (`rustup update stable`)
- SQLite development headers (usually bundled via SQLx)
- An X (Twitter) developer account with API access
- An API key for at least one LLM provider (OpenAI, Anthropic, or Ollama running locally)

## Repository Structure

```
ReplyGuy/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── replyguy-core/                  # Library crate — all business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├── config/
│   │       │   ├── mod.rs              # Config types + loading
│   │       │   └── defaults.rs         # Built-in defaults
│   │       ├── x_api/
│   │       │   ├── mod.rs              # X API client trait + types
│   │       │   ├── client.rs           # Reqwest-based X API v2 client
│   │       │   ├── auth.rs             # OAuth 2.0 PKCE flow
│   │       │   ├── types.rs            # Tweet, User, Mention types
│   │       │   └── tier.rs             # API tier detection logic
│   │       ├── llm/
│   │       │   ├── mod.rs              # LlmProvider trait + types
│   │       │   ├── openai_compat.rs    # OpenAI + Ollama provider
│   │       │   ├── anthropic.rs        # Anthropic native provider
│   │       │   └── factory.rs          # Provider factory from config
│   │       ├── scoring/
│   │       │   ├── mod.rs              # ScoringEngine + heuristics
│   │       │   └── signals.rs          # Individual scoring signals
│   │       ├── content/
│   │       │   └── generator.rs        # ContentGenerator (reply, tweet, thread)
│   │       ├── storage/
│   │       │   ├── mod.rs              # Storage trait + types
│   │       │   ├── db.rs               # SQLx pool init + migrations
│   │       │   ├── tweets.rs           # Tweet CRUD operations
│   │       │   ├── replies.rs          # Reply CRUD operations
│   │       │   ├── threads.rs          # Thread CRUD operations
│   │       │   ├── rate_limits.rs      # Rate limit tracking
│   │       │   ├── action_log.rs       # Action log operations
│   │       │   └── cleanup.rs          # Data retention cleanup
│   │       ├── automation/
│   │       │   ├── mod.rs              # Runtime orchestrator
│   │       │   ├── mentions_loop.rs    # Mentions monitoring loop
│   │       │   ├── discovery_loop.rs   # Tweet discovery + reply loop
│   │       │   ├── content_loop.rs     # Original content posting loop
│   │       │   ├── thread_loop.rs      # Thread generation loop
│   │       │   └── scheduler.rs        # Interval + jitter scheduling
│   │       ├── safety/
│   │       │   ├── mod.rs              # Rate limiter + dedup checker
│   │       │   └── dedup.rs            # Duplicate reply prevention
│   │       └── error.rs                # thiserror error types
│   │
│   └── replyguy-cli/                   # Binary crate — CLI entry point
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                 # Clap app + command dispatch
│           └── commands/
│               ├── mod.rs
│               ├── run.rs              # `replyguy run`
│               ├── auth.rs             # `replyguy auth`
│               ├── test.rs             # `replyguy test`
│               ├── discover.rs         # `replyguy discover`
│               ├── mentions.rs         # `replyguy mentions`
│               ├── post.rs             # `replyguy post`
│               ├── thread.rs           # `replyguy thread`
│               └── score.rs            # `replyguy score`
│
├── migrations/                         # SQLx embedded migrations
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
        ├── tweets.json                 # Recorded API responses
        └── config.toml                 # Test configuration
```

## Getting Started

### 1. Clone and build

```bash
git clone <repo-url>
cd ReplyGuy
cargo build
```

### 2. Set up configuration

```bash
mkdir -p ~/.replyguy
cp config.example.toml ~/.replyguy/config.toml
# Edit with your credentials
```

### 3. Authenticate with X

```bash
cargo run --bin replyguy -- auth
```

### 4. Validate setup

```bash
cargo run --bin replyguy -- test
```

### 5. Test discovery (dry run)

```bash
cargo run --bin replyguy -- discover --dry-run
```

### 6. Run the agent

```bash
cargo run --bin replyguy -- run --verbose
```

## Development Workflow

### Running tests

```bash
# All tests
cargo test --workspace

# Core library only
cargo test -p replyguy-core

# Integration tests
cargo test -p replyguy-core --test '*'

# With logging
RUST_LOG=debug cargo test --workspace -- --nocapture
```

### Quality checks

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo audit
```

### Database migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --features sqlite

# Create a new migration
sqlx migrate add <migration_name>

# Migrations are auto-applied at startup via sqlx::migrate!()
```

## Key Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `clap` | CLI argument parsing |
| `reqwest` | HTTP client (X API + LLM providers) |
| `serde` + `toml` | Configuration deserialization |
| `sqlx` | SQLite database (async, embedded migrations) |
| `tracing` + `tracing-subscriber` | Structured logging |
| `oauth2` | OAuth 2.0 PKCE flow |
| `thiserror` | Typed error definitions (library) |
| `anyhow` | Ergonomic errors (binary) |
| `wiremock` | HTTP mocking for tests |
| `rand` | Randomized delays and jitter |
| `chrono` | Timestamp handling |
