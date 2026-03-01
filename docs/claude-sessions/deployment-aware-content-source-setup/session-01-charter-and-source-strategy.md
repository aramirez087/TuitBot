# Session 01: Charter And Source Strategy

Paste this into a new Claude Code session:

```md
Continuity
- Start from the current repository files only; do not assume prior artifacts.

Mission
Audit the current deployment-mode and content-source flow, then lock the product and technical contract before any code changes.

Repository anchors
- `docs/architecture.md`
- `docs/configuration.md`
- `docs/lan-mode.md`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/source/google_drive.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `dashboard/src/lib/stores/runtime.ts`
- `dashboard/src/lib/stores/onboarding.ts`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`

Tasks
1. Document how desktop, self-host, and cloud modes currently expose `local_fs` and `google_drive` content sources.
2. Define the target UX by mode, including the exact rule that only desktop should default to direct vault-path setup.
3. Decide the connector model for self-host and LAN deployments, including how Google Drive account linking should work and how future remote connectors can fit.
4. Lock the migration boundaries for existing `local_fs` and service-account-based Google Drive users.
5. Break the implementation into explicit backend, provider, frontend, and migration slices with named files.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/charter.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-01-handoff.md`

Quality gates
- No code changes are expected; keep this session documentation-only.

Exit criteria
- The charter resolves mode-specific defaults, connector strategy, migration scope, security rules, and exact inputs for Session 02.
```
