# Session 02: Distraction-Free Writer

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Read docs/roadmap/typefully-beating-composer/charter.md.
- Read docs/roadmap/typefully-beating-composer/implementation-plan.md.
- Read docs/roadmap/typefully-beating-composer/session-01-handoff.md.

Mission
Build the distraction-free writing assistant slice so rough notes become polished content inside a focused, voice-aware composer.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/routes/(app)/settings/ContentPersonaSection.svelte
- dashboard/src/lib/api.ts
- crates/tuitbot-core/src/content/generator.rs
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs

Tasks
1. Refactor the compose surface into smaller focused components so every touched Svelte file ends at 400 lines or fewer.
2. Rework the compose layout into a writing-first studio that keeps non-writing chrome out of focus mode and makes note-to-draft actions feel primary.
3. Add a concrete voice-memory loop that turns existing persona settings plus explicit reusable writer cues into assist context for tweet, thread, and improve requests.
4. Upgrade the notes-to-content flow so thread generation preserves source-note context, shows clear loading and error states, and avoids destructive replacement surprises.
5. Update nearby docs and contract tests for any request or response changes introduced by the writer-assist slice.

Deliverables
- dashboard/src/lib/components/composer/ComposerShell.svelte
- dashboard/src/lib/components/composer/VoiceContextPanel.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/lib/api.ts
- crates/tuitbot-core/src/content/generator.rs
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs
- docs/roadmap/typefully-beating-composer/session-02-handoff.md

Quality gates
- Run cd dashboard && npm run check.
- If any Rust files change, run:
    cargo fmt --all && cargo fmt --all --check
    RUSTFLAGS="-D warnings" cargo test --workspace
    cargo clippy --workspace -- -D warnings

Exit criteria
- The composer feels materially more writing-focused than the pre-epic modal.
- Voice cues are visible, reusable, and actually influence assist requests.
- The handoff lists the exact preview files and contracts Session 03 must build on.
```
