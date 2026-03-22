# Session 10: Instrumentation And Success Metrics

Paste this into a new Claude Code session:

```md
Continue from Session 09 artifacts.

Mission
Instrument Hook Miner and Forge so adoption, fallback usage, and sync reliability are measurable after release.

Repository anchors
- dashboard/src/lib/analytics/backlinkFunnel.ts
- dashboard/src/lib/analytics/funnel.ts
- crates/tuitbot-server/src/routes/telemetry.rs
- docs/roadmap/hook-miner-forge-loop/hook-miner-ux.md
- docs/roadmap/hook-miner-forge-loop/settings-and-copy-notes.md

Tasks
1. Add typed frontend events for `hook_miner.angles_shown`, `hook_miner.angle_selected`, `hook_miner.fallback_opened`, `forge.prompt_shown`, `forge.enabled`, `forge.sync_succeeded`, and `forge.sync_failed`.
2. Extend the telemetry intake path so these events are accepted without regressing the current `backlink.*` event namespace.
3. Capture the minimum useful properties for each event: source path, session id when available, local-vs-nonlocal eligibility, angle kind, and success or failure reason.
4. Keep telemetry best-effort and privacy-safe; do not leak raw note bodies or frontmatter.
5. Document how to read these events to judge adoption, weak-signal fallback frequency, and sync reliability.
6. Add focused tests for the telemetry allowlist and helper functions.

Deliverables
- docs/roadmap/hook-miner-forge-loop/instrumentation-plan.md
- docs/roadmap/hook-miner-forge-loop/session-10-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Hook Miner and Forge events are explicit, typed, and accepted by the backend.
- Telemetry remains privacy-safe and backwards compatible.
- The plan makes post-release evaluation concrete.
```
