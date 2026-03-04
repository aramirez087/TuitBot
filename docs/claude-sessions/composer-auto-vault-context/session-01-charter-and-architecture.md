# Session 01: Charter and Architecture

Paste this into a new Claude Code session:

```md
Continuity
- Start from the current repository state only.

Mission
Audit the composer assist stack and produce the implementation charter for automatic vault context.

Repository anchors
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-core/src/context/winning_dna.rs
- crates/tuitbot-core/src/content/generator/mod.rs
- crates/tuitbot-core/src/workflow/draft.rs
- docs/composer-mode.md

Tasks
1. Trace how composer tweet, thread, and improve flows currently bypass the draft-context RAG path used by reply drafting.
2. Decide the target design for automatic composer RAG, including helper placement, keyword sourcing, fallback behavior, and how `AssistImproveRequest.context` coexists with automatic vault context.
3. Capture explicit non-goals: no new UI, no manual note picker, no request-shape changes.
4. Write a charter and implementation map that names the remaining sessions, risks, and validation strategy.
5. Write the session handoff with exact inputs for Session 02.

Deliverables
- docs/roadmap/composer-auto-vault-context/charter.md
- docs/roadmap/composer-auto-vault-context/implementation-plan.md
- docs/roadmap/composer-auto-vault-context/session-01-handoff.md

Quality gates
- Verify every referenced file exists and every decision ties back to the current codebase.
- Keep this session planning-only; avoid feature code unless a tiny proof is required for clarity.

Exit criteria
- The charter explains the architecture and main tradeoffs clearly enough that Session 02 can implement core API changes without re-auditing.
```
