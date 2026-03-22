# Session 07: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission
Validate the Backlink Synthesizer end to end and produce a go or no-go release assessment.

Repository anchors
- docs/roadmap/backlink-synthesizer/
- crates/tuitbot-server/tests/
- dashboard/tests/unit/
- dashboard/tests/e2e/
- crates/tuitbot-core/src/context/
- crates/tuitbot-core/src/storage/watchtower/

Tasks
1. Run and fix the relevant backend and frontend test coverage for graph ingestion, retrieval, API behavior, and Ghostwriter UX.
2. Add missing regression tests for sparse graph fallback, unresolved links, tag-only neighbors, and provenance preservation.
3. Perform a consistency pass over docs, route contracts, and UI copy so the feature tells one coherent story.
4. Write the QA matrix, residual risks, rollout guidance, and final go or no-go call.

Deliverables
- docs/roadmap/backlink-synthesizer/qa-matrix.md
- docs/roadmap/backlink-synthesizer/release-readiness.md
- docs/roadmap/backlink-synthesizer/session-07-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run
- npm --prefix dashboard run test:e2e

Exit criteria
- The epic has explicit pass or fail evidence for ingestion, retrieval, UX, and provenance.
- Residual risks and rollout notes are concrete.
- The handoff gives a clear next action if the result is no-go.
```
