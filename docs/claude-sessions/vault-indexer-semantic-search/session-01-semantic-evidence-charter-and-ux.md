# Session 01: Semantic Evidence Charter And UX

Paste this into a new Claude Code session:

```md
Mission
Define the product charter, UX, and implementation architecture for a background semantic vault index that upgrades Ghostwriter into an evidence-first drafting workspace.

Repository anchors
- CLAUDE.md
- docs/composer-mode.md
- docs/roadmap/obsidian-ghostwriter-edge/current-state-audit.md
- docs/roadmap/obsidian-ghostwriter-edge/dashboard-ghostwriter-ux.md
- docs/roadmap/obsidian-ghostwriter-edge/hook-first-workflow.md
- docs/roadmap/backlink-synthesizer/epic-charter.md
- docs/roadmap/backlink-synthesizer/ghostwriter-entry-flow.md
- dashboard/src/lib/components/composer/FromVaultPanel.svelte
- dashboard/src/lib/components/composer/VaultSelectionReview.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte
- crates/tuitbot-server/src/routes/vault/mod.rs
- crates/tuitbot-server/src/routes/vault/selections.rs
- crates/tuitbot-core/src/automation/watchtower/mod.rs

Tasks
1. Audit the live tweet, thread, selection, hook, and slot-refinement flow and name the exact seams where semantic evidence helps instead of adding clutter.
2. Define the best UX for search-before-generation and search-during-editing, including auto-query behavior, scope controls, index-status affordances, empty states, degraded states, and result actions.
3. Choose the preferred architecture for real-time indexing: semantic unit granularity, dirty-state tracking, embedding provider abstraction, index backend, freshness model, and deployment behavior.
4. Specify how semantic retrieval coexists with graph suggestions, keyword search, hook generation, thread slots, provenance, and current assist endpoints.
5. Split the work into the smallest safe follow-on sessions with file-level anchors and measurable success criteria.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/current-state-audit.md
- docs/roadmap/vault-indexer-semantic-search/epic-charter.md
- docs/roadmap/vault-indexer-semantic-search/semantic-index-architecture.md
- docs/roadmap/vault-indexer-semantic-search/ghostwriter-evidence-ux.md
- docs/roadmap/vault-indexer-semantic-search/implementation-map.md
- docs/roadmap/vault-indexer-semantic-search/session-01-handoff.md

Quality gates
- No code changes required unless a tiny clarifying doc fix is necessary.

Exit criteria
- The audit reflects the real compose flow and current graph work.
- The UX makes semantic search feel like a power tool, not a parallel workflow.
- The architecture chooses a concrete index and update strategy with no placeholders.
- The implementation map names safe follow-on sessions and measurable outcomes.
```
