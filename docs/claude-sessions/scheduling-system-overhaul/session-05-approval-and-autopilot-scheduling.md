# Session 05: Approval And Autopilot Scheduling

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Make scheduling behavior coherent across Autopilot, Composer, and Approval modes so approval policy changes never erase a user's intent to schedule.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/epic-charter.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/current-state-audit.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/draft-studio-and-calendar-ux.md
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/settings/SafetyLimitsSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/approval.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/approval/+page.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ApprovalCard.svelte
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/compose.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/approval.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/approval_queue/mod.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/approval_queue/queries.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/workflow/tests.rs
- /Users/aramirez/Code/ReplyGuy/migrations/20260222000004_approval_queue.sql

Tasks
1. Define and implement how scheduled manual posts behave when approval is on, including how the scheduled time is stored, surfaced, edited, and honored after approval.
2. Ensure Autopilot plus Approval off does not suppress scheduling affordances or force immediate publish from manual compose flows.
3. Update approval UI and settings copy so users can understand direct publish, scheduled publish, and scheduled items awaiting approval.
4. Add test coverage for mode combinations and any approval queue schema or API changes required to preserve schedule intent.
5. Document the final mode matrix and rollout risks.

Deliverables
- /Users/aramirez/Code/ReplyGuy/migrations/20260311000100_scheduled_approval_intent.sql
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/settings/SafetyLimitsSection.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/routes/(app)/approval/+page.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/components/ApprovalCard.svelte
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/stores/approval.ts
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/compose.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/approval.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/approval_queue/mod.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/approval_queue/queries.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/workflow/tests.rs
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/mode-and-approval-matrix.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- Manual scheduling survives mode switches and approval requirements.
- The mode matrix is explicit in code, UI copy, and docs.
- Tests cover Autopilot and Composer with Approval on and off.
```
