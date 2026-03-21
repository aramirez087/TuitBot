# Session 07: Native Workflow Polish

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/hook-first-workflow.md` and `docs/roadmap/obsidian-ghostwriter-edge/session-06-handoff.md`.

Mission
Propagate Ghostwriter provenance and chosen-hook context through drafts, scheduling, and adjacent native surfaces so the feature feels built-in rather than bolted on.

Repository anchors
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-core/src/storage/scheduled_content/tests/provenance.rs`
- `dashboard/src/lib/components/drafts/DraftMetadataSection.svelte`
- `dashboard/src/lib/components/drafts/DraftHistoryPanel.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/composer/CitationChips.svelte`
- `dashboard/src/lib/api/types.ts`

Tasks
1. Ensure Ghostwriter provenance and chosen-hook context survive draft creation, scheduling, publish handoff, and later editing flows.
2. Expose the right citation and source-context affordances in draft-studio and related product surfaces without overloading the UI.
3. Add regression coverage for provenance persistence in scheduled content and related route contracts.
4. Document the native workflow behavior that Session 08 must preserve while adding privacy gating.

Deliverables
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-core/src/storage/scheduled_content/tests/provenance.rs`
- `dashboard/src/lib/components/drafts/DraftMetadataSection.svelte`
- `dashboard/src/lib/components/drafts/DraftHistoryPanel.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/composer/CitationChips.svelte`
- `dashboard/src/lib/api/types.ts`
- `docs/roadmap/obsidian-ghostwriter-edge/native-workflow-polish.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-07-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run test:unit:run`

Exit criteria
- Ghostwriter state survives the native draft and schedule lifecycle.
- Session 08 can add privacy gating without losing product continuity.
```
