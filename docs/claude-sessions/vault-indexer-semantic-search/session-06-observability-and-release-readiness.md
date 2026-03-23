# Session 06: Observability And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Validate performance, instrumentation, and release readiness for semantic evidence search across supported deployment modes.

Repository anchors
- CLAUDE.md
- dashboard/src/lib/analytics/backlinkFunnel.ts
- dashboard/src/lib/stores/runtime.ts
- dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/analytics.rs
- crates/tuitbot-server/src/ws.rs
- docs/roadmap/obsidian-ghostwriter-edge/qa-matrix.md
- docs/roadmap/obsidian-ghostwriter-edge/release-readiness.md
- docs/roadmap/vault-indexer-semantic-search/

Tasks
1. Instrument index freshness, search latency, fallback rates, evidence pinning, and whole-draft versus slot-action usage with clear event definitions.
2. Add honest runtime and settings surfaces for semantic index status, privacy envelope, and degraded behavior in Desktop, Self-host, and Cloud.
3. Run end-to-end validation across initial indexing, stale indexes, no results, degraded search, tweet mode, thread mode, undo flows, and provenance integrity.
4. Produce the QA matrix, release checklist, and go or no-go report with explicit blockers if anything remains open.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/metrics-and-rollout.md
- docs/roadmap/vault-indexer-semantic-search/qa-matrix.md
- docs/roadmap/vault-indexer-semantic-search/release-readiness.md
- docs/roadmap/vault-indexer-semantic-search/session-06-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Latency, fallback, and adoption are measurable in production.
- Supported deployment modes communicate index status and privacy honestly.
- The release report ends with an explicit go or no-go decision backed by evidence.
```
