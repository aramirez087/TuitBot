# Session 05 Prompt - Contract Hardening and Tool Manifest

## Copy/paste prompt

You are executing Session 05 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: make this MCP truly interoperable with a strict, machine-verifiable tool contract.

Critical constraints:

- No backward compatibility requirements.
- Contract stability and predictability are primary goals.

Required implementation work:

1. Formalize one canonical response contract for all tool calls.
2. Make error taxonomy exhaustive and consistent across tool families.
3. Generate or maintain a machine-readable tool manifest that includes:
- tool name
- category
- input schema
- output schema
- auth requirements
- provider availability
- mutation/read classification
4. Add schema validation tests that fail CI on drift.
5. Ensure direct API tools do not leak workflow-specific fields unless explicitly namespaced.

Required artifacts to create:

- `roadmap/artifacts/session-05-tool-manifest.json`
- `roadmap/artifacts/session-05-contract-spec.md`
- `roadmap/artifacts/session-05-schema-validation-report.md`
- `roadmap/artifacts/session-05-handoff.md`

Definition of done:

- Contract is documented and enforced by tests.
- Manifest can be consumed by external agent runtimes without code inspection.
- Tool schemas are deterministic and easy to validate.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Contract decisions made.
2. Breaking cleanup intentionally performed.
3. What Session 06 will harden for reliability.
