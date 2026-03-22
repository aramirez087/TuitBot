# Session 02: Hook Miner Product Spec

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Lock the evidence-first Hook Miner UX and contract so implementation can proceed without product ambiguity.

Repository anchors
- dashboard/src/lib/components/composer/VaultSelectionReview.svelte
- dashboard/src/lib/components/composer/FromVaultPanel.svelte
- dashboard/src/lib/components/composer/HookPicker.svelte
- dashboard/src/lib/components/composer/GraphSuggestionCards.svelte
- crates/tuitbot-server/src/routes/assist/hooks.rs
- docs/roadmap/hook-miner-forge-loop/current-state-audit.md
- docs/roadmap/hook-miner-forge-loop/epic-charter.md

Tasks
1. Replace the current vault-backed first step of “generic 5 hooks” with “3 mined angles” for both Obsidian selection and From Vault flows.
2. Define the canonical angle taxonomy as exact enums: `story`, `listicle`, and `hot_take`.
3. Define the evidence taxonomy as exact enums: `contradiction`, `data_point`, and `aha_moment`.
4. Specify the angle-card UX: seed text, rationale, evidence badges, citation cues, loading states, sparse-note states, and recovery states.
5. Specify weak-signal fallback: if mined evidence is too thin, the user sees an explicit fallback state and can open the existing generic hook picker without losing context.
6. Define how accepted related notes influence angle generation without auto-inserting content into the draft.
7. Write precise copy and non-goals for the first version.

Deliverables
- docs/roadmap/hook-miner-forge-loop/hook-miner-ux.md
- docs/roadmap/hook-miner-forge-loop/hook-miner-contract.md
- docs/roadmap/hook-miner-forge-loop/session-02-handoff.md

Quality gates
- No code changes required unless a tiny clarifying doc fix is necessary.

Exit criteria
- The UX spec is concrete enough to build without extra product decisions.
- The fallback path is explicit and non-destructive.
- Related-note influence is defined without surprising draft mutation.
```
