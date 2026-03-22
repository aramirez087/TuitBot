# Session 11: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 10 artifacts.

Mission
Validate Hook Miner + Forge end to end and produce a release-readiness verdict with concrete residual risks.

Repository anchors
- docs/roadmap/hook-miner-forge-loop/
- crates/tuitbot-server/tests/
- dashboard/tests/unit/
- dashboard/tests/e2e/
- crates/tuitbot-core/src/automation/
- crates/tuitbot-core/src/storage/

Tasks
1. Run and fix the relevant backend and frontend tests for Hook Miner extraction, composer UX, provenance, thread normalization, Forge sync, settings UX, and telemetry.
2. Add missing regression coverage for weak-signal fallback, thread aggregation, missing-note sync skips, and non-local source guards.
3. Perform a consistency pass over docs, route contracts, settings copy, and telemetry naming.
4. Write the QA matrix, release-readiness assessment, rollout notes, and residual risks with explicit go or no-go reasoning.

Deliverables
- docs/roadmap/hook-miner-forge-loop/qa-matrix.md
- docs/roadmap/hook-miner-forge-loop/release-readiness.md
- docs/roadmap/hook-miner-forge-loop/session-11-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run
- npm --prefix dashboard run test:e2e

Exit criteria
- The epic has explicit pass or fail evidence for Hook Miner, Forge, thread normalization, and privacy boundaries.
- Residual risks are concrete and ranked.
- The final handoff tells the next operator exactly what to do if the result is no-go.
```
