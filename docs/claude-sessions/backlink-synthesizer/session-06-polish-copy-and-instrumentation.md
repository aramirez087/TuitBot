# Session 06: Polish Copy And Instrumentation

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Polish the Backlink Synthesizer so it feels intuitive in daily use and emits the product signals needed to judge whether the flow is actually working.

Repository anchors
- dashboard/src/lib/components/composer/
- dashboard/src/lib/analytics/
- dashboard/tests/unit/
- crates/tuitbot-server/src/routes/vault/
- crates/tuitbot-server/src/ws.rs
- docs/roadmap/backlink-synthesizer/epic-charter.md
- docs/roadmap/backlink-synthesizer/end-to-end-ux-journey.md
- docs/roadmap/backlink-synthesizer/draft-insertion-model.md

Tasks
1. Refine copy, labels, helper text, and action names so the feature explains itself with minimal cognitive load.
2. Improve small UX details: card ordering, motion, dismiss behavior, empty-state recovery, and transition clarity between selection, suggestions, and editing.
3. Add analytics or event instrumentation for key moments such as suggestions shown, suggestion accepted, suggestion dismissed, slot targeted, undo used, and draft completed.
4. Add any lightweight telemetry-safe backend support required for those events.
5. Document the UX copy system and the success-measurement plan.

Deliverables
- docs/roadmap/backlink-synthesizer/ux-copy-and-state-notes.md
- docs/roadmap/backlink-synthesizer/instrumentation-plan.md
- docs/roadmap/backlink-synthesizer/session-06-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- The flow feels legible and low-friction for both first-time and repeat users.
- Instrumentation can answer whether the feature improves usage and draft quality.
- Copy and state transitions are coherent across the whole flow.
```
