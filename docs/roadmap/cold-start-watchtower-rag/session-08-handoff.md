# Session 08 — Deployment Mode Source Capabilities: Handoff

## Summary

Introduced a `DeploymentMode` enum (Desktop/SelfHost/Cloud) and `DeploymentCapabilities` struct that makes local folder access a deployment-environment capability rather than a universal assumption. The backend now exposes deployment mode and capabilities via `GET /api/runtime/status`, and validation rejects `local_fs` sources in cloud mode. Frontend types are updated to consume the capability payload. All existing desktop/self-host configs remain backward compatible — `DeploymentMode` defaults to `Desktop`.

## What Was Delivered

### Core types (`crates/tuitbot-core/src/config/types.rs`)
- `DeploymentMode` enum: `Desktop` (default), `SelfHost`, `Cloud` — with serde, Display, PartialEq
- `DeploymentCapabilities` struct: 5 boolean capability flags
- `DeploymentMode::capabilities()` — pure function deriving capabilities from mode
- `DeploymentMode::allows_source_type()` — convenience check for source type strings

### Config integration (`crates/tuitbot-core/src/config/mod.rs`)
- `deployment_mode: DeploymentMode` field on `Config` with `#[serde(default)]`
- Re-exports `DeploymentMode` and `DeploymentCapabilities`

### Env var support (`crates/tuitbot-core/src/config/env_overrides.rs`)
- `TUITBOT_DEPLOYMENT_MODE` override accepting `desktop`, `self_host`/`selfhost`/`self-host`, `cloud`

### Validation (`crates/tuitbot-core/src/config/validation.rs`)
- Source capability check: rejects `local_fs` sources when `deployment_mode == Cloud`
- Error message includes both the source type and the deployment mode for clarity

### Server changes
- `AppState.deployment_mode` field (`crates/tuitbot-server/src/state.rs`)
- `GET /api/runtime/status` now returns `deployment_mode` and `capabilities` fields
- `main.rs` passes `deployment_mode` from config to AppState
- All test AppState construction sites updated

### Frontend types (`dashboard/src/lib/api.ts`)
- `DeploymentModeValue` type alias
- `DeploymentCapabilities` interface
- `RuntimeStatus` interface
- `api.runtime` namespace with `status()`, `start()`, `stop()` methods
- `deployment_mode` field added to `TuitbotConfig` interface

### Documentation
- `docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md` — authoritative capability reference
- `docs/architecture.md` — new "Deployment Modes" section, updated provider model table with mode availability
- `docs/configuration.md` — `deployment_mode` in config sections table, new "Deployment Mode" section, env var documentation, content sources per-mode note

### Tests (20 new tests in `crates/tuitbot-core/src/config/tests.rs`)
- `DeploymentMode::Desktop/SelfHost/Cloud` allows/rejects correct source types
- Unknown source types always rejected
- Capability struct correctness for all three modes
- Serde roundtrip (TOML + JSON)
- Default is `Desktop`
- Missing `deployment_mode` in config defaults to `Desktop`
- Env var override (cloud, self_host variants, invalid)
- Validation rejects `local_fs` in cloud, allows `google_drive` in cloud, allows `local_fs` in desktop
- `DeploymentCapabilities` JSON serde roundtrip
- `Display` implementation

## CI Results

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | 1014+ tests pass |
| `cargo clippy --workspace -- -D warnings` | 0 warnings |
| `npm run check` (dashboard) | 0 errors |

## Design Decisions

1. **Capabilities derived from mode, not configurable individually** — prevents impossible states. Operators pick a mode; capabilities follow automatically.
2. **Desktop as default** — zero-config backward compatibility for all existing users.
3. **Runtime status endpoint extended** (not new endpoint) — avoids API surface bloat. The dashboard already fetches runtime status on load.
4. **Validation rejects on save, not on load** — a cloud server with pre-existing `local_fs` entries starts without crashing. Validation errors only surface when trying to save via PATCH.
5. **`DeploymentMode` orthogonal to `OperatingMode`** — they're independent axes. This avoids coupling deployment concerns with autonomy levels.

## Open Items for Next Sessions

### Session 09 — Capability-Based Source UX
- Wire `api.runtime.status()` capabilities into the Settings > Content Sources section
- Conditionally render source type options based on `capabilities.local_folder`, etc.
- Show explanatory messages when capabilities are unavailable
- Disable the native file picker button when `capabilities.file_picker_native` is false

### Session 10 — Mode-Aware Validation UX
- Surface deployment-mode validation errors in the settings UI (not just API errors)
- Consider a "source migration assistant" that helps cloud users convert `local_fs` → `google_drive`
- Add Watchtower startup guard: skip local_fs sources in cloud mode with structured log warning

### Future
- `TUITBOT_DEPLOYMENT_MODE` in Docker compose templates
- Tauri sidecar could explicitly set `Desktop` (currently unnecessary since it's the default)
- Cloud billing integration may want to read `DeploymentMode` to gate premium source types

## Files Changed

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/config/types.rs` | Added `DeploymentMode`, `DeploymentCapabilities` |
| `crates/tuitbot-core/src/config/mod.rs` | Added `deployment_mode` field, re-exports |
| `crates/tuitbot-core/src/config/env_overrides.rs` | Added `TUITBOT_DEPLOYMENT_MODE` |
| `crates/tuitbot-core/src/config/validation.rs` | Added source capability validation |
| `crates/tuitbot-core/src/config/tests.rs` | 20 new tests |
| `crates/tuitbot-server/src/state.rs` | Added `deployment_mode` field |
| `crates/tuitbot-server/src/main.rs` | Pass deployment_mode to AppState |
| `crates/tuitbot-server/src/routes/runtime.rs` | Extended status response |
| `crates/tuitbot-server/tests/api_tests.rs` | Updated AppState construction (12 sites) |
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Updated AppState construction |
| `dashboard/src/lib/api.ts` | Added deployment types, runtime namespace |
| `docs/architecture.md` | New deployment modes section |
| `docs/configuration.md` | Deployment mode docs |
| `docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md` | New |
| `docs/roadmap/cold-start-watchtower-rag/session-08-handoff.md` | This file |
