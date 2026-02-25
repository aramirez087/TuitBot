# Session 06 Prompt - Rate Limits, Retries, and Pagination

## Copy/paste prompt

You are executing Session 06 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: make the MCP operationally reliable under real X API conditions.

Critical constraints:

- No backward compatibility requirements.
- Reliability beats convenience abstractions.

Required implementation work:

1. Centralize API rate-limit handling into one reusable policy component.
2. Implement adaptive retry strategy for retryable failures (429/5xx/network), including jitter.
3. Normalize pagination behavior across all list/search tools.
4. Surface rate-limit reset and retry hints consistently in tool responses.
5. Add idempotency safeguards for mutation tools where practical.
6. Expand tests with simulated 429, transient 500, and timeout scenarios.

Required artifacts to create:

- `roadmap/artifacts/session-06-reliability-design.md`
- `roadmap/artifacts/session-06-rate-limit-test-results.md`
- `roadmap/artifacts/session-06-pagination-contract.md`
- `roadmap/artifacts/session-06-handoff.md`

Definition of done:

- Retry policy is centralized and test-covered.
- Pagination output shape is consistent across endpoints.
- Agents receive machine-readable retry guidance.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Reliability improvements implemented.
2. New operational failure modes now covered.
3. What Session 07 will separate into workflow plugin lane.
