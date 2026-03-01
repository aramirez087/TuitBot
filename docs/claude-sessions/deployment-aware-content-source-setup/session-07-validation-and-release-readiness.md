# Session 07: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.
Continuity
- Read the charter, connection contract, drive flow, Watchtower contract, frontend flow, migration plan, and `docs/roadmap/deployment-aware-content-source-setup/session-06-handoff.md` first.

Mission
Validate the deployment-aware content-source epic end to end and publish a go or no-go release report with any remaining risks called out precisely.

Repository anchors
- `docs/roadmap/deployment-aware-content-source-setup/charter.md`
- `docs/roadmap/deployment-aware-content-source-setup/source-connection-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/drive-connection-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/watchtower-sync-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/frontend-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/migration-plan.md`
- `crates/tuitbot-server/tests/api_tests.rs`
- `crates/tuitbot-core/src/config/tests.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`

Tasks
1. Run the full quality gates and fix any blockers that prevent the chartered behavior from shipping.
2. Verify the critical paths for desktop fresh setup, self-host or LAN fresh setup, cloud fresh setup, and at least one legacy upgrade path.
3. Confirm that connector secrets never surface in logs or UI payloads and that broken links fail safely.
4. Reconcile the roadmap artifacts with the shipped code and record any residual risks or follow-up work.
5. Produce a release-readiness report with a clear go or no-go call.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/release-readiness.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-07-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cd dashboard && npm run check && npm run build`

Exit criteria
- The implementation matches the charter, the validation matrix is complete, and the final handoff states the release decision with exact residual risks.
```
