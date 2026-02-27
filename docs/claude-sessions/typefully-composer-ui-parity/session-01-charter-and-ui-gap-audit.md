# Session 01: Charter And UI Gap Audit

Paste this into a new Claude Code session:

```md
Mission: Produce a final UI-first charter to ship a composer that is measurably better than Typefully.

Repository anchors:
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/api.ts`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `docs/composer-mode.md`

Tasks:
1. Audit current compose UX against Typefully and identify concrete opportunities to outperform it.
2. Lock the UI architecture: component boundaries, state ownership, and backend payload contracts.
3. Define superiority criteria with measurable targets (time-to-compose, shortcut coverage, edit friction, accessibility).
4. Document explicit non-goals: Ghostwriter engine, watchtower, RAG, and background seed systems.
5. Define phased implementation map for Sessions 02-08 with exact file targets.
6. Produce a risk register covering accessibility, mobile behavior, and payload compatibility.

Deliverables:
- `docs/roadmap/typefully-composer-ui-parity/charter.md`
- `docs/roadmap/typefully-composer-ui-parity/ui-gap-audit.md`
- `docs/roadmap/typefully-composer-ui-parity/superiority-scorecard.md`
- `docs/roadmap/typefully-composer-ui-parity/session-execution-map.md`
- `docs/roadmap/typefully-composer-ui-parity/session-01-handoff.md`

Quality gates:
- If Rust changed: cargo fmt --all && cargo fmt --all --check
- If Rust changed: RUSTFLAGS="-D warnings" cargo test --workspace
- If Rust changed: cargo clippy --workspace -- -D warnings
- If dashboard changed: cd dashboard && npm run check

Exit criteria:
- Charter has no unresolved decisions.
- Scorecard defines exact criteria proving “better than Typefully.”
- Non-goals clearly exclude Ghostwriter engine work.
- Handoff includes exact inputs for Session 02.
```
