# Session 09 Prompt - Conformance and Performance Gates

## Copy/paste prompt

You are executing Session 09 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: prove this MCP is best-in-class through enforceable conformance and performance gates.

Critical constraints:

- No backward compatibility requirements.
- Claims without automated proof are unacceptable.

Required implementation work:

1. Build provider-agnostic conformance tests for direct tool contracts.
2. Add golden fixture validation for input/output schemas.
3. Add evaluation scenarios for:
- successful direct read flow
- mutation flow with policy checks
- rate-limited error behavior
- provider switching behavior
4. Add benchmark or latency checks for core tool families.
5. Wire quality gates into CI so failures block merges.

Required artifacts to create:

- `roadmap/artifacts/session-09-conformance-results.md`
- `roadmap/artifacts/session-09-schema-golden-report.md`
- `roadmap/artifacts/session-09-latency-report.md`
- `roadmap/artifacts/session-09-handoff.md`

Definition of done:

- Conformance tests run automatically and fail on contract drift.
- At least one measurable latency/quality threshold is enforced in CI.
- Evidence files are generated and readable by non-authors.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Quality gates now enforced.
2. Current pass/fail state and thresholds.
3. What Session 10 must finalize for release.
