# Session 01: Charter And Architecture

Paste this into a new Claude Code session:

```md
Continuity
- Start from repository state only and create the first roadmap artifacts under `docs/roadmap/obsidian-ghostwriter-edge/`.

Mission
Define the implementation charter for a Ghostwriter workflow that gives TuitBot a credible power-user edge over Typefully while staying grounded in the existing vault, compose, and privacy architecture.

Repository anchors
- `README.md`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/VaultNoteList.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/composer/CitationChips.svelte`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `dashboard/src/lib/utils/obsidianUri.ts`
- `dashboard/src-tauri/src/lib.rs`
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/rag_helpers.rs`
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-core/src/automation/seed_worker.rs`

Tasks
1. Audit the current vault search, chunk selection, highlight extraction, citation, compose, draft-studio, and Obsidian deep-link flow.
2. Define the target user journeys for block-level send, hook-first thread drafting, and local-first privacy, explicitly separating Desktop, Self-host, and Cloud behavior.
3. Choose the implementation path for exact block identity and ingress, preferring existing `chunk_id` and provenance primitives unless the current chunk model is provably insufficient.
4. Produce the core roadmap artifacts and a phased plan for the later sessions; keep code changes minimal and avoid speculative scaffolding.

Deliverables
- `docs/roadmap/obsidian-ghostwriter-edge/epic-charter.md`
- `docs/roadmap/obsidian-ghostwriter-edge/current-state-audit.md`
- `docs/roadmap/obsidian-ghostwriter-edge/ghostwriter-architecture.md`
- `docs/roadmap/obsidian-ghostwriter-edge/privacy-and-deployment-matrix.md`
- `docs/roadmap/obsidian-ghostwriter-edge/implementation-map.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-01-handoff.md`

Quality gates
- The artifacts are internally consistent, name exact file paths, and contain no `TBD` or `TODO` placeholders.
- The architecture states clear behavior for Desktop, Self-host, and Cloud modes.
- The handoff names the exact inputs Session 02 must consume.

Exit criteria
- Another engineer can start Session 02 without verbal context.
- The block identity, ingress path, and privacy boundary are decided and documented.
```
