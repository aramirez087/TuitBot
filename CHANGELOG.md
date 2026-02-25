# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/aramirez087/TuitBot/compare/tuitbot-cli-v0.1.4...tuitbot-cli-v0.1.5) - 2026-02-25

### Added

- Implement MCP mutation policy enforcement and approval queue for composite tools.
- Introduce new dashboard settings sections and refactor the CLI `init` command for improved modularity.
- Add Composer operating mode, enabling user-driven content creation with AI assist, drafts, and discovery features.
- Implement media upload and display functionality for approval items.
- Implement X API usage tracking, including storage, API endpoints, and dashboard display for costs and call breakdowns.
- implement MCP telemetry and policy management with new API routes, storage functions, and dashboard UI, and update benchmark artifacts.
- Implement MCP tool telemetry, including storage, query tools, and integration into existing tools.
- Implement context intelligence tools for author profiling, engagement recommendations, and topic performance snapshots.
- Implement Multi-Constraint Policy (MCP) evaluation and enforcement for tool mutations.
- Add direct X API tools, client initialization, and comprehensive documentation.
- Implement URL-aware tweet length calculation and validation using the `regex` crate.
- Introduce unified tool response envelope, add a benchmark tool, and establish comprehensive roadmap documentation.

### Other

- Merge branch 'main' into fix/feat_obsidian
- Update `reqwest` to use `rustls-tls` and improve release workflow robustness by handling cancellations and partial build failures.
- Creating the parent directory before both writes.

### Added

- Introduce Composer Mode (`mode = "composer"`) — user-controlled posting with on-demand AI intelligence. Disables autonomous loops; discovery runs read-only; approval mode is always active.
- Add AI Assist endpoints for on-demand content generation: tweet, reply, thread, improve, topics, optimal posting times.
- Add Draft system with full lifecycle: create, edit, schedule, publish, delete.
- Add Discovery Feed for browsing scored tweets with AI-powered reply composition.
- Add 4 new MCP tools for Composer mode: `get_mode`, `compose_tweet`, `get_discovery_feed`, `suggest_topics`.
- Add Drafts and Discovery pages to the dashboard.
- Add AI Assist button in ComposeModal for on-demand content generation and draft improvement.
- Add operating mode selector in the dashboard Settings page.
- Add approval poster loop that automatically posts approved queue items.
- Wire scheduled content to the posting runtime.
- Fix URL-aware tweet length calculation on client side.

#### MCP Superiority Program

- Add v1.0 structured response envelope with `success`, `data`, `error`, and `meta` fields for deterministic agent parsing.
- Add 11 Direct X API tools (5 read, 6 mutation) with full error taxonomy (10 typed error codes with retryable flags).
- Add centralized MCP mutation policy engine with configurable blocking, approval routing, dry-run mode, and hourly rate limits via `[mcp_policy]` config section.
- Add 4 composite goal-oriented tools: `find_reply_opportunities`, `draft_replies_for_candidates`, `propose_and_queue_replies`, `generate_thread_plan`.
- Add 3 context intelligence tools: `get_author_context`, `recommend_engagement_action`, `topic_performance_snapshot`.
- Add MCP telemetry capture (`mcp_telemetry` table) recording tool name, category, latency, success, error code, and policy decision for every invocation.
- Add `get_mcp_tool_metrics` and `get_mcp_error_breakdown` tools for operational observability.
- Add eval harness with 3 scenarios and 2 quality gates (schema validation ≥ 95%, unknown errors ≤ 5%).
- Add `get_policy_status` tool for real-time policy and rate limit introspection.
- Add `get_capabilities` tool with structured capability map including direct tools, policy state, and mode awareness.
- Upgrade OpenClaw plugin to v0.2.0 with 5-layer safety filtering (name allowlist, mutation gate, category allowlist, category denylist, risk ceiling), static tool catalog with 45 tools, and structured error propagation.
- Add MCP governance dashboard page with policy editor, telemetry charts, and activity panel.

## [0.1.4](https://github.com/aramirez087/TuitBot/compare/tuitbot-cli-v0.1.3...tuitbot-cli-v0.1.4) - 2026-02-24

### Added

- Add LLM cost tracking, pricing, and a dashboard for usage reporting.
- Remove `auto_follow` and `follow_warmup_days` settings, streamlining target account management and reinforcing the co-pilot approach.
- remove auto-follow and follow warmup features for target accounts
- Add aarch64-unknown-linux-gnu support to release builds and the update command.
- Add pricing support for Gemini and DeepSeek LLM models.
- Implement a new strategy module for weekly performance reports, recommendations, and a dedicated dashboard page.

### Added

- Add Strategy dashboard page with weekly growth scorecard, recommendation engine, and historical trend charts.
- Add `strategy_reports` table and storage CRUD for persisted weekly reports.
- Add `strategy/metrics.rs` module with date-ranged queries over existing analytics tables.
- Add `strategy/recommendations.rs` with 8-rule deterministic recommendation engine.
- Add 4 new API endpoints: `GET /api/strategy/current`, `GET /api/strategy/history`, `POST /api/strategy/refresh`, `GET /api/strategy/inputs`.

## [0.1.3](https://github.com/aramirez087/TuitBot/compare/tuitbot-cli-v0.1.2...tuitbot-cli-v0.1.3) - 2026-02-24

### Added

- Add target account management with dedicated UI components and API endpoints.
- Implement content scheduling with calendar views and real-time updates
- Implement the approval queue dashboard with full UI, API, and WebSocket integration.

### Other

- Merge branch 'main' into feat/dashboard

## [0.1.2](https://github.com/aramirez087/TuitBot/compare/tuitbot-cli-v0.1.1...tuitbot-cli-v0.1.2) - 2026-02-24

### Added

- Introduce analytics dashboard with components for performance, follower trends, and engagement metrics.
- Implement API token authentication, WebSocket event broadcasting, and new API routes for settings, runtime control, and expanded approval management.
- Introduce `tuitbot-server` crate to provide a read-only HTTP API for storage.

## [0.1.1](https://github.com/aramirez087/TuitBot/compare/tuitbot-cli-v0.1.0...tuitbot-cli-v0.1.1) - 2026-02-23

### Other

- update Cargo.toml dependencies

## [0.1.0](https://github.com/aramirez087/TuitBot/releases/tag/v0.1.0) - 2026-02-22

### Added

- Add `tick` command for single execution of automation loops via external schedulers and refactor runtime dependencies.
- introduce tuitbot-mcp crate implementing a multi-tool control plane server with various management tools and CLI integration.
- Implement slot-based scheduling for content and threads with configurable preferred times and per-day overrides.
- add JSON output for settings and approve commands, and introduce non-interactive approval options.

### Other

- modularize the settings command by splitting its logic into dedicated sub-modules.
- Remove extensive project scaffolding, update core automation loops and CLI commands, and introduce a new scheduling module.
- Renaming to tuitbot

