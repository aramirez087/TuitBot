# Session 04: Validation And Go/No-Go

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Read docs/roadmap/typefully-beating-composer/charter.md.
- Read docs/roadmap/typefully-beating-composer/implementation-plan.md.
- Read docs/roadmap/typefully-beating-composer/session-02-handoff.md.
- Read docs/roadmap/typefully-beating-composer/session-03-handoff.md.

Mission
Validate the completed composer overhaul, fix any ship-blocking regressions, and produce a clear go/no-go decision.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/TweetPreview.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/lib/api.ts
- crates/tuitbot-core/src/content/generator.rs
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs
- docs/composer-mode.md

Tasks
1. Verify the delivered work against the charter and confirm both differentiators are materially stronger than the pre-epic baseline.
2. Run the full required checks, then fix only the issues that block release readiness.
3. Manually audit focus mode, notes-to-thread flow, reusable voice cues, thread restructuring actions, autosave recovery, and preview fidelity on desktop and mobile breakpoints.
4. Update docs so the shipped behavior, constraints, and known limits match the code exactly.
5. Write a release-readiness report with an explicit go or no-go decision, evidence, known limitations, and follow-up items.

Deliverables
- docs/composer-mode.md
- docs/roadmap/typefully-beating-composer/release-readiness.md
- docs/roadmap/typefully-beating-composer/session-04-handoff.md

Quality gates
- Run:
    cargo fmt --all && cargo fmt --all --check
    RUSTFLAGS="-D warnings" cargo test --workspace
    cargo clippy --workspace -- -D warnings
    cd dashboard && npm run check
    cd dashboard && npm run build

Exit criteria
- All required checks pass or the report clearly documents why release is blocked.
- The go/no-go decision is explicit and evidence-based.
- The handoff names any remaining follow-up work with exact file paths.
```
