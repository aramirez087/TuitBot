# Session 06: Polish Telemetry And QA

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Finish the scheduling overhaul with telemetry, copy polish, accessibility, and a scenario-based QA matrix.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/mode-and-approval-matrix.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/draft-studio-and-calendar-ux.md
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/analytics/funnel.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftScheduleSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ContentItem.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/compose_contract_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/draft_studio_api_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/tests.rs

Tasks
1. Add instrumentation for schedule create, reschedule, unschedule, schedule-to-approval, and publish-now paths where it helps measure adoption and confusion.
2. Sweep copy, empty states, error messages, and screen-reader announcements so the new scheduling flows are explicit and accessible.
3. Write a QA matrix covering desktop, mobile, direct publish availability, approval on or off, timezone differences, and missed or past slot handling.
4. Add missing regression tests discovered during the sweep and remove dead scheduling UI or copy if it still duplicates behavior.
5. Keep the roadmap docs aligned with what actually shipped.

Deliverables
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/analytics/funnel.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/drafts/DraftScheduleSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ContentItem.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/compose_contract_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/draft_studio_api_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/tests.rs
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/qa-matrix.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/telemetry-and-copy-notes.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-06-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- The shipped flows have explicit success and error states.
- QA covers the main schedule, publish, and approval permutations.
- Telemetry and copy are documented and implemented consistently.
```
