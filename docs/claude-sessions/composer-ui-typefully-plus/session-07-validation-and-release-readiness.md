# Session 07: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission
Run end-to-end validation on the composer-first home experience and produce a release-quality go or no-go report.

Repository anchors
- dashboard/src/routes/(app)/+page.svelte
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/routes/(app)/settings/+page.svelte
- dashboard/src/lib/stores/homeSurface.ts
- docs/roadmap/composer-ui-typefully-plus/charter.md
- docs/roadmap/composer-ui-typefully-plus/release-readiness.md

Tasks
1. Run and record the required frontend and Rust checks, then note exact failures or confirm a clean pass.
2. Manually verify the critical flows:
   - fresh launch lands on composer
   - Settings flips home surface to analytics and the choice persists
   - full-page compose works
   - modal compose from calendar still works
   - split, merge, reorder, preview, schedule, publish, AI improve, from notes, autosave, and recovery all behave correctly
   - mobile and narrow-width layouts remain usable
3. Compare the finished experience against the charter and benchmark, then call out any remaining gaps that still keep it behind Typefully.
4. Update `docs/roadmap/composer-ui-typefully-plus/release-readiness.md` with a clear go or no-go verdict, evidence, and any non-blocking follow-up work.
5. Write a final handoff that either closes the epic or lists the exact blockers and the smallest next session needed to clear them.

Deliverables
- docs/roadmap/composer-ui-typefully-plus/release-readiness.md
- docs/roadmap/composer-ui-typefully-plus/session-07-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The release-readiness report makes a defensible go or no-go call.
- The critical flows are verified against the finished implementation, not assumptions.
- Anyone picking up the epic next knows whether it is complete or exactly what remains.
```
