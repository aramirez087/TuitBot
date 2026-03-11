# Session 07: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission
Validate the full scheduling overhaul end to end and produce a go or no-go report with remaining risks called out precisely.

Repository anchors
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul
- /Users/aramirez/Code/ReplyGuy/dashboard/package.json
- /Users/aramirez/Code/ReplyGuy/Cargo.toml
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/compose_contract_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-server/tests/draft_studio_api_tests.rs
- /Users/aramirez/Code/ReplyGuy/crates/tuitbot-core/src/storage/scheduled_content/tests.rs

Tasks
1. Run the full backend and frontend quality gates and fix the final issues that are clearly in scope for this epic.
2. Review the implemented mode matrix, timezone handling, schedule creation, reschedule, unschedule, approval interaction, and calendar flows against the charter.
3. Reconcile the roadmap docs with the final code so the implementation, QA notes, and rollout caveats agree.
4. Write a concise release-readiness report with blockers, non-blockers, follow-up recommendations, and exact user-facing behavior after the overhaul.
5. Do not start new scope; only fix defects or inconsistencies required for go or no-go confidence.

Deliverables
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/release-readiness.md
- /Users/aramirez/Code/ReplyGuy/docs/roadmap/scheduling-system-overhaul/session-07-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd /Users/aramirez/Code/ReplyGuy/dashboard && npm run check

Exit criteria
- All quality gates pass or any exception is explicitly justified in release-readiness.md.
- The release-readiness report gives a clear go or no-go recommendation.
- The final handoff states whether the epic is complete or what exact follow-up remains.
```
