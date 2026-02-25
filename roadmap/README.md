# X MCP Session Roadmap

This folder contains copy/paste prompts to run Claude Code session by session.

Program goal: turn TuitBot into a best-in-class X MCP platform by fixing all three core concerns:

- Scope: deliver broad, reliable X API coverage.
- Coupling: decouple generic MCP from TuitBot-specific DB/config/workflow.
- Interoperability: provide a predictable "Twitter client as tools" layer that other agents can reuse.

Non-negotiable rule for this roadmap: no backward compatibility constraints. Prefer ideal architecture over legacy preservation.

## Session order

1. `session-01-baseline-and-gap-audit.md`
2. `session-02-target-architecture-and-boundaries.md`
3. `session-03-runtime-decoupling-and-api-profile.md`
4. `session-04-api-v2-coverage-expansion.md`
5. `session-05-contract-hardening-and-tool-manifest.md`
6. `session-06-rate-limits-retries-and-pagination.md`
7. `session-07-workflow-layer-pluginization.md`
8. `session-08-scraper-provider-optional-lane.md`
9. `session-09-conformance-and-performance-gates.md`
10. `session-10-release-docs-and-go-live.md`

## How to use

1. Open one session file.
2. Copy the prompt into a fresh Claude Code session.
3. Let Claude implement fully, run tests, and produce listed artifacts.
4. Do not start the next session until all acceptance criteria pass.
5. Commit after each session with a clear message.

## Required quality bar (every session)

- Rust changes must pass:
  - `cargo fmt --all && cargo fmt --all --check`
  - `RUSTFLAGS="-D warnings" cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Any failed or skipped check must be explained in the session artifact.
- Every session must write a handoff report in `roadmap/artifacts/`.
