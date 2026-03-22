# Session 01: Graph RAG Charter And Audit

Paste this into a new Claude Code session:

```md
Mission
Define the product charter and architecture for graph-aware Ghostwriter retrieval around a selected note.

Repository anchors
- crates/tuitbot-server/src/routes/rag_helpers.rs
- crates/tuitbot-server/src/routes/vault/mod.rs
- crates/tuitbot-server/src/routes/vault/selections.rs
- crates/tuitbot-core/src/context/retrieval.rs
- crates/tuitbot-core/src/context/winning_dna/analysis.rs
- crates/tuitbot-core/src/automation/watchtower/mod.rs
- crates/tuitbot-core/src/storage/watchtower/
- plugins/obsidian-tuitbot/src/main.ts
- docs/roadmap/obsidian-ghostwriter-edge/

Tasks
1. Audit the current Ghostwriter and vault pipeline and state precisely what exists today versus what the Backlink Synthesizer requires.
2. Define the full user journey from Obsidian selection to finished draft, including loading, empty, success, dismissal, recovery, and return-to-source states.
3. Define the target UX for related-note synthesis after note selection, including suggestion cards, reason labels, user controls, thread-slot insertion flows, and how accepted suggestions change the draft without surprising the user.
4. Specify the preferred architecture: link extraction, normalized tags, graph expansion, ranking, provenance, and fallback behavior when graph data is sparse.
5. Split the implementation into the smallest safe sequence of follow-on sessions.

Deliverables
- docs/roadmap/backlink-synthesizer/current-state-audit.md
- docs/roadmap/backlink-synthesizer/epic-charter.md
- docs/roadmap/backlink-synthesizer/graph-rag-architecture.md
- docs/roadmap/backlink-synthesizer/end-to-end-ux-journey.md
- docs/roadmap/backlink-synthesizer/implementation-map.md
- docs/roadmap/backlink-synthesizer/session-01-handoff.md

Quality gates
- No code changes required unless a tiny clarifying doc fix is necessary.

Exit criteria
- The audit names the real implementation gap, not guesses.
- The charter explains why graph-aware retrieval is product-critical.
- The UX journey makes the whole flow legible before implementation starts.
- The architecture chooses a concrete storage and retrieval design with no TBDs.
- The implementation map names the next sessions with file-level anchors.
```
