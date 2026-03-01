# Session 01: Benchmark And Charter

Paste this into a new Claude Code session:

```md
Mission

Audit the current composer, benchmark the best Typefully-like interaction patterns, and lock a concrete UI charter plus implementation slices for a cleaner thread-first compose experience.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/MediaSlot.svelte
- dashboard/src/lib/components/composer/ComposerShell.svelte
- dashboard/src/lib/components/composer/ThreadPreviewRail.svelte
- dashboard/src/app.css

Tasks
1. Audit the existing compose flow and identify the specific UI friction that still makes it feel heavier or noisier than Typefully when drafting or editing.
2. Capture the benchmark patterns to emulate or exceed: thread spine editing, low-noise top controls, command-first editing, focused writing canvas, contextual secondary actions, and clean mobile behavior.
3. Define a new composer interaction model that is explicitly thread-first while still supporting single tweets without a separate mental model.
4. Slice the work into the smallest safe implementation phases, with clear boundaries between shell/layout, thread interactions, and secondary controls/polish.
5. Keep code changes minimal in this session; only make tiny scaffolding edits if they reduce risk for later sessions.

Deliverables
- docs/roadmap/composer-ui-typefully-plus/charter.md
- docs/roadmap/composer-ui-typefully-plus/benchmark-notes.md
- docs/roadmap/composer-ui-typefully-plus/ui-architecture.md
- docs/roadmap/composer-ui-typefully-plus/session-01-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- The roadmap docs describe a clearly differentiated composer direction that is more polished than the current modal.
- Session boundaries are concrete enough that a later session can execute without re-planning.
- Any code touch in this session is low-risk and documented.
```
