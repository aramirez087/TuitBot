# Session 04: Draft Studio And Calendar UX

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Bring the same high-quality scheduling UX to Draft Studio and the content calendar so editing, rescheduling, and slot selection feel consistent.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/composer-scheduling-ux.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/scheduling-api-contract.md
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftScheduleSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftStudioShell.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftMetadataSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftHistoryPanel.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/draftStudio.svelte.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/CalendarWeekView.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/CalendarMonthView.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ContentItem.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/calendar.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/content/+page.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/calendar.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/scheduled.rs

Tasks
1. Reuse the new scheduling primitives in Draft Studio instead of maintaining a second, simpler scheduler UI.
2. Improve scheduled item affordances in the calendar and detail views so reschedule, unschedule, and edit actions are obvious and consistent.
3. Make slot selection and prefill flows respect the canonical timezone contract end to end.
4. Improve scheduled-state copy, metadata, and history so users can tell exactly when and how a post will go out.
5. Add regression coverage for Draft Studio scheduling, calendar prefill, and scheduled item editing.

Deliverables
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftScheduleSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftStudioShell.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftMetadataSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/draftStudio.svelte.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/CalendarWeekView.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ContentItem.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/calendar.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/content/+page.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/calendar.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/scheduled.rs
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/draft-studio-and-calendar-ux.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- Compose, Draft Studio, and calendar all express scheduling with shared rules and matching copy.
- Calendar-driven prefill and manual reschedule round-trip correctly in the configured timezone.
- Scheduled items expose edit and unschedule actions without hidden state.
```
