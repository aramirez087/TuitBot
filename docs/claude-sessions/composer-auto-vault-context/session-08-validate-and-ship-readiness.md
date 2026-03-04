# Session 08: Validate and Ship Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/release-notes.md
- Read docs/roadmap/composer-auto-vault-context/session-07-handoff.md

Mission
Run the final validation pass and issue a go or no-go decision for composer automatic vault context.

Repository anchors
- crates/tuitbot-core/src/content/generator/mod.rs
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-server/tests/api_tests.rs
- docs/composer-mode.md

Tasks
1. Re-read the implemented backend changes, regression tests, and updated docs for consistency.
2. Run the full required quality gates and capture the exact outcome, including any failures or flakes.
3. Verify the shipped behavior against the feature intent: all composer-generated content paths should automatically use vault context with graceful fallback.
4. Write a release-readiness report with go/no-go status, evidence, residual risks, and rollback notes if needed.
5. Write the final handoff summarizing the entire initiative and any post-merge follow-up work.

Deliverables
- docs/roadmap/composer-auto-vault-context/release-readiness.md
- docs/roadmap/composer-auto-vault-context/qa-matrix.md
- docs/roadmap/composer-auto-vault-context/session-08-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- A clear go/no-go recommendation exists, backed by build output, test results, and explicit remaining risks.
```
