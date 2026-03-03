# Session 02: Settings X Access Mode

Paste this into a new Claude Code session:

```md
Continuity
Continue from Session 01 artifacts.
Read `docs/roadmap/no-x-api-local-mode/charter.md` and `docs/roadmap/no-x-api-local-mode/x-access-contract.md` before changing code.

Mission
Expose the existing backend selector as a first-class settings and onboarding UX and make config validation accept scraper mode without X developer credentials.

Repository anchors
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/defaults.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-core/src/config/tests.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `dashboard/src/lib/api.ts`
- `dashboard/src/routes/(app)/settings/XApiSection.svelte`
- `dashboard/src/lib/components/onboarding/XApiStep.svelte`

Tasks
1. Keep `x_api.provider_backend` and `x_api.scraper_allow_mutations` as the shared config fields and ensure they serialize through the settings API and dashboard types.
2. Update validation so `provider_backend = "scraper"` allows empty `client_id` and `client_secret`, while `provider_backend = "x_api"` preserves the current credential rules.
3. Reject or auto-correct scraper mode when `deployment_mode = "cloud"` and cover the behavior with tests.
4. Replace the current X API-only copy with a mode-aware "X Access" UX that lets users choose `Official X API` or `Local No-Key Mode`, conditionally shows credential fields, and explains the risk and safety tradeoffs.
5. Surface `scraper_allow_mutations` as an explicit advanced toggle with warning copy and default it to off in the UI.
6. Add or update tests covering config round-trips, validation rules, and settings payload shape.
7. Write `docs/roadmap/no-x-api-local-mode/settings-flow.md` with the final UI contract and end with a handoff.

Deliverables
- `docs/roadmap/no-x-api-local-mode/settings-flow.md`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/defaults.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `dashboard/src/lib/api.ts`
- `dashboard/src/routes/(app)/settings/XApiSection.svelte`
- `dashboard/src/lib/components/onboarding/XApiStep.svelte`
- `docs/roadmap/no-x-api-local-mode/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Local and LAN users can select scraper mode in settings without entering X API credentials.
- Cloud mode does not expose or accept scraper mode.
- The handoff identifies the exact runtime gaps remaining for Session 03.
```
