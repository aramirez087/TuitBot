# Session 05: Hook Miner Provenance And Draft Semantics

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Make Hook Miner attribution survive draft creation, revision, publish, and later Forge sync.

Repository anchors
- crates/tuitbot-core/src/storage/provenance.rs
- crates/tuitbot-server/src/routes/content/compose/mod.rs
- crates/tuitbot-server/src/routes/content/drafts.rs
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/api/types.ts
- docs/roadmap/hook-miner-forge-loop/hook-miner-contract.md
- docs/roadmap/hook-miner-forge-loop/hook-miner-api-contract.md

Tasks
1. Extend provenance with additive optional fields that snapshot Hook Miner attribution: `angle_kind`, `signal_kind`, `signal_text`, and `source_role`.
2. Use exact `source_role` values `primary_selection` and `accepted_neighbor` so later sync logic can distinguish original note context from related-note context.
3. Ensure draft creation and compose flows persist enough provenance to trace a generated draft back to the chosen angle and its supporting signals.
4. Preserve existing node, chunk, seed, edge, and snippet provenance instead of replacing it.
5. Ensure revisions, publish handoff, and later analytics sync can still resolve the originating note set and chosen angle without re-mining the note.
6. Add tests for serialization, backward compatibility, and draft lifecycle propagation.
7. Document the additive provenance contract.

Deliverables
- docs/roadmap/hook-miner-forge-loop/hook-miner-provenance-contract.md
- docs/roadmap/hook-miner-forge-loop/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Drafts can be traced back to the selected mined angle and supporting evidence.
- Existing provenance consumers remain compatible with older rows.
- Forge has enough attribution to update the right note later.
```
