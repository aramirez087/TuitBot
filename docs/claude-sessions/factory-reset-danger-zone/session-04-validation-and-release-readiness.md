# Session 04: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.
Continuity
- Read the charter, reset contract, frontend flow, and `docs/roadmap/factory-reset-danger-zone/session-03-handoff.md` first.

Mission
Validate the factory-reset epic end-to-end, reconcile the artifacts with the code, and publish a go or no-go release report.

Repository anchors
- `docs/roadmap/factory-reset-danger-zone/charter.md`
- `docs/roadmap/factory-reset-danger-zone/reset-contract.md`
- `docs/roadmap/factory-reset-danger-zone/frontend-flow.md`
- `crates/tuitbot-server/tests/factory_reset.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `dashboard/src/routes/(app)/settings/DangerZoneSection.svelte`
- `dashboard/src/routes/+layout.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`

Tasks
1. Run the backend and dashboard quality gates and fix any regressions that block the chartered flow.
2. Verify the critical path: configured instance -> danger zone -> confirmed reset -> onboarding -> fresh init path.
3. Confirm the reset leaves no accessible user data behind in config-backed flows and that protected endpoints still require auth before reset.
4. Update the roadmap docs if code diverged from the earlier artifacts and record residual risks clearly.
5. Produce a release-readiness report with a go or no-go call and exact follow-up items if anything remains.

Deliverables
- `docs/roadmap/factory-reset-danger-zone/release-readiness.md`
- `docs/roadmap/factory-reset-danger-zone/session-04-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cd dashboard && npm run check && npm run build`

Exit criteria
- All required checks pass or the blockers are documented precisely, the artifacts match the shipped behavior, and the final handoff states the release decision.
```
