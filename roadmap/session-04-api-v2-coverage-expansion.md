# Session 04 Prompt - API v2 Coverage Expansion

## Copy/paste prompt

You are executing Session 04 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: close high-value endpoint gaps so this MCP can stand as a general-purpose X API toolset.

Critical constraints:

- No backward compatibility requirements.
- Prioritize correctness, predictable inputs/outputs, and pagination consistency.

Required implementation work:

1. Use Session 01 backlog and implement missing high-value endpoints in the generic API lane.
2. Add at least 8 new or upgraded direct tools across missing categories, prioritizing:
- follows graph (`followers`, `following`, relationship checks)
- bookmarks (list/add/remove)
- likes retrieval
- additional user lookup and tweet retrieval variants
- timeline/search enhancements where missing
3. Add full input validation and structured errors for each new tool.
4. Ensure every new tool reports clear capability/tier constraints when unavailable.
5. Add focused unit/integration tests per endpoint path.

Required artifacts to create:

- `roadmap/artifacts/session-04-added-tools-matrix.md`
- `roadmap/artifacts/session-04-endpoint-test-report.md`
- `roadmap/artifacts/session-04-handoff.md`

Definition of done:

- New tools are implemented, tested, and visible in MCP tool list.
- Output schema style is consistent across old/new direct tools.
- Missing endpoint list from Session 01 is materially reduced and quantified.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. New tools added with one-line purpose each.
2. Endpoint categories still missing after this session.
3. What Session 05 will lock down in contracts.
