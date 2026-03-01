# Session 05: Validation And Release

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission

Validate the completed composer overhaul, fix ship-blocking regressions, and produce a release-readiness decision for the new compose experience.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/MediaSlot.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- docs/composer-mode.md
- docs/roadmap/composer-ui-typefully-plus/

Tasks
1. Run all required checks and fix only the regressions that block shipping the composer overhaul.
2. Perform a manual code audit of the main flows: open from calendar, draft a tweet, draft a thread, reorder thread items, attach media, use AI assist, schedule content, recover autosave, and close the modal cleanly.
3. Confirm the final UI still reflects the Session 01 charter and call out any deliberate deviations.
4. Write a go or no-go report with concrete evidence, known limitations, and any follow-up work that should not block release.

Deliverables
- docs/roadmap/composer-ui-typefully-plus/release-readiness.md
- docs/roadmap/composer-ui-typefully-plus/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check
- cd dashboard && npm run build

Exit criteria
- All blocking checks pass or failures are explicitly called out with a no-go decision.
- The release report gives a defensible ship decision.
- The handoff names any non-blocking follow-up work separately from release blockers.
```
