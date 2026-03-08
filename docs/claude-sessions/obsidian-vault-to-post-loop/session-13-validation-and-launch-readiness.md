# Session 13: Validation And Launch Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 12 artifacts.

Continuity
- Validate the entire initiative end to end and treat residual-risk reporting as part of the work, not an optional appendix.

Mission
- Validate the full Obsidian vault initiative end to end and produce a release recommendation with explicit residual risks.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-server/src/routes/assist.rs`

Tasks
1. Run the full quality gates and fix any regressions that are in scope for this epic.
2. Execute manual coverage for source setup, sync status, From Vault, reply assist, provenance, loop-back, posting, and desktop note-open behavior.
3. Write a QA matrix and release-readiness report with a go or no-go call, known issues, and rollback or mitigation notes.
4. Ensure all roadmap docs and handoffs are internally consistent and reference final file paths only.

Deliverables
- `docs/roadmap/obsidian-vault-to-post-loop/qa-matrix.md`
- `docs/roadmap/obsidian-vault-to-post-loop/release-readiness.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-13-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- All gates pass or every failure is explicitly triaged, QA covers the new workflow thoroughly, and the report makes a clear ship or no-ship recommendation.
```
