# Session 08 Prompt - Scraper Provider Optional Lane

## Copy/paste prompt

You are executing Session 08 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: add a scraper-backed provider lane as an optional read-heavy backend without weakening safety or clarity.

Critical constraints:

- No backward compatibility requirements.
- Scraper lane must be explicit opt-in and clearly labeled as higher-risk.
- Do not silently route API calls to scraper backend.

Required implementation work:

1. Add provider selection framework that supports at least:
- `official_api`
- `scraper`
2. Implement scraper provider interface/stub or concrete adapter path wired into generic tool contracts.
3. Add capability flags showing provider source and confidence/risk metadata.
4. Gate unsafe or unsupported scraper mutations by default.
5. Ensure telemetry records provider type per call.
6. Add tests covering provider selection and guard behavior.

Required artifacts to create:

- `roadmap/artifacts/session-08-provider-selection-spec.md`
- `roadmap/artifacts/session-08-scraper-risk-guardrails.md`
- `roadmap/artifacts/session-08-handoff.md`

Definition of done:

- Providers are pluggable without core contract changes.
- Scraper lane is visible, explicit, and policy-constrained.
- Generic MCP consumers can choose provider predictably.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Provider architecture after implementation.
2. What is enabled/disabled by default in scraper mode.
3. What Session 09 must prove via conformance tests.
