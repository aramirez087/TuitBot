# Session 06: Responsive Accessible Polish

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.
Mission: Harden the composer UX to premium quality across performance, accessibility, and responsiveness.

Repository anchors:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `dashboard/src/app.css`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/routes/(app)/drafts/+page.svelte`

Tasks:
1. Optimize mobile and tablet layouts for composer, card editing, and media handling.
2. Validate and fix keyboard-only flows (reorder, media assignment, submit, modal close).
3. Improve contrast, focus indicators, and error messaging for accessibility compliance.
4. Add motion and transitions that feel intentional without harming responsiveness.
5. Remove visual regressions and align spacing/typography with the app design language.

Deliverables:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/ThreadComposer.svelte`
- `docs/roadmap/typefully-composer-ui-parity/session-06-polish-notes.md`
- `docs/roadmap/typefully-composer-ui-parity/session-06-handoff.md`

Quality gates:
- cd dashboard && npm run check
- If Rust changed: cargo fmt --all && cargo fmt --all --check
- If Rust changed: RUSTFLAGS="-D warnings" cargo test --workspace
- If Rust changed: cargo clippy --workspace -- -D warnings

Exit criteria:
- Composer UX is stable across desktop/mobile with keyboard-accessible core actions.
- Interactions remain smooth under realistic thread sizes (document measured behavior).
- No known high-severity UI regressions remain.
- Handoff includes final validation script for Session 07.
```
