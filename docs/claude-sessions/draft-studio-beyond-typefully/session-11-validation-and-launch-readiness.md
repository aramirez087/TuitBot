# Session 11: Validation And Launch Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 10 artifacts.

Continuity
- Validate the full initiative end to end and treat residual risk reporting as part of the work, not an optional appendix.

Mission
- Validate the full Draft Studio initiative end to end and produce a release recommendation with explicit residual risks.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`

Tasks
1. Run the full quality gates and fix any regressions that are in scope for this epic.
2. Execute manual coverage for tweet, thread, media, autosave failure, recovery, schedule, publish, history restore, keyboard navigation, and mobile behavior.
3. Write a QA matrix and release-readiness report with a go or no-go call, known issues, and rollback or mitigation notes.
4. Ensure all roadmap docs and handoffs are internally consistent and reference final file paths only.

Deliverables
- `docs/roadmap/draft-studio-beyond-typefully/qa-matrix.md`
- `docs/roadmap/draft-studio-beyond-typefully/release-readiness.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-11-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- All gates pass or every failure is explicitly triaged, QA covers the new workflow thoroughly, and the report makes a clear ship or no-ship recommendation.
```
