# Session 02 Prompt - Target Architecture and Boundaries

## Copy/paste prompt

You are executing Session 02 in `/Users/aramirez/Code/ReplyGuy`.  
Mission: define and implement the first production slice of a decoupled MCP architecture.

Critical constraints:

- No backward compatibility requirements.
- Prefer clean boundaries over incremental patching.
- Keep changes compile-safe and test-covered.

Architecture target:

1. `contract layer` - tool schemas, envelope, error taxonomy, capability descriptors.
2. `kernel layer` - tool dispatch and provider-agnostic orchestration.
3. `provider layer` - concrete backends (official X API first, scraper later).
4. `workflow layer` - TuitBot growth features as optional extension, not baseline MCP.

Required implementation work:

1. Create a concise ADR:
- `roadmap/artifacts/session-02-adr-decoupled-mcp.md`
2. Introduce concrete Rust module boundaries for the target layers.
3. Move shared response envelope and error mapping out of mixed tool modules into the contract boundary.
4. Introduce a provider trait that can back generic tools without direct TuitBot DB assumptions.
5. Refactor at least three direct X read tools to run through the new kernel + provider boundary.
6. Add/update tests proving these tools work with a mocked provider implementation.

Required artifacts to create:

- `roadmap/artifacts/session-02-adr-decoupled-mcp.md`
- `roadmap/artifacts/session-02-boundary-map.md`
- `roadmap/artifacts/session-02-refactor-notes.md`
- `roadmap/artifacts/session-02-handoff.md`

Definition of done:

- Direct X read tools no longer depend on workflow-only services.
- Contract types are reusable by non-TuitBot consumers.
- Test suite includes provider-mock tests for refactored tools.
- Handoff names exact modules moved and why.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Final output format:

1. Files/modules added.
2. Files/modules deleted or replaced.
3. What Session 03 must decouple next.
