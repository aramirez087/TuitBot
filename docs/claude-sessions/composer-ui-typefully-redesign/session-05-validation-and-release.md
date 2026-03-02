# Session 05: Validation And Release

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
- Validate the redesigned composer end to end and produce a clear release recommendation.

Repository anchors
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowCard.svelte`
- `dashboard/src/lib/components/composer/ComposerPreviewSurface.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`

Tasks
1. Validate the home composer, modal composer, full-screen preview mode, autosave/recovery, media, schedule, publish, and thread-splitting flows.
2. Run the required checks, fix only release-blocking regressions, and document any remaining non-blocking gaps with evidence.
3. Produce a QA matrix that includes desktop and narrow-width coverage, keyboard paths, and state restoration after entering preview.
4. Write a go or no-go recommendation with explicit reasons tied to the charter.

Deliverables
- `docs/roadmap/composer-ui-typefully-redesign/qa-matrix.md`
- `docs/roadmap/composer-ui-typefully-redesign/release-readiness.md`
- `docs/roadmap/composer-ui-typefully-redesign/session-05-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- All critical compose, preview, and shortcut flows are verified and documented.
- Remaining issues are triaged as blocking or non-blocking with file-level evidence.
- The release-readiness document states a clear ship recommendation.
```
