# Session 08: Final Validation And Go No Go

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.
Mission: Validate that the shipped composer is objectively better than Typefully and produce a go/no-go report.

Repository anchors:
- `docs/roadmap/typefully-composer-ui-parity/charter.md`
- `docs/roadmap/typefully-composer-ui-parity/session-07-handoff.md`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `crates/tuitbot-server/tests/api_tests.rs`

Tasks:
1. Validate all core requirements: visual composer cards, reorder, media placement, and distraction-free writing flow.
2. Validate superiority criteria from the scorecard (speed, control, feedback, accessibility) with evidence.
3. Run full quality gates and summarize results with exact command outputs.
4. Execute smoke scenarios: new thread compose, reorder + media reassignment, draft edit roundtrip, schedule/publish path.
5. Confirm Ghostwriter engine remains untouched in this initiative.
6. Produce final go/no-go report with residual risks, rollback notes, and follow-up backlog.

Deliverables:
- `docs/roadmap/typefully-composer-ui-parity/traceability-matrix.md`
- `docs/roadmap/typefully-composer-ui-parity/superiority-scorecard-final.md`
- `docs/roadmap/typefully-composer-ui-parity/final-go-no-go-report.md`
- `docs/roadmap/typefully-composer-ui-parity/session-08-handoff.md`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria:
- All UI parity requirements are marked pass/fail with file-path evidence.
- Superiority scorecard shows objective wins versus baseline.
- Final report contains explicit go/no-go verdict.
- Handoff includes post-release monitoring and rollback notes.
```
