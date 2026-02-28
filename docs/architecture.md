# Architecture

## Three-Layer Model

Tuitbot's business logic in `tuitbot-core` is organized into three distinct layers with strict dependency rules:

```
Consumers: MCP Server, HTTP Server, CLI, Tests
                      |
  ┌───────────────────┴───────────────────────────┐
  │  Layer 3: AUTOPILOT  (core::automation/)       │
  │  Scheduled loops (discovery, mentions, content, │
  │  thread, target, analytics, approval, posting)  │
  │  Calls: Workflow + Toolkit. Never XApiClient.   │
  └───────────────────┬───────────────────────────┘
                      |
  ┌───────────────────┴───────────────────────────┐
  │  Layer 2: WORKFLOW  (core::workflow/)           │
  │  Stateful composites: discover, draft, queue,  │
  │  publish, thread_plan, orchestrate             │
  │  Requires: DB + optional LLM.                  │
  │  Calls: Toolkit only.                          │
  └───────────────────┬───────────────────────────┘
                      |
  ┌───────────────────┴───────────────────────────┐
  │  Layer 1: TOOLKIT  (core::toolkit/)            │
  │  Stateless X API utilities: read, write,       │
  │  engage, media                                 │
  │  Requires: &dyn XApiClient only. No DB/LLM.   │
  └───────────────────┬───────────────────────────┘
                      |
  ┌───────────────────┴───────────────────────────┐
  │  Foundation: x_api::XApiClient, storage, llm   │
  └────────────────────────────────────────────────┘
```

### Layer 1 — Toolkit (`core::toolkit/`)

Stateless utility functions over `&dyn XApiClient`. Every function takes a trait reference and returns typed results. No DB, no LLM, no policy enforcement.

| Module | Functions |
|--------|-----------|
| `read.rs` | `search_tweets`, `get_tweet`, `get_user_by_username`, `get_user_by_id`, `get_users_by_ids`, `get_user_mentions`, `get_user_tweets`, `get_home_timeline`, `get_followers`, `get_following`, `get_liked_tweets`, `get_bookmarks`, `get_tweet_liking_users`, `get_me` |
| `write.rs` | `post_tweet`, `reply_to_tweet`, `quote_tweet`, `delete_tweet`, `post_thread` |
| `engage.rs` | `like_tweet`, `unlike_tweet`, `follow_user`, `unfollow_user`, `retweet`, `unretweet`, `bookmark_tweet`, `unbookmark_tweet` |
| `media.rs` | `upload_media` |

Write and engage functions are **raw** — no policy gate. Consumers call `workflow::policy` before calling toolkit writes.

### Layer 2 — Workflow (`core::workflow/`)

Stateful composite operations combining toolkit functions with DB and LLM. These are the reusable building blocks that both MCP handlers and autopilot loops consume.

| Module | Purpose |
|--------|---------|
| `discover.rs` | Search → score → persist → rank pipeline |
| `draft.rs` | Fetch candidates → LLM generation → safety checks |
| `queue.rs` | Validate → safety-check → route to approval or execute |
| `publish.rs` | Thin wrappers over toolkit write functions |
| `thread_plan.rs` | LLM thread generation + hook analysis |
| `orchestrate.rs` | Deterministic discover → draft → queue cycle |

### Layer 3 — Autopilot (`core::automation/`)

Scheduled orchestration. Runs background loops that invoke workflow and toolkit functions on a timer with jitter, circuit breaking, and graceful shutdown.

| Module | Purpose |
|--------|---------|
| `discovery_loop.rs` | Search and queue replies to matching tweets |
| `mentions_loop.rs` | Monitor @-mentions and generate replies |
| `content_loop.rs` | Generate and post educational tweets |
| `thread_loop.rs` | Generate and post multi-tweet threads |
| `target_loop.rs` | Monitor target accounts for engagement |
| `analytics_loop.rs` | Snapshot follower counts and engagement |
| `posting_queue.rs` | Serialized posting queue for concurrent loops |
| `approval_poster.rs` | Execute approved items from the queue |

### Dependency Rules

```
Autopilot ──uses──> Workflow ──uses──> Toolkit ──uses──> XApiClient trait
```

1. Toolkit MUST NOT import from `workflow::` or `automation::`
2. Workflow MUST NOT import from `automation::`
3. Workflow MUST NOT use `XApiClient` directly — only through `toolkit::*`
4. Autopilot MUST NOT use `XApiClient` directly — only through `toolkit::*` or `workflow::*`
5. MCP handlers MUST NOT contain business logic — only parameter parsing + envelope wrapping + delegation

## Workspace Crates

| Crate | Role |
|-------|------|
| `tuitbot-core` | All business logic: three layers above, plus `x_api`, `storage`, `llm`, `config`, `scoring`, `safety`, `content`, `strategy`, `source`, `context` |
| `tuitbot-cli` | CLI binary: parsing, logging, dispatch |
| `tuitbot-mcp` | MCP server: AI agent integration, 140 tools across 4 profiles |
| `tuitbot-server` | Axum HTTP/WS API: thin layer over core |

### Key Modules

| Module | Notes |
|--------|-------|
| `core/source/` | `ContentSourceProvider` trait; `LocalFsProvider`, `GoogleDriveProvider` implementations |
| `core/automation/watchtower/` | File watcher, remote polling, shared `ingest_content()` pipeline, loop-back metadata |
| `core/automation/seed_worker.rs` | Background LLM worker extracting draft seeds from content nodes |
| `core/context/winning_dna.rs` | Archetype classification, engagement scoring, ancestor retrieval, cold-start seeds |
| `core/storage/watchtower/` | CRUD for `source_contexts`, `content_nodes`, `draft_seeds` tables |

## Deployment Modes

Tuitbot supports three deployment modes controlling which features and source types are available:

| Mode | Context | Capabilities |
|------|---------|-------------|
| **Desktop** (default) | Native Tauri app | Full local filesystem + native file picker + remote sources |
| **SelfHost** | Docker/VPS browser UI | Local filesystem (server-side) + remote sources |
| **Cloud** | Managed cloud service | Remote sources only (no local filesystem access) |

Deployment mode is set via `TUITBOT_DEPLOYMENT_MODE` env var or `deployment_mode` in config. Default is `Desktop`. Capabilities are derived from mode — they are not individually configurable. See `docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md` for the full matrix.

`DeploymentMode` is orthogonal to `OperatingMode` (Autopilot/Composer). A cloud user can run in Composer mode; a desktop user can run in Autopilot mode.

## Frontend Stack & Modes

- **Dashboard UI**: SvelteKit single-page application built out of `dashboard/`. Connects to `tuitbot-server`.
- **Tauri Integration**: Wraps the Dashboard and `tuitbot-server` into a single standalone native Desktop App package.
- **Docker/Cloud**: The Dashboard can be served statically by the Axum backend via the `TUITBOT_DASHBOARD_DIR` flag for self-hosted environments. The backend also supports `tuitbot-server --mode cloud` for a Stripe-gated multi-tenant mode.

## Storage

- SQLite via SQLx, WAL mode, pool of 4
- Migrations embedded from crate-local migrations directory
- Single-process lock prevents overlapping run/tick instances
- 90-day retention, dedup records never deleted

## Content Source Pipeline (Watchtower)

The Watchtower subsystem ingests content from external sources, extracts draft seeds, and enriches the content generation pipeline via Winning DNA retrieval.

### Provider Model

Content sources implement the `ContentSourceProvider` trait (`core::source/`). Source availability is gated by the deployment mode capability matrix — `local_fs` requires `local_folder` capability (Desktop/SelfHost only).

| Provider | Module | Mechanism | Status | Modes |
|----------|--------|-----------|--------|-------|
| `local_fs` | `source/local_fs.rs` | `notify` watcher + fallback polling | Stable | Desktop, SelfHost |
| `google_drive` | `source/google_drive.rs` | Interval polling via Drive API v3 | Stable (read-only) | All |
| `manual` | (inline via API) | Direct `POST /api/ingest` | Stable | All |

### Pipeline Flow

```
Source → scan/watch → ingest_content() → content_nodes → SeedWorker → draft_seeds → Winning DNA → draft pipeline
```

### Storage Tables

| Table | Purpose |
|-------|---------|
| `source_contexts` | Registered sources with sync state |
| `content_nodes` | Ingested content with dedup by (source_id, relative_path, hash) |
| `draft_seeds` | Pre-computed hooks/angles for cold-start context |

## Runtime Loops

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
| Watchtower (content sources) | Active | Active |
| Seed worker | Active | Active |

Strategy reports run weekly (and on-demand via API) in both modes.

## AI Assist Endpoints

Stateless generation endpoints under `/api/assist/` provide on-demand content creation. Each endpoint accepts a topic or draft text and returns generated content using the configured LLM and persona. Available in both modes but primarily designed for Composer workflows.

## Draft Lifecycle

Drafts follow a linear lifecycle: `draft` (created or generated) -> `scheduled` (assigned a publish time) -> `posted` (routed through the approval queue and posting pipeline). Drafts are mode-independent and persist across mode switches.

## Discovery Feed

The Discovery Feed exposes scored tweets from the read-only discovery loop. Users browse conversations, compose replies (optionally with AI Assist), and queue them for posting through the approval queue. Available via `/api/discovery/feed` endpoints and the dashboard Discovery page.

## Design Principles

- Utility-first: every X API operation is a standalone, composable function
- Layered architecture with strict dependency rules
- Dual operating modes: fully autonomous (Autopilot) and user-driven (Composer)
- Conservative automation defaults
- Explicit approval and guardrails
- Deterministic CLI interfaces for scheduler and agent integration
