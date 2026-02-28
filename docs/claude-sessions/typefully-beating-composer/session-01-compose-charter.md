# Session 01: Compose Charter

Paste this into a new Claude Code session:

```md
Continuity
- Read CLAUDE.md.
- Read docs/composer-mode.md.
- Read dashboard/src/routes/(app)/content/+page.svelte.
- Read dashboard/src/lib/components/ComposeModal.svelte.
- Read dashboard/src/lib/components/ThreadComposer.svelte.
- Read dashboard/src/lib/components/TweetPreview.svelte.
- Read dashboard/src/lib/components/FromNotesPanel.svelte.
- Read crates/tuitbot-server/src/routes/assist.rs.
- Read crates/tuitbot-server/src/routes/content/compose.rs.
- Read crates/tuitbot-server/tests/compose_contract_tests.rs.

Mission
Define the implementation charter and delivery slices for a composer overhaul that beats Typefully on focus and preview fidelity.

Repository anchors
- docs/composer-mode.md
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/TweetPreview.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-server/src/routes/content/compose.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs

Tasks
1. Audit the current compose flow and document the exact gaps against the two target differentiators.
2. Write a concrete architecture plan that preserves composer-mode guarantees, keeps compose payloads backward-compatible, and reduces the oversized Svelte surfaces.
3. Define a realistic v1 meaning for "learns your voice over time" using existing persona fields plus explicit reusable cues, not vague model-training claims.
4. Define what the preview must emulate, including tweet breaks, X-style media layouts, aspect-ratio handling, and visible crop expectations.
5. Split the implementation into the next two build sessions plus a final validation session, with explicit risk controls and fallback scope cuts.

Deliverables
- docs/roadmap/typefully-beating-composer/charter.md
- docs/roadmap/typefully-beating-composer/implementation-plan.md
- docs/roadmap/typefully-beating-composer/session-01-handoff.md

Quality gates
- Keep code changes limited to docs unless a small clarifying edit is required.
- If any Rust files change, run:
    cargo fmt --all && cargo fmt --all --check
    RUSTFLAGS="-D warnings" cargo test --workspace
    cargo clippy --workspace -- -D warnings
- If any dashboard files change, run cd dashboard && npm run check

Exit criteria
- The charter names the chosen v1 definition for voice learning and preview fidelity.
- The implementation plan clearly scopes Session 02, Session 03, and Session 04.
- The handoff lists the exact files Session 02 must read first.
```
