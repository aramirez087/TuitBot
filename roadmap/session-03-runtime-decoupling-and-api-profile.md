# Session 03 Prompt - Runtime Decoupling and API Profile

## Copy/paste prompt

You are executing Session 03 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: make MCP runtime launchable as a generic API client profile, independent from workflow-heavy state.

Critical constraints:

- No backward compatibility requirements.
- If a legacy coupling blocks progress, remove or replace it.

Required implementation work:

1. Add explicit MCP runtime profiles:
- `api` profile: generic X client tools first.
- `workflow` profile: TuitBot growth/approval/scoring features.
2. Extend CLI MCP serve command to accept profile selection.
3. Refactor state bootstrapping so `api` profile can run with minimal dependencies.
4. Ensure direct X tools operate without requiring workflow-specific DB tables.
5. Update `get_capabilities` to report active profile and loaded tool families.
6. Add tests for profile startup and profile-specific tool visibility.

Required artifacts to create:

- `roadmap/artifacts/session-03-profile-design.md`
- `roadmap/artifacts/session-03-runtime-decoupling-report.md`
- `roadmap/artifacts/session-03-handoff.md`

Definition of done:

- Running MCP in `api` profile is possible and tested.
- Direct X tool lane no longer hard-depends on approval/scoring workflow.
- Capabilities output is profile-aware and deterministic.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Exact CLI usage after this session.
2. What remains coupled and why.
3. What Session 04 will expand in endpoint coverage.
