# Backend/Core Board — TuitBot Structural Refactoring Tasks

**Board Lead Inbox:** Structural debt consolidation for TuitBot  
**Audit Date:** 2026-03-13  
**Total Effort:** ~4–6 weeks (staged across 3–4 sprints)  
**Success Metric:** All Rust files ≤ 500 lines; test coverage ≥75% on core modules

---

## Task 1.1: Modularize `content_loop.rs`

**File:** `crates/tuitbot-core/src/automation/content_loop.rs`  
**Current Size:** 1259 lines (2.5x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 1

### Requirements
- [ ] Extract `content_loop.rs` into `content_loop/mod.rs` + submodules:
  - `generator.rs` — LLM content generation logic (~300 lines)
  - `scheduler.rs` — scheduling and slot management (~250 lines)
  - `publisher.rs` — publication workflow (~350 lines)
  - `mod.rs` — orchestration and tests (~150 lines)
- [ ] Maintain all existing public API surface
- [ ] All tests pass: `cargo test -p tuitbot-core automation::content_loop`
- [ ] Lint: `cargo clippy --workspace -- -D warnings`

### Definition of Done
- [ ] New module structure matches layout in ROADMAP.md
- [ ] No new dependencies introduced
- [ ] Test coverage unchanged or improved
- [ ] Code review approved by core team
- [ ] Documentation updated in comments

### Risks
- Content generation logic is complex; extraction may reveal hidden state dependencies.
- Publishing workflow touches database and X API; ensure idempotency preserved.

---

## Task 1.2: Modularize `thread_loop.rs`

**File:** `crates/tuitbot-core/src/automation/thread_loop.rs`  
**Current Size:** 1137 lines (2.3x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 1

### Requirements
- [ ] Extract into `thread_loop/mod.rs` + submodules:
  - `generator.rs` — thread composition and planning (~320 lines)
  - `planner.rs` — thread strategy and topic selection (~280 lines)
  - `mod.rs` — orchestration (~200 lines)
- [ ] All tests pass: `cargo test -p tuitbot-core automation::thread_loop`
- [ ] Maintain idempotency for scheduled threads

### Definition of Done
- [ ] New module structure in place
- [ ] All integration tests passing
- [ ] Clippy clean
- [ ] Comments explain module responsibilities

### Notes
- This module schedules weekly threads; ensure no timing regressions during refactor.

---

## Task 1.3: Split MCP Server Admin Handlers

**File:** `crates/tuitbot-mcp/src/server/admin.rs`  
**Current Size:** 1175 lines (2.35x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 2

### Requirements
- [ ] Extract into `server/admin/mod.rs` + submodules:
  - `tools.rs` — tool implementations (~450 lines)
  - `handlers.rs` — MCP handler routing (~300 lines)
  - `mod.rs` — public API (~150 lines)
- [ ] All admin tools remain functional: `cargo test -p tuitbot-mcp admin`
- [ ] Policy enforcement unchanged
- [ ] Tool manifests auto-generate correctly

### Definition of Done
- [ ] Module split complete
- [ ] All conformance tests pass: `cargo test -p tuitbot-mcp conformance`
- [ ] Golden snapshot test passes: `cargo test -p tuitbot-mcp golden_snapshot`

### Notes
- This impacts the MCP interface; coordinate with frontend/testing teams for manifest validation.

---

## Task 1.4: Split MCP Server Write Handlers

**File:** `crates/tuitbot-mcp/src/server/write.rs`  
**Current Size:** 1086 lines (2.17x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 2

### Requirements
- [ ] Extract into `server/write/mod.rs` + submodules matching admin pattern:
  - `tools.rs` — write tool implementations
  - `handlers.rs` — MCP handler routing
  - `mod.rs` — orchestration
- [ ] All write tools remain functional and policy-gated
- [ ] Conformance tests pass

### Definition of Done
- [ ] Module structure matches admin refactor pattern
- [ ] All policy gates intact
- [ ] Tool telemetry still captured

---

## Task 1.5: Organize Test Suite (api_tests.rs)

**File:** `crates/tuitbot-server/tests/api_tests.rs`  
**Current Size:** 2398 lines (4.8x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 2–3

### Requirements
- [ ] Create `tests/` directory structure:
  ```
  tests/
  ├── compose_tests.rs (draft, scheduling, publishing)
  ├── approval_tests.rs (queue, approval workflow)
  ├── analytics_tests.rs (metrics, stats, reports)
  ├── content_tests.rs (content types, validation)
  ├── discovery_tests.rs (search, scoring)
  ├── auth_tests.rs (auth, sessions, tokens)
  └── mod.rs (shared fixtures)
  ```
- [ ] Each file ≤ 500 lines
- [ ] All tests passing: `cargo test --test "*"`
- [ ] Shared fixtures in `mod.rs` for DRY setup

### Definition of Done
- [ ] Test organization complete
- [ ] All tests still passing
- [ ] No test logic changes (only reorganization)
- [ ] Fixtures documented

---

## Task 1.6: Review & Modularize Large Modules

**Scope:** `spec/endpoints.rs` (1837), `safety/qa.rs` (968), `startup.rs` (927)  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 3–4

### Requirements
- [ ] `spec/endpoints.rs` (1837 lines):
  - Evaluate if endpoint definitions should be generated or split
  - If splitting: create `spec/endpoints/{mod.rs, workflow.rs, admin.rs, readonly.rs}`
  - Ensure tool manifest generation still works
  
- [ ] `safety/qa.rs` (968 lines):
  - Extract QA checks: `safety/qa/{mod.rs, content_qa.rs, policy_qa.rs, spam_qa.rs}`
  - Tests for each check type
  
- [ ] `startup.rs` (927 lines):
  - Extract: `startup/{mod.rs, init.rs, config_load.rs, db_setup.rs, validation.rs}`
  - Ensure startup sequence unchanged

### Definition of Done
- [ ] All files ≤ 500 lines
- [ ] All tests passing
- [ ] Startup sequence verified end-to-end
- [ ] No behavioral changes

---

## Task 1.7: Test Suite Organization (config/tests.rs)

**File:** `crates/tuitbot-core/src/config/tests.rs`  
**Current Size:** 1384 lines  
**Status:** NOT STARTED  
**Assigned to:** [Backend Lead]  
**Sprint:** 3

### Requirements
- [ ] Organize into `config/tests/{mod.rs, defaults.rs, validation.rs, migrations.rs, serde.rs}`
- [ ] Each file ≤ 500 lines
- [ ] All tests passing

### Definition of Done
- [ ] Test organization matches pattern from Task 1.5
- [ ] All config tests still passing

---

## Summary & Success Criteria

| Task | Est. Days | Priority | Sprint | Owner |
|------|-----------|----------|--------|-------|
| 1.1 content_loop | 5–7 | HIGH | 1 | Backend |
| 1.2 thread_loop | 5–7 | HIGH | 1 | Backend |
| 1.3 admin.rs | 4–5 | HIGH | 2 | Backend |
| 1.4 write.rs | 4–5 | HIGH | 2 | Backend |
| 1.5 api_tests | 5–7 | MEDIUM | 2–3 | Backend |
| 1.6 large modules | 7–10 | MEDIUM | 3–4 | Backend |
| 1.7 config/tests | 3–5 | MEDIUM | 3 | Backend |

**Total Estimated Effort:** ~4–6 weeks  
**Expected Outcomes:**
- All Rust files in compliance with 500-line constraint
- Improved test maintainability
- Clearer module boundaries for future contributors
- Faster code reviews (smaller, focused changes)

