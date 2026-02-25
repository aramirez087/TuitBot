# Architecture

## Workspace crates

- `tuitbot-core`: business logic, storage, API integrations, safety, strategy reports.
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

The runtime spawns concurrent loops whose behavior depends on the operating mode:

| Loop | Autopilot | Composer |
|---|---|---|
| Discovery | Active — scores and queues replies | Read-only — scores tweets for Discovery Feed |
| Mentions | Active | Disabled |
| Target monitoring | Active | Disabled |
| Content posting | Active | Disabled |
| Thread publishing | Active | Disabled |
| Posting queue | Active | Active |
| Approval poster | Active | Active |
| Analytics snapshots | Active | Active |
| Token refresh | Active | Active |

Strategy reports run weekly (and on-demand via API) in both modes.

## AI Assist endpoints

Stateless generation endpoints under `/api/assist/` provide on-demand content creation. Each endpoint accepts a topic or draft text and returns generated content using the configured LLM and persona. Available in both modes but primarily designed for Composer workflows.

## Draft lifecycle

Drafts follow a linear lifecycle: `draft` (created or generated) -> `scheduled` (assigned a publish time) -> `posted` (routed through the approval queue and posting pipeline). Drafts are mode-independent and persist across mode switches.

## Discovery Feed

The Discovery Feed exposes scored tweets from the read-only discovery loop. Users browse conversations, compose replies (optionally with AI Assist), and queue them for posting through the approval queue. Available via `/api/discovery/feed` endpoints and the dashboard Discovery page.

## Design principles

- dual operating modes: fully autonomous (Autopilot) and user-driven (Composer)
- conservative automation defaults
- explicit approval and guardrails
- deterministic CLI interfaces for scheduler and agent integration
