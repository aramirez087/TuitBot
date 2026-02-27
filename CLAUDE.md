# CLAUDE.md

## Project Overview

Tuitbot — local-first autonomous growth assistant for X (Twitter). CLI + desktop app (Tauri) with visual dashboard. Discovers conversations, builds relationships, replies authentically, posts tweets/threads, tracks analytics. All data on user's machine (SQLite).

**Positioning:** Typefully is a typewriter. Tuitbot is a growth co-pilot. The autonomous engagement loop is the moat — the dashboard is the delivery mechanism.

## Commands

```bash
# Build & Run
cargo build                              # debug
cargo build --release                    # release → target/release/tuitbot
cargo run -p tuitbot-server              # API server (127.0.0.1:3001)
cargo run -p tuitbot-server -- --host 0.0.0.0  # LAN mode (all interfaces)

# Test
cargo test                               # all tests
cargo test -p tuitbot-core scoring       # specific module
cargo test -p tuitbot-core -- --test-threads=1  # serial (env-var tests)

# Lint & Format
cargo fmt --all                          # auto-format
cargo clippy --workspace -- -D warnings  # lint

# Frontend
cd dashboard && npm run dev              # dev server (localhost:5173)
cd dashboard && npm run build            # production build
cd dashboard && npm run check            # type-check (svelte-check)

# Desktop
cd dashboard && npm run tauri dev        # dev mode
cd dashboard && npm run tauri build      # production → DMG/MSI/AppImage
```

## Mandatory CI Checklist

Run before handing off **any** Rust change — fix all failures:

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

- Always run `cargo fmt --all` — never assume hand-formatted code is correct.
- All warnings are release blockers. Do not finish until this passes.

## Crates.io Release Discipline

When changing crates under `crates/`:

- Keep `Cargo.lock` committed. Every local `path` dep must include explicit `version`.
- Maintain package metadata: `description`, `license`, `repository`, `homepage`, `documentation`, `keywords`.
- Do not change `release-plz.toml` tag conventions without approval.

Validate before handoff:

```bash
release-plz update --config release-plz.toml --allow-dirty
cargo package --workspace --allow-dirty
```

## File Size Limits

- **Rust:** Max 500 lines per `.rs` file. Exceed → convert to module directory (`foo.rs` → `foo/mod.rs` + submodules). Follow `commands/settings/` and `commands/init/` patterns: thin `mod.rs` orchestrator, logic in focused submodules. Tests in `tests.rs` submodule when >100 lines.
- **Svelte:** Max 400 lines per `+page.svelte`. Extract sections into sibling `*Section.svelte` files. Parent passes callbacks/state via `$props()`; sections import their own stores/icons. Follow settings page pattern.

## Architecture

```
crates/
  tuitbot-core/    — All business logic (library crate)
  tuitbot-cli/     — CLI binary: parsing, logging, dispatch
  tuitbot-mcp/     — MCP server: AI agent integration
  tuitbot-server/  — Axum HTTP/WS API: thin layer over core
dashboard/         — Svelte 5 + SvelteKit + Tauri frontend
  src-tauri/       — Tauri sidecar (embeds tuitbot-server)
migrations/        — SQLite migrations (shared across crates)
```

### Three-Layer Model (tuitbot-core)

Business logic in `tuitbot-core` follows a strict three-layer architecture:

| Layer | Module | Role | Dependencies |
|-------|--------|------|--------------|
| **Toolkit** (L1) | `core/toolkit/` | Stateless X API utilities (`read.rs`, `write.rs`, `engage.rs`, `media.rs`) | `&dyn XApiClient` only — no DB, no LLM |
| **Workflow** (L2) | `core/workflow/` | Stateful composites (`discover`, `draft`, `queue`, `publish`, `thread_plan`, `orchestrate`) | DB + optional LLM. Calls Toolkit only. |
| **Autopilot** (L3) | `core/automation/` | Scheduled loops with jitter, circuit breaking, graceful shutdown | Calls Workflow + Toolkit. Never XApiClient directly. |

Dependency rules: Toolkit ← Workflow ← Autopilot. No upward or skip-level imports. MCP handlers are thin adapters (param parse → delegate → envelope).

### Key Modules

| Module | Notes |
|--------|-------|
| `core/auth/` | Passphrase generation (EFF wordlist, bcrypt), session CRUD (SHA-256 hashed tokens in SQLite) |
| `core/toolkit/` | Stateless X API wrappers: `read.rs`, `write.rs`, `engage.rs`, `media.rs` over `&dyn XApiClient` |
| `core/workflow/` | Stateful composites: `discover.rs`, `draft.rs`, `queue.rs`, `publish.rs`, `thread_plan.rs`, `orchestrate.rs` |
| `core/config/` | Defaults → TOML → env vars (`TUITBOT_` prefix, `__` separator) |
| `core/x_api/` | `XApiClient` trait; OAuth 2.0 PKCE; tier detection |
| `core/llm/` | `LlmProvider` trait; `OpenAiCompatProvider` (OpenAI + Ollama), `AnthropicProvider` |
| `core/scoring/` | 6-signal heuristic: keyword, followers (bell curve), recency, engagement, reply count, content type |
| `core/storage/` | SQLx + SQLite WAL, pool of 4, `sqlx::migrate!()`, `init_test_db()` for tests |
| `core/automation/` | `Runtime` with `CancellationToken` + concurrent loops. Mode-aware: Autopilot (all loops) vs Composer (read-only + posting queue). `LoopScheduler` for jitter, `schedule_gate()` for timezone-aware hours |
| `core/safety/` | Dedup, rate limits, banned phrases, per-author limits, self-reply prevention |
| `core/content/` | LLM generation + `frameworks.rs` (reply archetypes, tweet formats, thread structures — weighted random selection) |
| `core/strategy/` | Weekly reports: metrics, 8-rule recommendation engine, ISO week computation |
| `server/auth/` | Multi-strategy middleware (Bearer token + session cookie), login/logout/status routes, rate limiting |
| `server/routes/` | Handlers by domain: analytics, approval, content, targets, settings, activity, strategy |
| `server/ws.rs` | Broadcast channels → WebSocket fan-out for real-time events (token or cookie auth) |

### Patterns to Follow

- **Trait-based testing**: External services behind `async_trait`. Tests use mocks + `wiremock`.
- **Error handling**: `thiserror` in core (typed enums per domain), `anyhow` in binaries.
- **Config precedence**: CLI flags > env vars > `config.toml` > defaults. `init`/`upgrade`/`settings` subcommands handle config before general loading.
- **Server boundary**: Server owns zero business logic — only routing, serialization, WebSocket fan-out, auth. All domain logic in core.
- **Dual auth**: Bearer token (Tauri/API/MCP) and session cookie (web/LAN) coexist. Middleware checks bearer first, then cookie. CSRF token required for mutating cookie-auth requests. `--host 0.0.0.0` enables LAN access.
- **Frontend**: Svelte 5, TypeScript strict, TailwindCSS, SPA mode (no SSR). Stores for state — no external libraries.
- **Approval queue**: When `approval_mode = true` (always in Composer mode), posting routes to `approval_queue` table instead of X API.
- **Storage**: WAL mode, 90-day retention, dedup records never deleted. `build.rs` watches `migrations/` for recompilation.
- **Tauri**: Sidecar starts `tuitbot-server` on launch. Frontend connects to `localhost:3001`.
