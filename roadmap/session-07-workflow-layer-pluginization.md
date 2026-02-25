# Session 07 Prompt - Workflow Layer Pluginization

## Copy/paste prompt

You are executing Session 07 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: keep TuitBot's growth workflow strength, but isolate it as an optional plugin layer above the generic MCP core.

Critical constraints:

- No backward compatibility requirements.
- Generic API MCP must not depend on workflow plugin internals.

Required implementation work:

1. Identify workflow-only tools and move them behind a dedicated workflow extension boundary.
2. Register tools by profile/lane:
- generic API lane loads by default in `api` profile
- workflow lane loads only in `workflow` profile
3. Remove residual compile-time and runtime coupling from core generic tools to workflow services.
4. Keep policy and approval semantics available for workflow profile only.
5. Add tests validating tool set composition per profile.

Required artifacts to create:

- `roadmap/artifacts/session-07-workflow-tool-boundary.md`
- `roadmap/artifacts/session-07-toolset-by-profile.md`
- `roadmap/artifacts/session-07-handoff.md`

Definition of done:

- Clear separation between interoperability lane and growth-operations lane.
- API profile can be used as a clean "Twitter client as tools" package.
- Workflow profile preserves TuitBot value without contaminating generic contracts.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Which tools are now workflow-only.
2. Which tools are now guaranteed generic.
3. What Session 08 will add for scraper lane.
