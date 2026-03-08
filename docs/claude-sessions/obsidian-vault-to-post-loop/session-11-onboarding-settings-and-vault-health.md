# Session 11: Onboarding Settings And Vault Health

Paste this into a new Claude Code session:

```md
Continue from Session 10 artifacts.

Continuity
- The setup and settings surfaces must describe the real runtime contract that now exists, not the old generic content-source story.

Mission
- Redesign onboarding, settings, and health surfaces so the vault feels like a trustworthy product area instead of a config subsection.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/ux-blueprint.md`
- `docs/roadmap/obsidian-vault-to-post-loop/source-lifecycle.md`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`
- `dashboard/src/lib/stores/onboarding.ts`
- `dashboard/src/lib/stores/runtime.ts`

Tasks
1. Make onboarding and settings copy truthful about local folders, Drive behavior, sync semantics, and loop-back support.
2. Add vault health visibility such as last sync, status, note or fragment counts, and actionable errors or reindex controls where the backend supports them.
3. Remove or fix misleading optional-field UX, especially where runtime requirements are stricter than current copy suggests.
4. Keep desktop, self-hosted, and cloud flows mode-aware without hiding important limitations.
5. Document the setup UX and health model.

Deliverables
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`
- `dashboard/src/lib/stores/onboarding.ts`
- `dashboard/src/lib/stores/runtime.ts`
- `docs/roadmap/obsidian-vault-to-post-loop/source-setup-ux.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-11-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Source setup now matches runtime truth, vault health is visible, and users can understand the feature without reading implementation details.
```
