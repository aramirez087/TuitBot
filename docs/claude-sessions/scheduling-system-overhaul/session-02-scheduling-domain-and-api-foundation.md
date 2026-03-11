# Session 02: Scheduling Domain And API Foundation

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Implement the canonical scheduling contract so manual content can be scheduled independently of direct publish capability and with timezone-safe timestamps.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/epic-charter.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/current-state-audit.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/implementation-map.md
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/utils/composeHandlers.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/api/client.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/api/types.ts
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/compose.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/draft_studio.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/scheduled.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/mod.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/compose_contract_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/draft_studio_api_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/tests.rs

Tasks
1. Introduce a shared scheduling helper or contract that validates timestamps, rejects clearly invalid past schedules, and documents the account-timezone rules for storage and display.
2. Remove implicit browser-local or ambiguous ISO handling from the client helpers so schedule payloads are explicit and round-trip safely.
3. Unify manual scheduling behavior across compose and Draft Studio without breaking existing routes; if compatibility shims are needed, keep them documented.
4. Make rescheduling a single logical operation instead of an unschedule-then-schedule race.
5. Expand backend and client tests around timestamp parsing, schedule, unschedule, reschedule, and cross-mode behavior.

Deliverables
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/utils/composeHandlers.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/api/client.ts
- /Users/aramirez/Code/ReplyGuy/dashboard/src/lib/api/types.ts
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/compose.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/draft_studio.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/src/routes/content/scheduled.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/mod.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/compose_contract_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/draft_studio_api_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/tests.rs
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/scheduling-domain-model.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/scheduling-api-contract.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- Schedule intent is no longer coupled to immediate publish capability.
- Timestamp handling has one documented contract across UI, API, and storage.
- Reschedule is atomic from the user's perspective and covered by tests.
```
