# Session 10 Prompt - Release Docs and Go Live

## Copy/paste prompt

You are executing Session 10 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: finalize release-grade docs, positioning, and operational readiness for the new multi-lane MCP platform.

Critical constraints:

- No backward compatibility requirements.
- Documentation must match real behavior in code and tests.

Required implementation work:

1. Update docs to present three clear MCP lanes:
- `official API client MCP`
- `scraper MCP` (optional/risk-aware)
- `workflow MCP` (TuitBot growth operations)
2. Publish a capability matrix that is generated from actual tool manifest data.
3. Write a migration guide from legacy single-lane MCP layout to new profiles/providers.
4. Update README and MCP reference examples for profile/provider selection.
5. Create a release readiness checklist and mark pass/fail against objective evidence.
6. Run full validation and capture final status.

Required artifacts to create:

- `roadmap/artifacts/session-10-final-capability-matrix.md`
- `roadmap/artifacts/session-10-migration-guide.md`
- `roadmap/artifacts/session-10-release-readiness.md`
- `roadmap/artifacts/session-10-handoff.md`

Definition of done:

- External users can understand and choose the correct MCP lane quickly.
- Docs, manifest, and actual tools are aligned.
- Release checklist includes technical, operational, and compliance gates.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Final architecture summary in one page.
2. Go/no-go recommendation with evidence.
3. Immediate post-release monitoring priorities.
