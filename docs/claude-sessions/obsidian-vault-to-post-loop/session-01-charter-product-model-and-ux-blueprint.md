# Session 01: Charter Product Model And UX Blueprint

Paste this into a new Claude Code session:

```md
Continuity
- Start from current repository state only.
- Read the listed anchors and existing vault/content-source roadmaps before deciding anything.

Mission
- Define the north-star product model, UX, and technical plan for turning Tuitbot's vault support into a real Obsidian-native note-to-post loop.

Repository anchors
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `docs/roadmap/deployment-aware-content-source-setup/charter.md`
- `docs/roadmap/composer-auto-vault-context/charter.md`

Tasks
1. Audit the current vault, Watchtower, composer, reply, settings, and posting flows as one connected system.
2. Write a charter that locks these defaults: per-account vault isolation, chunk-level retrieval with citations, explicit provenance, real loop-back, and consistent vault context across composer and reply surfaces.
3. Define the product model for sources, notes, fragments, selected references, citations, draft provenance, post provenance, and loop-back state.
4. Define the UX blueprint for onboarding, settings, vault health, From Vault in composer, reply assistance, citations, and desktop Obsidian affordances.
5. Produce an implementation plan aligned to Sessions 02-13 with concrete defaults and no deferred decisions.
6. Keep this session planning-first; code changes should be minimal and only support documentation accuracy.

Deliverables
- `docs/roadmap/obsidian-vault-to-post-loop/charter.md`
- `docs/roadmap/obsidian-vault-to-post-loop/product-model.md`
- `docs/roadmap/obsidian-vault-to-post-loop/ux-blueprint.md`
- `docs/roadmap/obsidian-vault-to-post-loop/implementation-plan.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-01-handoff.md`

Quality gates
- No broad code changes expected. Verify every referenced path exists and every later-session dependency is explicit.

Exit criteria
- The documents contain no TBDs, choose concrete product defaults, and make the remaining sessions executable without re-planning.
```
