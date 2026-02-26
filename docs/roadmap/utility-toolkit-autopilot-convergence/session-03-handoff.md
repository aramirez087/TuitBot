# Session 03 Handoff: Utility MCP Profiles

**Date:** 2026-02-26
**Session:** 03 of 08
**Branch:** `feat/mcp_final`

---

## Completed Work

1. **Added `UtilityReadonly` and `UtilityWrite` profile variants** to:
   - `state.rs` — `Profile` enum, `Display`, `FromStr`, and all tests
   - `tools/manifest.rs` — manifest `Profile` enum, `From` impl, profile constant arrays
   - `spec/endpoints.rs` — profile shorthands for spec-generated tools
   - `conformance_tests/coverage.rs` — `profile_str` match arm

2. **Created utility profile server modules**:
   - `server/toolkit_response.rs` — shared `ToolkitError` → `CallToolResult` mapping
   - `server/utility_readonly.rs` — `UtilityReadonlyMcpServer` (11 direct tools, 15 total with spec)
   - `server/utility_write.rs` — `UtilityWriteMcpServer` (30+ direct tools, 67 total with spec)

3. **Wired utility profiles into routing**:
   - `server/mod.rs` — module declarations and re-exports
   - `lib.rs` — `run_server()` dispatch, `run_utility_readonly_server()`, `run_utility_write_server()`
   - Both utility servers reuse `init_readonly_state()` (X client only, no DB)
   - `commands/mcp.rs` — updated doc comment

4. **Updated manifest metadata**:
   - Write/engage/media tools with utility profiles: `Lane::Shared`, `requires_db: false`
   - `spec/generator.rs` — utility-aware lane/db assignment for generated tools
   - Regenerated snapshot: `roadmap/artifacts/session-06-tool-manifest.json`

5. **Added 7 boundary tests** enforcing profile isolation:
   - No workflow tools in utility profiles
   - No DB/LLM requirements in utility profiles
   - No mutations in utility-readonly
   - Superset relationship verified
   - Tool count sanity checks

6. **Generated manifest JSON files**:
   - `docs/generated/mcp-manifest-utility-readonly.json` (15 tools)
   - `docs/generated/mcp-manifest-utility-write.json` (67 tools)

---

## Concrete Decisions Made

| Decision | Summary |
|----------|---------|
| State type | Both utility profiles use `SharedReadonlyState` — no DB pool, no LLM provider |
| Lane assignment | Mutation tools available in utility profiles use `Lane::Shared`, not `Lane::Workflow` |
| requires_db | False for all tools in utility profiles — DB is a workflow audit concern |
| Error handling | `toolkit_response.rs` maps `ToolkitError` directly to error envelope — no audit guard |
| Superset invariant | Every tool in utility-readonly is also in utility-write (tested) |
| quote_tweet media | Utility server does not pass media_ids to `toolkit::write::quote_tweet` (API doesn't support it) |
| Spec generator | Utility-aware: checks profiles for utility membership, adjusts lane/db accordingly |

---

## Open Issues

1. **No policy gate on utility-write mutations**: By design — utility profiles are raw toolkit calls. Session 04 builds a unified policy gateway that could optionally be applied to utility mutations.

2. **upload_media not in utility-write server**: The utility-write manifest includes `x_upload_media` from the curated entries, but the server implementation doesn't include a media upload handler. The spec-generated media endpoints may cover this, or it can be added in a follow-up.

3. **Retry behavior**: Utility profiles call `XApiClient` directly with no retry wrapper. The toolkit layer is intentionally stateless. Retry is a workflow/orchestration concern.

---

## Session 04 Inputs

### Files to Read First

1. **`docs/roadmap/utility-toolkit-autopilot-convergence/session-03-handoff.md`** — This file
2. **`docs/roadmap/utility-toolkit-autopilot-convergence/charter.md`** — Section 4 (policy gateway)
3. **`docs/roadmap/utility-toolkit-autopilot-convergence/architecture-decisions.md`** — AD-03, AD-04
4. **`crates/tuitbot-core/src/mcp_policy/mod.rs`** — Current policy evaluator
5. **`crates/tuitbot-core/src/mcp_policy/evaluator.rs`** — Policy evaluation logic
6. **`crates/tuitbot-core/src/storage/rate_limits.rs`** — Rate limit storage
7. **`crates/tuitbot-core/src/storage/mutation_audit.rs`** — Mutation audit records
8. **`crates/tuitbot-mcp/src/tools/workflow/policy_gate.rs`** — Current policy gate
9. **`crates/tuitbot-mcp/src/tools/idempotency.rs`** — Idempotency store
10. **`crates/tuitbot-mcp/src/tools/workflow/x_actions/write.rs`** — Current mutation path
11. **`crates/tuitbot-mcp/src/tools/workflow/x_actions/engage.rs`** — Current engage path

### Commands to Run Before Starting

```bash
# Verify baseline is green
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings

# Record baseline test counts
cargo test --workspace 2>&1 | grep "test result"
```

### Session 04 Deliverables

1. Single policy gateway interface in core (block rules, rate limits, idempotency, audit)
2. All mutation paths routed through the gateway
3. Duplicate policy checks removed
4. Tests for allowed, blocked, and rate-limited scenarios
5. `docs/roadmap/utility-toolkit-autopilot-convergence/session-04-policy-gateway.md`
6. `docs/roadmap/utility-toolkit-autopilot-convergence/session-04-handoff.md`

### Session 04 Exit Criteria

- Every mutation path uses the same gateway
- Duplicate policy code paths are removed
- Session 05 inputs are explicit in the handoff

---

## Artifact Inventory

| File | Status |
|------|--------|
| `crates/tuitbot-mcp/src/state.rs` | Modified (2 new Profile variants) |
| `crates/tuitbot-mcp/src/server/mod.rs` | Modified (3 new module declarations) |
| `crates/tuitbot-mcp/src/server/toolkit_response.rs` | Created |
| `crates/tuitbot-mcp/src/server/utility_readonly.rs` | Created |
| `crates/tuitbot-mcp/src/server/utility_write.rs` | Created |
| `crates/tuitbot-mcp/src/lib.rs` | Modified (2 new server runners, 6-profile dispatch) |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Modified (new profile constants) |
| `crates/tuitbot-mcp/src/spec/generator.rs` | Modified (utility-aware lane/db) |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Modified (profile constants, lane changes, boundary tests) |
| `crates/tuitbot-mcp/src/tools/conformance_tests/coverage.rs` | Modified (new profile_str arms) |
| `crates/tuitbot-cli/src/commands/mcp.rs` | Modified (doc comment) |
| `docs/generated/mcp-manifest-utility-readonly.json` | Created |
| `docs/generated/mcp-manifest-utility-write.json` | Created |
| `docs/roadmap/.../session-03-profiles.md` | Created |
| `docs/roadmap/.../session-03-handoff.md` | Created (this file) |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerated |
