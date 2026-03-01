# Session 05: Onboarding And Settings UX

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.
Continuity
- Read `docs/roadmap/deployment-aware-content-source-setup/charter.md`, `docs/roadmap/deployment-aware-content-source-setup/watchtower-sync-contract.md`, and `docs/roadmap/deployment-aware-content-source-setup/session-04-handoff.md` first.

Mission
Make onboarding and settings deployment-aware so desktop users see local vault setup first while self-host and cloud users see a linked remote-sync flow first.

Repository anchors
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/runtime.ts`
- `dashboard/src/lib/stores/onboarding.ts`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/routes/+layout.svelte`
- `crates/tuitbot-server/src/routes/settings.rs`

Tasks
1. Keep desktop onboarding browse-first for local vault selection, including explicit Obsidian-friendly copy and no regression to the native picker path.
2. Make self-host and cloud onboarding default to a remote connector flow and remove raw server-side path entry from the primary setup path.
3. Replace service-account key inputs with connect, status, reconnect, disconnect, and folder-selection states that match the new backend contract.
4. Update onboarding submission, settings save flows, and capability-driven copy so the payloads match the new connector model.
5. Add frontend validation coverage and guard the UI against partially linked or revoked connections.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/frontend-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-05-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cd dashboard && npm run check && npm run build`

Exit criteria
- Desktop and non-desktop users now land in the correct source setup flow, and the UI fully reflects connector state without leaking implementation details.
```
