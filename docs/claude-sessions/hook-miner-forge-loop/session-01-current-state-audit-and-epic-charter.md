# Session 01: Current State Audit And Epic Charter

Paste this into a new Claude Code session:

```md
Mission
Audit the real Ghostwriter, hook, provenance, publish, analytics, and loopback flow and lock the charter for Hook Miner + Forge.

Repository anchors
- plugins/obsidian-tuitbot/src/main.ts
- crates/tuitbot-server/src/routes/vault/selections.rs
- dashboard/src/lib/components/composer/VaultSelectionReview.svelte
- dashboard/src/lib/components/composer/FromVaultPanel.svelte
- crates/tuitbot-core/src/automation/watchtower/loopback.rs
- crates/tuitbot-core/src/automation/approval_poster.rs
- docs/roadmap/obsidian-ghostwriter-edge/
- docs/claude-sessions/backlink-synthesizer/

Tasks
1. State precisely what already exists today for Obsidian selection ingress, graph-aware related notes, hook generation, provenance propagation, publish writeback, and analytics storage.
2. Name the real gaps for Hook Miner and Forge without re-solving the completed Backlink Synthesizer work.
3. Call out the current thread-specific weakness in publish persistence and analytics measurement.
4. Define the product charter, success bar, non-goals, and rollout stance for this epic.
5. Break the implementation into the smallest safe follow-on sessions and verify the repository anchors for each.

Deliverables
- docs/roadmap/hook-miner-forge-loop/current-state-audit.md
- docs/roadmap/hook-miner-forge-loop/epic-charter.md
- docs/roadmap/hook-miner-forge-loop/implementation-map.md
- docs/roadmap/hook-miner-forge-loop/session-01-handoff.md

Quality gates
- No code changes required unless a tiny clarifying doc fix is necessary.

Exit criteria
- The audit separates shipped behavior from new epic scope with no guesswork.
- The charter makes Hook Miner and Forge feel like one coherent product move.
- The implementation map explicitly names the thread and analytics normalization gap.
```
