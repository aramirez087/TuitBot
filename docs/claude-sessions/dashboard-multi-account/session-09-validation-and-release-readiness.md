# Session 09: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Continuity
- Validate against the roadmap docs and shipped code rather than relying on undocumented session memory.

Mission
Validate the end-to-end multi-account implementation, close regressions, and produce the release-readiness package for shipping.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/implementation-plan.md`
- `docs/roadmap/dashboard-multi-account/session-08-handoff.md`
- `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md`
- `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md`
- `docs/roadmap/dashboard-multi-account/runtime-isolation-plan.md`
- `docs/roadmap/dashboard-multi-account/frontend-switching-flow.md`
- `docs/roadmap/dashboard-multi-account/account-management-flow.md`
- `docs/roadmap/dashboard-multi-account/settings-override-ux.md`
- `docs/roadmap/dashboard-multi-account/x-access-account-flow.md`

Tasks
1. Run targeted and full checks, fix regressions that block multi-account readiness, and document any consciously accepted residual risk.
2. Exercise end-to-end scenarios for default-account migration, adding a second account, switching while drafts or settings are open, isolated analytics or activity data, and per-account credential failures.
3. Produce `docs/roadmap/dashboard-multi-account/qa-matrix.md`, `docs/roadmap/dashboard-multi-account/release-readiness.md`, and `docs/roadmap/dashboard-multi-account/session-09-handoff.md` with a go or no-go call and rollback notes.
4. Reconcile any remaining drift between the charter, implementation, and shipped UX before closing the epic.

Deliverables
- `docs/roadmap/dashboard-multi-account/qa-matrix.md`
- `docs/roadmap/dashboard-multi-account/release-readiness.md`
- `docs/roadmap/dashboard-multi-account/session-09-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run build`

Exit criteria
- Full workspace checks pass and release blockers are either fixed or explicitly documented as no-go issues.
- The QA matrix covers the multi-account flows that were previously singletons.
- The release-readiness doc contains a clear ship recommendation and rollback guidance.
```
