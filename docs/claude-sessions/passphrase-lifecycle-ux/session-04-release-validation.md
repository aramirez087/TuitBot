# Session 04: Release Validation

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Validate the passphrase lifecycle fixes end to end and produce a release-readiness decision for the onboarding and reset changes.

Repository anchors
- docs/roadmap/passphrase-lifecycle-ux/charter.md
- docs/roadmap/passphrase-lifecycle-ux/session-01-handoff.md
- docs/roadmap/passphrase-lifecycle-ux/session-02-handoff.md
- docs/roadmap/passphrase-lifecycle-ux/session-03-handoff.md
- crates/tuitbot-server/src/main.rs
- crates/tuitbot-server/src/auth/routes.rs
- dashboard/src/routes/+layout.svelte
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/login/+page.svelte
- docs/lan-mode.md

Tasks
1. Review Sessions 01-03 outputs and verify that each user-reported issue maps to a concrete code change and a passing check.
2. Run the full quality gates and targeted manual checks for web onboarding, CLI reset output, and login after an out-of-band passphrase reset, then capture the exact results.
3. Write release-readiness with scenario-by-scenario pass or fail status, residual risks, and a go or no-go recommendation.
4. Create the final handoff with any follow-up work that is still required.

Deliverables
- docs/roadmap/passphrase-lifecycle-ux/release-readiness.md
- docs/roadmap/passphrase-lifecycle-ux/session-04-handoff.md

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The report explicitly resolves all three reported issues or names the blocking failure with exact follow-up file paths.
- Any remaining risk is concrete and scoped.
- The go or no-go recommendation is unambiguous.
```
