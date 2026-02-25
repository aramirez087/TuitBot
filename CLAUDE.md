# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tuitbot is a local-first autonomous growth assistant for X (Twitter), evolving from
a CLI tool into a desktop app with a visual dashboard. It helps founders and indie
hackers grow their accounts organically by discovering relevant conversations, building
relationships with target accounts, replying with varied and authentic content, posting
educational tweets, publishing weekly threads, and tracking analytics to optimize
strategy over time. All data stays on the user's machine (SQLite). The dashboard
provides visual analytics, content approval, scheduling, and settings management.

**Product positioning:** Typefully is a typewriter. Tuitbot is a growth co-pilot.
The autonomous engagement loop (discovery, scoring, relationship building) is the
moat — the dashboard is the delivery mechanism.

## Build & Development Commands

```bash
# Build
cargo build                              # debug build
cargo build --release                    # release build (binary: target/release/tuitbot)

# Test
cargo test                               # run all tests
cargo test -p tuitbot-core              # test core crate only
cargo test -p tuitbot-cli               # test CLI crate only
cargo test -p tuitbot-server            # test server crate only
cargo test -p tuitbot-core scoring      # test a specific module
cargo test -p tuitbot-core -- --test-threads=1  # serial (for env-var tests)

# Lint & Format
cargo clippy --workspace                 # lint all crates
cargo fmt --all -- --check               # check formatting
cargo fmt --all                          # auto-format

# Run
cargo run -p tuitbot-cli -- init        # run a subcommand via cargo
cargo run -p tuitbot-server             # start the API server (default: 127.0.0.1:3001)
cargo install --path crates/tuitbot-cli # install binary as `tuitbot`

# Frontend (dashboard/app)
cd dashboard && npm install              # install frontend dependencies
cd dashboard && npm run dev              # dev server with HMR (default: localhost:5173)
cd dashboard && npm run build            # production build → dashboard/dist/
cd dashboard && npm run lint             # lint frontend code
cd dashboard && npm run check            # type-check (svelte-check)

# Tauri (desktop app — wraps server + frontend)
cd dashboard && npm run tauri dev        # dev mode with hot reload
cd dashboard && npm run tauri build      # production build → DMG/MSI/AppImage
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
  - `tuitbot-server-vX.Y.Z`
- Keep `release_always = false` except for one-time bootstrap scenarios.

Before handing off publish-affecting changes, run:

```bash
release-plz update --config release-plz.toml --allow-dirty
cargo package --workspace --allow-dirty
```

If either command fails, treat it as a release blocker and fix before handoff.

## Architecture

### Workspace Layout

```
crates/
  tuitbot-core/      — Library crate: all business logic (unchanged)
  tuitbot-cli/       — CLI binary: parsing, logging, dispatch to core
  tuitbot-mcp/       — MCP server: AI agent integration
  tuitbot-server/    — HTTP/WebSocket API server (axum): bridges core ↔ dashboard
dashboard/           — Svelte + Tauri frontend: desktop app shell
  src/               — Svelte components, pages, stores
  src-tauri/         — Tauri Rust sidecar (embeds tuitbot-server)
migrations/          — SQLite migrations (shared across all crates)
docs/roadmap/        — Sequential implementation prompts (01-10)
config.example.toml  — Full configuration reference
```

### Core Modules (`tuitbot-core/src/`)

| Module | Purpose |
|--------|---------|
| `config/` | Three-layer config: defaults → TOML file → env vars (`TUITBOT_` prefix, `__` separator) |
| `error.rs` | Per-domain error enums: `ConfigError`, `XApiError`, `LlmError`, `StorageError`, `ScoringError` |
| `x_api/` | X API v2 client behind `XApiClient` trait; OAuth 2.0 PKCE auth; tier detection |
| `llm/` | `LlmProvider` trait with `OpenAiCompatProvider` (OpenAI + Ollama) and `AnthropicProvider` |
| `scoring/` | 6-signal heuristic scoring — keyword relevance, targeted followers (bell curve), recency, engagement, reply count, content type |
| `storage/` | SQLite via SQLx — WAL mode, pool of 4, embedded migrations, `init_test_db()` for tests. Submodules: `analytics`, `approval_queue`, `author_interactions`, `target_accounts`, `replies`, `threads`, `tweets`, `action_log`, `rate_limits`, `cleanup`, `strategy`, `scheduled_content`, `cursors` |
| `strategy/` | Weekly report engine — `metrics.rs` (date-ranged queries over existing tables), `recommendations.rs` (8-rule deterministic engine), `report.rs` (orchestration + ISO week computation) |
| `safety/` | Deduplication, rate limit enforcement, banned phrase filtering, per-author reply limits, self-reply prevention |
| `content/` | Content generation via LLM + `frameworks.rs` (reply archetypes, tweet formats, thread structures with weighted random selection) |
| `automation/` | Runtime with `CancellationToken` + 6 concurrent loops: discovery, mentions, content, threads, target monitoring, analytics. Also: posting queue with optional approval mode, `schedule.rs` (timezone-aware active hours gating via `chrono-tz`), `scheduler.rs` (`LoopScheduler` with jitter for human-like timing) |
| `startup.rs` | Agent startup orchestration |

### Server Layer (`tuitbot-server/src/`)

| Module | Purpose |
|--------|---------|
| `routes/` | Axum route handlers grouped by domain: `analytics`, `approval`, `content`, `targets`, `settings`, `activity`, `strategy` |
| `state.rs` | Shared `AppState` — holds `DbPool`, config, runtime handle, broadcast channels |
| `ws.rs` | WebSocket hub for real-time events (activity feed, approval notifications, runtime status) |
| `auth.rs` | Local bearer token auth (generated on first run, stored in config dir) |
| `error.rs` | API error types mapping core errors → HTTP status codes |

### Dashboard (`dashboard/src/`)

| Area | Purpose |
|------|---------|
| `routes/` | SvelteKit pages: dashboard, activity, approval, calendar, targets, settings, strategy |
| `lib/components/` | Reusable UI components (charts, cards, tables, forms) |
| `lib/stores/` | Svelte stores wrapping API calls + WebSocket subscriptions |
| `lib/api.ts` | Typed API client (auto-generated from server OpenAPI spec or hand-written) |

### Key Patterns

- **Trait-based testing**: All external services (`XApiClient`, `LlmProvider`) are behind `async_trait` traits. Tests use trait-based mocks + `wiremock` for HTTP fixtures.
- **Error handling**: `thiserror` in core library (typed enums per domain), `anyhow` in CLI and server binaries.
- **LLM providers**: Thin reqwest wrappers — `openai_compat` serves both OpenAI and Ollama via compatible API, `anthropic` uses native Messages API.
- **Config layering**: CLI flags > env vars (`TUITBOT_X_API__CLIENT_ID`) > `config.toml` > built-in defaults. The `init`, `upgrade`, and `settings` subcommands are handled before general config loading since they manage their own config lifecycle.
- **Automation runtime**: `Runtime` struct spawns tokio tasks sharing a `CancellationToken`. Graceful shutdown on SIGTERM/Ctrl+C with 30s timeout. Loop behavior is mode-aware: in Autopilot, all loops run (discovery, mentions, content, threads, target monitoring, analytics, posting queue, approval poster); in Composer mode, only read-only discovery, posting queue, approval poster, and analytics run. All posting loops use `LoopScheduler` for jittered intervals and `schedule_gate()` for timezone-aware active hours (analytics loop runs 24/7, no schedule gate).
- **Content frameworks**: `ReplyArchetype` (weighted random), `TweetFormat` (avoid-recent), `ThreadStructure` (random). Each provides prompt fragments injected into LLM generation. Persona opinions/experiences/content pillars enrich prompts.
- **Analytics feedback loop**: Hourly follower snapshots, 24h engagement measurement on posted content, performance scoring formula `(likes*3 + replies*5 + retweets*4) / max(impressions,1) * 1000`, epsilon-greedy topic selection (80% exploit / 20% explore).
- **Strategy reports**: Weekly aggregation layer computing derived metrics (follower delta, acceptance rate, topic performance) from existing tables. 8-rule deterministic recommendation engine (promote winners, kill losers, detect stalls, celebrate high-performers). Reports stored in `strategy_reports` table with JSON columns for topics and recommendations. Server computes on-the-fly for the current (in-progress) week; historical reports are cached.
- **Approval queue**: When `approval_mode = true` (or in Composer mode, where it is always implicit), posting queue routes actions to `approval_queue` table instead of X API. `tuitbot approve` provides interactive CLI review. The dashboard provides a visual alternative.
- **Storage**: SQLite WAL mode, pool of 4, `sqlx::migrate!()` for embedded migrations, 90-day configurable retention, dedup records are never deleted. Tests use `storage::init_test_db()` for in-memory SQLite.
- **Build script**: `crates/tuitbot-core/build.rs` watches `migrations/` directory for recompilation.
- **Server ↔ Core boundary**: The server crate is a thin HTTP layer over `tuitbot-core`. It owns no business logic — only serialization, routing, WebSocket fan-out, and auth. All domain logic stays in core.
- **Real-time updates**: The server uses `tokio::sync::broadcast` channels to push events (new actions, approval items, follower changes) to WebSocket clients. The automation runtime publishes events to these channels.
- **Frontend conventions**: Svelte 5 with SvelteKit, TypeScript strict mode, TailwindCSS for styling. Prefer server-side rendering disabled (SPA mode) since Tauri serves it locally. Use Svelte stores for state management, not external libraries.
- **Tauri integration**: The Tauri sidecar starts `tuitbot-server` as a child process on app launch. The frontend connects to `http://localhost:3001`. Tauri handles system tray, auto-start on login, and native OS integration.
