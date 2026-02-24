# Architecture

## Workspace crates

- `tuitbot-core`: business logic, storage, API integrations, safety.
- `tuitbot-server`: Axum-based HTTP API and WebSocket event hub serving endpoints for internal UIs.
- `tuitbot-cli`: command-line UX and runtime entrypoints.
- `tuitbot-mcp`: MCP tool surface and transport wiring.

## Frontend Stack & Modes
- **Dashboard UI**: SvelteKit single-page application built out of `dashboard/`. Connects to `tuitbot-server`.
- **Tauri Integration**: Wraps the Dashboard and `tuitbot-server` into a single standalone native Desktop App package.
- **Docker/Cloud (Multi-Tenant vs Self-host)**: The identical Dashboard can be served statically by the Axum backend when running outside the Tauri environment via the `TUITBOT_DASHBOARD_DIR` environment flag, unlocking pure self-hosted environments. Additionally, the backend supports `tuitbot-server --mode cloud` for a Stripe-gated multi-tenant mode spanning multiple SQLite shards on `tuitbot.dev`.

## Storage

- SQLite via SQLx
- Migrations embedded from crate-local migrations directory
- Single-process lock prevents overlapping run/tick instances

## Runtime loops

- discovery
- mentions
- target monitoring
- content posting
- thread publishing
- analytics snapshots

## Design principles

- conservative automation defaults
- explicit approval and guardrails
- deterministic CLI interfaces for scheduler and agent integration
