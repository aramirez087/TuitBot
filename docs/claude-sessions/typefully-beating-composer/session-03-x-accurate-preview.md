# Session 03: X-Accurate Preview

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Read docs/roadmap/typefully-beating-composer/charter.md.
- Read docs/roadmap/typefully-beating-composer/implementation-plan.md.
- Read docs/roadmap/typefully-beating-composer/session-02-handoff.md.

Mission
Build a high-fidelity thread preview that mirrors X's card breaks and media crop behavior closely enough to beat generic composer previews.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/TweetPreview.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/MediaSlot.svelte
- dashboard/src/lib/api.ts
- docs/composer-mode.md

Tasks
1. Extract the preview rail into focused components so preview logic is maintainable and every touched Svelte file stays within the repo size limits.
2. Replace the generic preview rendering with an X-accurate layout model for single tweets and threads, including tweet spacing, connectors, and 1-4 media arrangements.
3. Use intrinsic image and video dimensions from local previews or served media URLs to render realistic crop windows instead of a generic fixed-ratio placeholder.
4. Keep preview state synchronized with reorder, split, merge, delete, recovery, and mobile stacking behavior without introducing lag or stale media.
5. Document the fidelity rules you implemented and add deterministic helper logic where needed so future preview tweaks do not become guesswork.

Deliverables
- dashboard/src/lib/components/composer/ThreadPreviewRail.svelte
- dashboard/src/lib/components/composer/MediaCropPreview.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/TweetPreview.svelte
- dashboard/src/lib/components/MediaSlot.svelte
- dashboard/src/lib/api.ts
- docs/composer-mode.md
- docs/roadmap/typefully-beating-composer/session-03-handoff.md

Quality gates
- Run cd dashboard && npm run check.
- If any Rust files change, run:
    cargo fmt --all && cargo fmt --all --check
    RUSTFLAGS="-D warnings" cargo test --workspace
    cargo clippy --workspace -- -D warnings

Exit criteria
- The preview communicates likely tweet breaks and media crops without obvious placeholder behavior.
- Desktop and mobile preview layouts remain usable and visually stable.
- The handoff lists the exact files and checks Session 04 must validate.
```
