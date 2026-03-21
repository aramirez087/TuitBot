# Session 05: Hook Generation Engine

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/epic-charter.md`, `docs/roadmap/obsidian-ghostwriter-edge/block-contracts.md`, `docs/roadmap/obsidian-ghostwriter-edge/dashboard-ghostwriter-ux.md`, and `docs/roadmap/obsidian-ghostwriter-edge/session-04-handoff.md`.

Mission
Implement the backend hook-generation system that returns five differentiated hook options grounded in Ghostwriter context.

Repository anchors
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/rag_helpers.rs`
- `crates/tuitbot-core/src/content/generator/mod.rs`
- `crates/tuitbot-core/src/automation/seed_worker.rs`
- `crates/tuitbot-core/src/workflow/thread_plan.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `crates/tuitbot-core/src/content/generator/tests.rs`

Tasks
1. Add an assist contract for `Generate 5 Hook Options` that accepts Ghostwriter provenance and returns structured, differentiated hooks.
2. Reuse and extend existing hook extraction, thread-plan, and content-generation primitives so the feature sits on top of real domain logic.
3. Return enough metadata for downstream UI ranking and later measurement, but avoid inventing a heavy analytics subsystem in this session.
4. Add tests for prompt shaping, hook differentiation, and contract stability.

Deliverables
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/rag_helpers.rs`
- `crates/tuitbot-core/src/content/generator/mod.rs`
- `crates/tuitbot-core/src/automation/seed_worker.rs`
- `crates/tuitbot-core/src/workflow/thread_plan.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `crates/tuitbot-core/src/content/generator/tests.rs`
- `docs/roadmap/obsidian-ghostwriter-edge/hook-generation-contract.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-05-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`

Exit criteria
- The backend can return five differentiated hook options for Ghostwriter context.
- Session 06 can focus on the hook-first user flow instead of backend prompt mechanics.
```
