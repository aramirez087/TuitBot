# TuitBot Roadmap — Q1 2026 Structural Improvements

**Last Updated:** 2026-03-13  
**Status:** Initial prioritization based on codebase health audit  
**Next Review:** After board lead intake and sprint planning

## Executive Summary

TuitBot has strong feature delivery and recent enterprise API coverage (v0.1.22). However, structural debt is accumulating:
- **42 files** violate size constraints (10 Rust, 5 Svelte at 2–4.8x limit)
- **Test coverage gaps** in composer, stores, and route handlers
- **CI inefficiencies** (3x dashboard rebuilds per release)
- **UX debt** in monolithic components and missing navigation patterns

This roadmap prioritizes **structural refactoring, test hardening, and CI optimization** to improve maintainability and developer velocity.

---

## Priority Matrix

| Severity | Category | Est. Effort | Impact | Board Owner |
|----------|----------|------------|--------|-------------|
| 🔴 HIGH | Rust file modularization | 3–4 sprints | Maintainability, team velocity | Backend/Core |
| 🔴 HIGH | Svelte component decomposition | 2–3 sprints | UX clarity, test coverage | Frontend |
| 🟠 MEDIUM | Test coverage hardening | 2–3 sprints | Release confidence, stability | QA/Test |
| 🟠 MEDIUM | CI pipeline optimization | 1–2 sprints | Release velocity, cost | DevOps/CI |
| 🟡 LOW | UX polish (nav, pagination) | 1–2 sprints | User experience | Frontend |

---

## Workstream 1: Core Refactoring (Backend)

**Owner:** Backend/Core Board Lead  
**Duration:** 4–6 weeks  
**Goal:** Bring all Rust files into compliance with 500-line constraint.

### 1.1 Automation Layer Modularization
- [ ] `content_loop.rs` (1259 lines) → `automation/content_loop/{mod.rs, generator.rs, scheduler.rs, publisher.rs}`
- [ ] `thread_loop.rs` (1137 lines) → `automation/thread_loop/{mod.rs, generator.rs, planner.rs}`
- [ ] `watchtower.rs` (857 lines) → Review and modularize connection management
- **Priority:** HIGH (most complex, highest test impact)

### 1.2 MCP Server Handler Split
- [ ] `server/admin.rs` (1175 lines) → `server/admin/{mod.rs, tools.rs, handlers.rs}`
- [ ] `server/write.rs` (1086 lines) → `server/write/{mod.rs, tools.rs, handlers.rs}`
- [ ] `server/api_readonly.rs` (945 lines) → Review split if needed
- **Priority:** HIGH (external API surface, policy implications)

### 1.3 Test Suite Organization
- [ ] `api_tests.rs` (2398 lines) → `tests/{compose.rs, approval.rs, analytics.rs, content.rs, discovery.rs}`
- [ ] `config/tests.rs` (1384 lines) → `config/tests/{defaults.rs, validation.rs, migrations.rs}`
- **Priority:** MEDIUM (non-blocking but improves maintainability)

### 1.4 Large Modules Review
- [ ] `spec/endpoints.rs` (1837 lines) — Consider generated code or logical split
- [ ] `safety/qa.rs` (968 lines) — Extract QA checks into submodules
- [ ] `startup.rs` (927 lines) — Move init/config/db setup to startup/mod.rs
- **Priority:** MEDIUM (lower impact, can be staged)

---

## Workstream 2: Frontend Component Decomposition (Frontend)

**Owner:** Frontend Board Lead  
**Duration:** 3–4 weeks  
**Goal:** Bring all Svelte components into compliance with 400-line constraint.

### 2.1 Composer Workspace Refactor
**Current:** `ComposeWorkspace.svelte` (892 lines) — monolithic, hard to test  
**Target:** Extract into focused, reusable components

```
dashboard/src/lib/components/composer/
├── ComposeWorkspace.svelte (orchestrator, ~150 lines)
├── ComposerCanvas.svelte (editor surface, ~200 lines)
├── ComposerToolbar.svelte (formatting/actions, ~180 lines)
├── ThreadPreviewRail.svelte (thread visualization, ~150 lines)
└── ComposerInspector.svelte (metadata/settings, ~150 lines)
```
- **Priority:** HIGH (core user workflow, high visibility)

### 2.2 Settings Panel Decomposition
- [ ] `AccountsSection.svelte` (731 lines) → `AccountList + AddAccountModal + ProfileEditForm`
- [ ] `CredentialCard.svelte` (713 lines) → Separate forms for X API, Google Drive, LLM
- **Priority:** HIGH (settings is critical path for onboarding)

### 2.3 MCP Policy Section Refactor
**Current:** `PolicySection.svelte` (887 lines) — combines rules, templates, evaluation  
**Target:** `PolicyRulesSection + PolicyTemplatesSection + PolicyEvaluationPanel`
- **Priority:** HIGH (complex domain, needs clarity)

### 2.4 Approval Queue Optimization
- [ ] `ApprovalCard.svelte` (722 lines) → `ApprovalCardHeader + ApprovalCardActions + ApprovalHistory + RejectionDialog`
- [ ] `approval/+page.svelte` (443 lines) → Add pagination/virtualization for large queues
- **Priority:** MEDIUM (performance impact)

### 2.5 Discovery Interface Improvements
- [ ] `discovery/+page.svelte` (557 lines) → Add inline tweet preview pane, reduce navigation
- **Priority:** LOW (nice-to-have improvement)

---

## Workstream 3: Test Coverage Hardening (QA/Test)

**Owner:** QA/Test Board Lead  
**Duration:** 3–4 weeks  
**Goal:** Close visibility gaps and enable regression testing.

### 3.1 Component Test Coverage
- [ ] Add Vitest/Playwright tests for Composer components (ComposeWorkspace, Canvas, Toolbar, etc.)
- [ ] Add Svelte store tests (analytics, approval, settings stores)
- [ ] Aim for **≥80% coverage** on stores and major components
- **Priority:** HIGH (blocks confident refactoring)

### 3.2 Route Handler Coverage
- [ ] Add route-level integration tests for content routes (compose, draft, publish)
- [ ] Add route tests for approval workflow (list, approve, reject, edit)
- [ ] Add discovery route tests
- **Priority:** MEDIUM (improves stability)

### 3.3 Automation Layer Tests
- [ ] Expand content_loop tests post-refactor
- [ ] Add thread_loop comprehensive tests
- [ ] Add watchtower connection tests
- **Priority:** MEDIUM (post-refactor validation)

### 3.4 Coverage Reporting
- [ ] Integrate `cargo-tarpaulin` into CI pipeline
- [ ] Set minimum coverage threshold (e.g., ≥75% for core)
- [ ] Add coverage badges to README
- **Priority:** LOW (observability, not blocking)

---

## Workstream 4: CI/Release Pipeline Optimization (DevOps/CI)

**Owner:** DevOps/CI Board Lead  
**Duration:** 2–3 weeks  
**Goal:** Reduce release time and improve automation.

### 4.1 Dashboard Build Caching
- [ ] Unify dashboard build step (currently 3+ separate builds in release.yml)
- [ ] Cache SvelteKit output artifacts between jobs
- [ ] **Expected savings:** 15–20 minutes per release
- **Priority:** HIGH (immediate ROI)

### 4.2 Coverage & Performance Reporting
- [ ] Add `cargo-tarpaulin` step to CI (test coverage reporting)
- [ ] Add cargo-criterion benchmark comparisons
- [ ] Add SvelteKit build output size tracking
- **Priority:** MEDIUM (observability)

### 4.3 Dependency Automation
- [ ] Integrate Dependabot or Renovate for automated dependency updates
- [ ] Set up automated PR creation for security patches
- [ ] Configure scheduled dependency update checks
- **Priority:** MEDIUM (operational efficiency)

### 4.4 Release Artifact Integrity
- [ ] Add GPG signature generation for release artifacts
- [ ] Document signature verification in release notes
- **Priority:** LOW (nice-to-have, supply chain security)

---

## Implementation Timeline

### Sprint 1 (Week of 2026-03-17)
- **Backend:** Start automation layer refactoring (content_loop, thread_loop)
- **Frontend:** Begin Composer workspace decomposition
- **DevOps:** Unify dashboard build steps in CI
- **QA:** Audit existing test coverage, plan component test strategy

### Sprint 2–3 (Weeks of 2026-03-24 & 2026-03-31)
- **Backend:** Complete MCP server handler splits; start large module review
- **Frontend:** Complete Composer, AccountsSection, CredentialCard refactors
- **QA:** Implement component tests, store tests
- **DevOps:** Integrate coverage reporting, set up Dependabot

### Sprint 4+ (Post-April 1)
- **Backend:** Complete all refactoring; update tests
- **Frontend:** Complete remaining UX polish (discovery, pagination)
- **QA:** Expand integration test coverage
- **DevOps:** Monitor and tune CI pipeline

---

## Success Metrics

| Metric | Target | Owner |
|--------|--------|-------|
| All files ≤ 500 lines (Rust) / 400 lines (Svelte) | 100% compliance | Backend/Frontend |
| Test coverage (core modules) | ≥75% | QA |
| Release cycle time | -20 min (via dashboard caching) | DevOps |
| Known file constraint violations | 0 | All |
| CI execution time | Trending down | DevOps |

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Refactoring introduces regressions** | Increase test coverage first; use feature flags for gradual rollout |
| **Estimation underrun (especially frontend)** | Pair refactoring with component test writing; aim for test-driven splits |
| **Dependency updates break builds** | Run Dependabot on dev branch; pin major version updates for review |
| **CI caching complexity** | Start with dashboard only; validate thoroughly before expanding |

---

## Not in Scope (Yet)

- Desktop (Tauri) app refactoring (separate effort)
- Major feature work (focus on structural debt only)
- Database schema migrations (stable, low priority)
- Performance tuning (benchmark infrastructure first, then optimize)

---

## Questions for Board Leads

1. **Backend:** Can you commit to modularizing `content_loop` and `thread_loop` by end of Sprint 2? These are on the critical path for test coverage.
2. **Frontend:** What's your confidence in decomposing Composer into 5 smaller components without breaking existing user workflows? Do you need staging environment support?
3. **QA:** Can you have component test infrastructure (Vitest + Playwright setup) in place by end of Sprint 1?
4. **DevOps:** Any concerns with unifying dashboard builds or adding GPG signing to release workflow?

---

## Related Docs

- `CLAUDE.md` — Constraint definitions and build/test commands
- `CHANGELOG.md` — Recent changes for context
- `README.md` — Feature overview

