# AGENTS.md

This is the canonical repository guide for coding agents. Keep shared instructions here so agent tooling has one root file to read before exploring the repo. `CLAUDE.md` should remain a small Claude-specific overlay.

## Project Description

Tuitbot is a Rust workspace plus a Svelte/Tauri dashboard for an AI-powered X growth copilot.

- `tuitbot-core` contains the business logic for discovery, drafting, approval, posting, analytics, content sources, safety, and strategy.
- `tuitbot-cli` is the operator entrypoint for setup, auth, testing, run/tick, backup/restore, and MCP serving.
- `tuitbot-mcp` exposes typed MCP tools for AI agents across `readonly`, `api-readonly`, `write`, and `admin` profiles.
- `tuitbot-server` is a thin Axum and WebSocket API used by the dashboard and desktop app.
- `dashboard/` is the Svelte 5 SPA packaged by Tauri for desktop and served by the backend for self-hosted mode.

Two operating modes matter when changing behavior:

- `autopilot`: scheduled loops can discover, draft, queue, and post autonomously.
- `composer`: autonomous posting loops are disabled; users drive content creation while AI assist stays available.

## Start Here

Before broad repo exploration, read the closest high-signal doc:

- `docs/architecture.md` for the three-layer model, crate responsibilities, and deployment modes.
- `docs/mcp-reference.md` for MCP tool behavior, profiles, and mutation policy.
- `docs/operations.md` for runbooks, profile selection, and operational safety.
- `README.md` for product intent, user workflows, and top-level commands.

## Repository Map

- `crates/tuitbot-core/src/toolkit/`: stateless X API utilities over `&dyn XApiClient`
- `crates/tuitbot-core/src/workflow/`: stateful orchestration using DB and optional LLM
- `crates/tuitbot-core/src/automation/`: scheduled loops, background workers, posting queue
- `crates/tuitbot-core/src/storage/`, `x_api/`, `llm/`, `config/`, `strategy/`, `context/`: foundations
- `crates/tuitbot-mcp/src/`: MCP contract, profile wiring, handlers, telemetry, manifests
- `crates/tuitbot-server/src/`: HTTP routes, auth/session handling, serialization, WebSocket fan-out
- `dashboard/src/`: Svelte routes, components, stores, tests
- `migrations/` and `crates/*/migrations/`: SQL schema changes
- `scripts/`: manifest generation/checking and release helpers

## Common Commands

```bash
# Workspace
cargo build
cargo fmt --all
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings

# Server
cargo run -p tuitbot-server
cargo run -p tuitbot-server -- --host 0.0.0.0

# Dashboard
cd dashboard && npm run dev
cd dashboard && npm run check
cd dashboard && npx vitest run
cd dashboard && npm run build

# Desktop
cd dashboard && npm run tauri dev
cd dashboard && npm run tauri build
```

## Task Routing

- Rust or business-logic change: start in `tuitbot-core`, preserve layer boundaries, then run the workspace Rust checks.
- MCP change: also run `cargo test -p tuitbot-mcp eval_harness` and `bash scripts/check-mcp-manifests.sh`.
- Server or API change: keep business logic in `tuitbot-core`; `tuitbot-server` owns routing, auth, serialization, and fan-out only.
- Frontend change: work in `dashboard/`; use Svelte 5 runes and run `npm run check` plus `npx vitest run`.
- Release or packaging change under `crates/`: keep `Cargo.lock` committed, ensure local `path` deps have explicit versions, then run `release-plz update --config release-plz.toml --allow-dirty` and `cargo package --workspace --allow-dirty`.

## Architecture Rules

- Dependency direction is `toolkit <- workflow <- automation`. No upward or skip-level imports.
- Toolkit code is stateless over `&dyn XApiClient`. No DB, no LLM, no policy enforcement.
- Workflow code composes toolkit with DB and LLM and owns policy and safety checks.
- Autopilot loops orchestrate scheduled work and must not call `XApiClient` directly.
- MCP handlers and HTTP handlers stay thin: parse input, delegate, wrap output.
- `tuitbot-server` owns zero business logic.

## Conventions

- Use `thiserror` in `tuitbot-core` with typed domain errors. Use `anyhow` only in binary crates.
- Keep changes cross-platform: no hard-coded `/tmp`, no assumed `/` separators, no Unix-only newline assertions.
- File size targets: max 500 lines per `.rs` file and max 400 lines per Svelte page component. Split modules or components before they sprawl.
- Frontend stack is Svelte 5 runes, TypeScript strict, TailwindCSS, SPA mode.
- All warnings are blockers. Treat compiler and clippy warnings as failures.
- Keep agent guidance high-signal and stable. Put deep operational detail in docs instead of duplicating it across agent files.

## Completion Checklist

- The change is scoped to the correct layer or crate.
- Relevant checks for the touched area pass locally.
- Tests or docs are updated when behavior or operator workflows change.
- Shared agent guidance changes land in `AGENTS.md`; Claude-only deltas stay in `CLAUDE.md`.
