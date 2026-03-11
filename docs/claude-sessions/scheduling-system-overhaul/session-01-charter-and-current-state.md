# Session 01: Charter And Current State

Paste this into a new Claude Code session:

```md
Continuity
Start from current repository state.

Mission
Audit the existing scheduling system and write the initiative charter, target experience, and implementation map without making broad code changes.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/HomeComposerHeader.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/TimePicker.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftScheduleSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftStudioShell.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/calendar.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/content/+page.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/compose.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/draft_studio.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/calendar.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/mod.rs

Tasks
1. Audit every user-facing scheduling entry point, state transition, and API path for manual content, including compose, Draft Studio, calendar editing, scheduled content editing, and approval interactions.
2. Call out the concrete gaps blocking first-class scheduling, including publish-vs-schedule coupling, dual API paths, timezone and ISO handling risks, non-atomic reschedule behavior, and mode-specific inconsistencies.
3. Summarize external UX patterns from Typefully, Buffer, and Planable that are worth adopting, with links and direct implications for ReplyGuy.
4. Define the product goals, non-goals, success metrics, rollout risks, and the session-by-session execution map for this epic.
5. Keep code changes minimal; only create or touch docs unless a tiny clarification patch is unavoidable.

Deliverables
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/epic-charter.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/current-state-audit.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/competitive-ux-notes.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/implementation-map.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-01-handoff.md

Exit criteria
- The charter names the user problem in exact terms, including why scheduling must remain available even when direct publish is possible.
- The audit references every relevant scheduling surface and backend path.
- The handoff gives Session 02 exact starting files and decisions to honor.
```
