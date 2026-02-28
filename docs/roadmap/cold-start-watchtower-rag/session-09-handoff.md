# Session 09 — Capability-Based Source UX: Handoff

## Summary

Wired the `DeploymentCapabilities` payload into both the Settings and Onboarding source-configuration UIs. The frontend now fetches deployment capabilities via a shared store and conditionally renders source type options, the native folder picker, and manual path entry based on the server's declared deployment mode. Cloud users see only connector options with explanatory copy; self-host users get manual path entry without the native picker; desktop users retain the full experience including native folder selection.

## What Was Delivered

### Server changes (`crates/tuitbot-server/src/routes/settings.rs`)
- Extended `GET /api/settings/status` (unauthenticated) to include `deployment_mode` and `capabilities` fields alongside the existing `configured` boolean
- This allows onboarding pages (which lack auth tokens) to discover capabilities without requiring authentication

### Frontend types (`dashboard/src/lib/api.ts`)
- Added `ConfigStatus` interface with `configured`, `deployment_mode`, and `capabilities` fields
- Updated `api.settings.configStatus()` return type from `{ configured: boolean }` to `ConfigStatus`

### Runtime capabilities store (`dashboard/src/lib/stores/runtime.ts`) — NEW
- New Svelte store providing `capabilities`, `deploymentMode`, and `capabilitiesLoaded` derived stores
- `loadCapabilities()` uses dual-fetch strategy: tries authenticated `api.runtime.status()` first, falls back to unauthenticated `api.settings.configStatus()`, then defaults to desktop capabilities
- Singleton fetch guard — idempotent re-calls are no-ops
- Desktop defaults used as fallback when server is unreachable (safe for all deployment modes)

### Settings page (`dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`)
- Replaced `isTauri` platform detection with capability-derived flags: `canLocalFs`, `canManualPath`, `canNativePicker`, `canGoogleDrive`
- Source type `<select>` now conditionally renders options based on capabilities
- Browse button gated on `canNativePicker` instead of `isTauri`
- Hint text adapts: "Click Browse..." for desktop, "Enter the full server-side path..." for self-host
- Added `.capability-notice` banner when `canLocalFs` is false explaining cloud limitations
- `$effect` auto-switches source type to `google_drive` when capabilities disallow `local_fs`

### Onboarding page (`dashboard/src/lib/components/onboarding/SourcesStep.svelte`)
- Same capability-gated approach as settings: `canLocalFs`, `canNativePicker`, `canGoogleDrive`
- Source type dropdown filtered by capabilities
- Browse button gated on `canNativePicker`
- Added `.capability-hint` message for cloud deployments explaining why local folders are unavailable
- Added self-host hint text for manual path entry
- `$effect` auto-switches `sourceType` to `google_drive` in cloud mode

### Tests (2 new in `crates/tuitbot-server/tests/api_tests.rs`)
- `config_status_includes_capabilities`: Verifies `GET /api/settings/status` returns `configured`, `deployment_mode`, and `capabilities` fields without authentication; confirms desktop defaults
- `config_status_capabilities_match_cloud_mode`: Creates AppState with `DeploymentMode::Cloud`, verifies `local_folder`, `manual_local_path`, and `file_picker_native` are all `false` while `google_drive` is `true`

## CI Results

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | All tests pass (including 2 new) |
| `cargo clippy --workspace -- -D warnings` | 0 warnings |
| `npm run check` (dashboard) | 0 errors |

## Design Decisions

1. **Dual-fetch strategy for capabilities** — The runtime status endpoint (`GET /api/runtime/status`) requires auth, but onboarding pages don't have a token. Solution: added capabilities to the unauthenticated `GET /api/settings/status` endpoint; the store tries both, falling back gracefully.

2. **Remove options instead of disabling them** — When `canLocalFs` is false, "Local Folder" is removed from the dropdown entirely rather than shown as disabled. This prevents confusion and dead-end clicks.

3. **$effect for auto-switching** — When capabilities load and current source type is unsupported, an `$effect` automatically switches to `google_drive`. The guard `sourceType === 'local_fs'` prevents infinite loops.

4. **Capability defaults err toward permissiveness** — If the API call fails (server not running), defaults assume desktop mode with `file_picker_native: false`. This is safe: desktop users still see manual path entry, and the Browse button was already gated on the Tauri dynamic import succeeding.

5. **No backend changes beyond `config_status`** — The runtime endpoint, Tauri lib.rs, and Cargo.toml were already correct from Session 08. Only the unauthenticated settings/status endpoint needed the new fields.

## Exit Criteria Verification

| Criterion | Status |
|-----------|--------|
| Desktop users get native folder-picker flow | Met — `canNativePicker` derives from `capabilities.file_picker_native`; desktop mode sets this to `true` |
| Self-host users can configure local path without native picker | Met — `canManualPath` is `true`, `canNativePicker` is `false` in self-host mode |
| Cloud users are not shown local filesystem affordances | Met — `canLocalFs` is `false` in cloud mode; dropdown hides "Local Folder" and shows explanation |

## Open Items for Next Sessions

### Session 10 — Mode-Aware Validation UX
- Surface deployment-mode validation errors in the settings UI (not just API errors)
- Consider a "source migration assistant" that helps cloud users convert `local_fs` to `google_drive`
- Add Watchtower startup guard: skip local_fs sources in cloud mode with structured log warning

### Future
- `TUITBOT_DEPLOYMENT_MODE` in Docker compose templates
- Tauri sidecar could explicitly set `Desktop` (currently unnecessary since it's the default)
- Cloud billing integration may want to read `DeploymentMode` to gate premium source types

## Files Changed

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/routes/settings.rs` | Extended `config_status` with `deployment_mode` and `capabilities` |
| `crates/tuitbot-server/tests/api_tests.rs` | 2 new tests for config_status capabilities |
| `dashboard/src/lib/api.ts` | Added `ConfigStatus` interface, updated `configStatus()` return type |
| `dashboard/src/lib/stores/runtime.ts` | **New** — capabilities store with dual-fetch strategy |
| `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` | Capability-gated source options, Browse button, notice banner |
| `dashboard/src/lib/components/onboarding/SourcesStep.svelte` | Capability-gated onboarding source step, auto-switch, hint text |
| `docs/roadmap/cold-start-watchtower-rag/session-09-handoff.md` | This file |
