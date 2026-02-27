# Session 04: Reorder And Media Placement UI

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.
Mission: Deliver best-in-class structural control and media choreography that outperforms Typefully.

Repository anchors:
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/api.ts`
- `crates/tuitbot-server/src/routes/content/compose.rs`

Tasks:
1. Add reorder controls (move up/down plus drag handle) with keyboard-accessible equivalents.
2. Add power actions: duplicate tweet card, split card, and merge-adjacent cards.
3. Implement per-tweet media slot assignment and reassignment in the composer UI.
4. Ensure payload serialization preserves order and media mapping deterministically.
5. Add boundary handling for X media rules, inline errors, and tests/checks for persistence.

Deliverables:
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `docs/roadmap/typefully-composer-ui-parity/session-04-reorder-media.md`
- `docs/roadmap/typefully-composer-ui-parity/session-04-handoff.md`

Quality gates:
- cd dashboard && npm run check
- If Rust changed: cargo fmt --all && cargo fmt --all --check
- If Rust changed: RUSTFLAGS="-D warnings" cargo test --workspace
- If Rust changed: cargo clippy --workspace -- -D warnings

Exit criteria:
- Reordered cards persist correctly through submit/edit cycles.
- Power actions (duplicate/split/merge) preserve content and ordering safely.
- Media can be moved between tweet cards without data loss.
- Handoff identifies distraction-free mode tasks for Session 05.
```
