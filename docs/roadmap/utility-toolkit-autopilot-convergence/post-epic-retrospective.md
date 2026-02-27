# Post-Epic Retrospective: Utility Toolkit + Autopilot Convergence

**Date:** 2026-02-26
**Sessions:** 01–08
**Branch:** `feat/mcp_final`

---

## What Was Accomplished

Transformed TuitBot's architecture from autopilot-first to utility-first across 8 sessions:

1. **Toolkit layer** (`core::toolkit/`): Stateless X API utilities (read, write, engage, media) callable with just `&dyn XApiClient`. No DB, no LLM, no MCP transport required.

2. **Workflow layer** (`core::workflow/`): Stateful composites (discover, draft, queue, publish, thread_plan, orchestrate) that combine toolkit with DB/LLM. Reusable by MCP, HTTP server, CLI, and automation.

3. **MCP handler simplification**: Handlers in `tuitbot-mcp` became thin adapters — parameter parsing, toolkit/workflow delegation, and envelope wrapping. Business logic moved to core.

4. **Documentation overhaul**: Architecture docs, operational guides, CLAUDE.md, and README all reflect the three-layer model with dependency rules.

5. **Comprehensive test coverage**: 15 new e2e tests proving layers compose independently, 34 conformance tests, 40 boundary tests, 4 eval harness scenarios. Total workspace: 1399 tests.

6. **Zero regressions**: All 109 MCP tools preserved. Manifest tool counts unchanged. Response envelope contract unchanged.

---

## What Went Well

- **Charter-first approach**: Session 01 produced a thorough charter with gap analysis, architecture decisions, and execution plan. This gave all subsequent sessions clear scope boundaries and prevented scope creep.

- **Layered extraction strategy**: Building bottom-up (Toolkit → Workflow → Consumer rewiring) avoided circular dependencies and allowed each session to produce a green CI state.

- **Test-driven confidence**: Conformance tests, boundary tests, and eval harness provided continuous regression detection throughout the refactor.

- **Manifest sync tooling**: `scripts/check-mcp-manifests.sh` caught drift early and prevented silent tool count changes.

- **Handoff discipline**: Every session produced a handoff document with explicit files-to-read, commands-to-run, and exit criteria. This enabled fresh-context session execution without cumulative context.

---

## What Could Be Improved

- **Session 07 scope redirection**: The execution plan assigned autopilot loop rewiring to Session 07, but the actual session instructions specified docs/manifests/e2e instead. This created an AD-06 enforcement gap that wasn't caught until Session 08 validation.

- **Env-var test isolation**: The `config::tests::env_var_override_approval_mode` race condition was known but not fixed during the epic. Should have been addressed as a drive-by fix when touching adjacent code.

- **File size enforcement**: `workflow/tests.rs` grew to 781 lines (above the 500-line limit). Should have been proactively split when tests were added in Sessions 04–05.

- **`check_policy` → `run_gateway` migration**: The legacy path in `propose_queue.rs` was identified in Session 06 but not fixed. Small scope, should have been included in a later session as a 5-minute fix.

---

## Follow-Up Actions

These items are outside the scope of this epic but should be addressed in subsequent work:

### P1 — Should Do Soon

| Action | Description | Effort |
|--------|-------------|--------|
| **AD-06 full enforcement** | Refactor autopilot loops to call toolkit/workflow directly instead of through `loop_helpers` trait indirection. Remove or simplify `adapters.rs`. Add CI lint script. | 1 session |
| **Unify policy path** | Update `propose_queue.rs` to use `run_gateway` instead of `check_policy`. | 1 hour |
| **Split `workflow/tests.rs`** | Convert to `tests/` module directory with per-step test files. | 1 hour |

### P2 — Nice to Have

| Action | Description | Effort |
|--------|-------------|--------|
| **Fix env-var test flake** | Add `#[serial]` (via `serial_test` crate) or use `temp_env` for `env_var_override_approval_mode*` tests. | 30 minutes |
| **Update `wasm-bindgen`** | Yanked version in Cargo.lock produces warning during `cargo package`. | 15 minutes |
| **Toolkit instrumentation** | Add optional telemetry hooks to toolkit functions for observability. | 1 session |
| **CLI toolkit commands** | Expose toolkit functions as `tuitbot toolkit search`, `tuitbot toolkit post` etc. for ad-hoc use. | 1 session |

### P3 — Future Consideration

| Action | Description | Effort |
|--------|-------------|--------|
| **Layer dependency lint in CI** | Automated check that toolkit doesn't import workflow, workflow doesn't import automation. | 2 hours |
| **Workflow integration tests** | Real DB + mock X API integration tests for composite workflows. | 1 session |
| **HTTP server rewiring** | Point `tuitbot-server` routes at workflow functions instead of reimplementing logic. | 1–2 sessions |

---

## Metrics

| Metric | Before Epic | After Epic | Change |
|--------|-------------|------------|--------|
| Test count | ~1300 | 1399 | +~99 |
| Toolkit functions | 0 (none existed) | 29 (14 read + 5 write + 8 engage + 1 media + scoring) | New layer |
| Workflow functions | 0 (logic in MCP) | 6 modules (discover, draft, queue, publish, thread_plan, orchestrate) | New layer |
| MCP tool count | 109 | 109 | Unchanged |
| Architecture docs | Outdated single page | 3-layer model in 4 docs (architecture, CLAUDE.md, README, operations) | Comprehensive |
| E2E test coverage | None for layers | 15 e2e tests across toolkit + workflow | New |
| Roadmap artifacts | 0 | 20 files (charter, plan, decisions, 8 handoffs, 8 session docs, 3 final docs) | Full trail |

---

## Key Lessons

1. **Bottom-up extraction works**: Start with the lowest layer (stateless utilities), validate independently, then build upward. This avoids the "big bang" refactor risk.

2. **Handoff documents are essential for multi-session work**: Without memory between sessions, explicit handoffs with file paths and commands are the only reliable continuity mechanism.

3. **Test suites as refactoring safety nets**: Conformance and boundary tests caught issues that would have been invisible without them. Invest in test infrastructure before refactoring.

4. **Scope boundaries prevent creep**: Hard "this session does X only" rules kept each session deliverable. Discoveries were logged and deferred, not chased.

5. **Architecture decisions upfront save time**: The 14 ADs in Session 01 eliminated most design debates in subsequent sessions. Code sessions could focus on implementation rather than design.
