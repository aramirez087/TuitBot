# Session 05: Distraction Free Writing Mode

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.
Mission: Implement a distraction-free writing system that feels faster and smarter than Typefully, without Ghostwriter engine dependencies.

Repository anchors:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/api.ts`

Tasks:
1. Add a focused writing panel/state that hides non-essential UI chrome while composing.
2. Add a command palette with keyboard shortcuts for core writing actions and card transforms.
3. Add simple assist actions using existing `/api/assist/*` endpoints only (no new ghostwriter backend).
4. Add “from notes” helper UX using local text input/import area, not background ingestion.
5. Ensure escape hatches: return to full composer, preserve unsaved edits, and keep keyboard shortcuts.

Deliverables:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `docs/roadmap/typefully-composer-ui-parity/session-05-distraction-free.md`
- `docs/roadmap/typefully-composer-ui-parity/session-05-handoff.md`

Quality gates:
- cd dashboard && npm run check
- If Rust changed: cargo fmt --all && cargo fmt --all --check
- If Rust changed: RUSTFLAGS="-D warnings" cargo test --workspace
- If Rust changed: cargo clippy --workspace -- -D warnings

Exit criteria:
- Composer can run in a focused writing mode with no loss of functionality.
- Command palette and shortcuts reduce pointer-heavy interactions.
- Assist actions work through existing APIs.
- Handoff includes responsiveness/accessibility priorities for Session 06.
```
