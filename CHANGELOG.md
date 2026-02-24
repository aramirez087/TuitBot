# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

