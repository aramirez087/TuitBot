# Session 09: End To End Validation And Launch Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Mission
Validate the onboarding-to-first-value flow end to end and produce a launch-readiness verdict with no ambiguous follow-up state.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/session-08-handoff.md
- docs/roadmap/x-profile-prefill-onboarding/funnel-metrics.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- crates/tuitbot-server/src/routes/onboarding.rs
- crates/tuitbot-server/src/routes/settings.rs

Tasks
1. Run the full validation matrix across X-first onboarding, progressive activation, starter-state provisioning, and first-value landing behavior.
2. Verify that desktop and self-host flows stay aligned where intended and diverge only where the charter explicitly allows it.
3. Close any last inconsistencies in contracts, docs, or route behavior that block a launch-quality experience.
4. Produce a go or no-go launch report with concrete residual risks, mitigation status, and recommended next steps.
5. Finish with a complete handoff that names exactly what remains after the epic is done.

Deliverables
- docs/roadmap/x-profile-prefill-onboarding/qa-matrix.md
- docs/roadmap/x-profile-prefill-onboarding/launch-readiness.md
- docs/roadmap/x-profile-prefill-onboarding/session-09-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- The launch-readiness report gives a clear verdict supported by evidence from checks and scenario coverage.
- All prior session decisions are reflected in the final docs and no critical path remains undocumented.
- The epic can be handed to execution without reopening the core product design question.
```
