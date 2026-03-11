# Session 03: Compose Scheduling Experience

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Make scheduling a first-class composer action with a faster and clearer UX than the current inspector-hidden time picker.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/competitive-ux-notes.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/scheduling-domain-model.md
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/HomeComposerHeader.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposerCanvas.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/InspectorContent.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/TimePicker.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/utils/composeHandlers.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/api/types.ts

Tasks
1. Redesign the compose scheduling flow so schedule is visible beside Publish, even when direct publish is enabled.
2. Replace or upgrade the current time picker to support timezone-aware date plus time selection, quick actions like Next free slot and Add to queue slot, and a clear unschedule state.
3. Make the CTA copy and state transitions explicit so Publish now, Schedule, and fallback states always match real behavior.
4. Preserve keyboard access, mobile usability, and autosave or recovery behavior for drafts.
5. Add or update component and logic tests for the new compose scheduling states.

Deliverables
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/HomeComposerHeader.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposerCanvas.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/InspectorContent.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ScheduleComposerSheet.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/TimePicker.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/utils/composeHandlers.ts
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/composer-scheduling-ux.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- A user can discover scheduling without opening the inspector first.
- Publish and schedule actions can coexist without ambiguity.
- The UI shows the active timezone and the selected schedule in plain language.
```
