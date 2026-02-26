# Session 07 Prompt: Final Hardening and Ship Checklist

## Use this as your Claude Code prompt

You are implementing Session 07 (final) of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Final quality pass, risk closure, and release readiness.

Scope for this session:
1. Run final validation suite and fix any breakages:
   - `cargo fmt --all --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --workspace`
   - `mkdocs build --strict`
2. Verify generated manifest artifacts are up-to-date and committed.
3. Verify docs and CLI help are aligned on canonical profile names and counts.
4. Add/refresh a short operator runbook section for profile verification:
   - how to list tools per profile
   - how to confirm read-only guarantee quickly
5. Produce a release checklist section in epic docs summarizing:
   - completed tasks
   - known limitations
   - rollback strategy

Acceptance criteria:
- `readonly` and `api-readonly` are demonstrably mutation-free by registration.
- `full` remains fully capable.
- CI contains drift guards for docs vs manifest.
- Release notes are complete.

Deliverables:
- Final code/doc polish.
- Ship-ready summary with explicit pass/fail checklist.
