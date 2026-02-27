# Session 03: Thread Composer Foundation UI

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.
Mission: Build a premium WYSIWYG thread composer foundation that clearly exceeds Typefully baseline UX.

Repository anchors:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/utils/tweetLength.ts`

Tasks:
1. Implement `ThreadComposer.svelte` with tweet-card visual structure matching live post composition.
2. Add a two-pane experience: editable thread cards plus live final-output preview.
3. Support add/remove/edit tweet cards with per-card character counters and validation states.
4. Integrate composer into `ComposeModal` for both tweet and thread modes using the Session 02 schema.
5. Preserve existing schedule and submit behavior from content/drafts pages.
6. Add accessibility semantics for labels, focus order, keyboard editing, and SR-friendly status updates.

Deliverables:
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `docs/roadmap/typefully-composer-ui-parity/session-03-ui-foundation.md`
- `docs/roadmap/typefully-composer-ui-parity/session-03-handoff.md`

Quality gates:
- cd dashboard && npm run check
- If Rust changed: cargo fmt --all && cargo fmt --all --check
- If Rust changed: RUSTFLAGS="-D warnings" cargo test --workspace
- If Rust changed: cargo clippy --workspace -- -D warnings

Exit criteria:
- Users can compose in cards while seeing live rendered output side-by-side.
- Validation prevents invalid submit payloads.
- Handoff includes reorder/media requirements for Session 04.
```
