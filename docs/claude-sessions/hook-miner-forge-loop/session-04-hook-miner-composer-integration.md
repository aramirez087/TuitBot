# Session 04: Hook Miner Composer Integration

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Wire Hook Miner into both vault-backed entry paths so users see mined angles first and the old hook picker only as fallback.

Repository anchors
- dashboard/src/lib/components/composer/VaultSelectionReview.svelte
- dashboard/src/lib/components/composer/FromVaultPanel.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/composer/HookPicker.svelte
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- dashboard/tests/unit/
- docs/roadmap/hook-miner-forge-loop/hook-miner-ux.md
- docs/roadmap/hook-miner-forge-loop/hook-miner-api-contract.md

Tasks
1. Add client types and API wiring for `POST /api/assist/hook-miner` while preserving the existing `/api/assist/hooks` fallback call.
2. Replace the first vault-backed hook step with a Hook Miner step in both the Obsidian selection flow and the From Vault flow.
3. Render angle cards that show angle label, seed text, rationale, evidence badges, and lightweight source cues.
4. Add a clear “More hook styles” action that opens the existing HookPicker without losing current selection state, accepted neighbors, or output format.
5. Preserve current related-note acceptance and dismissal behavior and feed accepted neighbors into Hook Miner results.
6. Add unit tests for state transitions, fallback behavior, and format switching.
7. Document the UI state machine and copy choices.

Deliverables
- docs/roadmap/hook-miner-forge-loop/hook-miner-ui-notes.md
- docs/roadmap/hook-miner-forge-loop/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Both vault entry paths land on Hook Miner first.
- Users can fall back to generic hooks without losing context.
- The compose flow remains legible and reversible.
```
