# QA/Test Board — TuitBot Coverage & Testing Infrastructure

**Board Lead Inbox:** Test coverage expansion and infrastructure setup  
**Audit Date:** 2026-03-13  
**Total Effort:** ~3–4 weeks  
**Success Metric:** ≥75% code coverage on core modules; comprehensive component tests

---

## Task 3.1: Component Testing Infrastructure

**Status:** NOT STARTED  
**Assigned to:** [QA Lead]  
**Sprint:** 1 (BLOCKING for frontend refactoring)

### Requirements
- [ ] Set up Vitest for Svelte component unit tests
  - [ ] Install: `vitest`, `@sveltejs/vite-plugin-svelte`, `@testing-library/svelte`
  - [ ] Configure `vitest.config.ts` in dashboard root
  - [ ] Add test helpers for fixture data, mocks, stores

- [ ] Set up Playwright for component and E2E tests
  - [ ] Install: `@playwright/test`
  - [ ] Configure component testing mode in `playwright.config.ts`
  - [ ] Create base test fixtures (auth, API mocks, store setup)

- [ ] Create test utilities
  - [ ] Mock stores (composerStore, approvalStore, settingsStore)
  - [ ] Mock API client (fetch intercepts)
  - [ ] Test data factories (sample tweets, approval items, accounts)

- [ ] CI integration
  - [ ] Add `npm run test:unit` (Vitest) to CI
  - [ ] Add `npm run test:component` (Playwright) to CI
  - [ ] Generate coverage reports

### Definition of Done
- [ ] Vitest + Playwright working locally
- [ ] Sample component test written and passing
- [ ] CI pipeline executes tests
- [ ] Documentation for test patterns in `TESTING.md`

### Effort Estimate
- 5–7 days (infrastructure and examples)

---

## Task 3.2: Composer Component Tests

**Scope:** Test components from Task 2.1 (Frontend)  
**Status:** BLOCKED until frontend refactoring  
**Assigned to:** [QA Lead]  
**Sprint:** 2 (after frontend splits components)

### Requirements
Test each new Composer component:

- **ComposerCanvas** (~50 lines of test)
  - [ ] Text input and state updates
  - [ ] Cursor position tracking
  - [ ] Undo/redo functionality
  - [ ] Character count accuracy

- **ComposerToolbar** (~50 lines of test)
  - [ ] Formatting buttons (bold, italic, link, quote)
  - [ ] AI assist trigger
  - [ ] Quick publish button

- **ThreadPreviewRail** (~60 lines of test)
  - [ ] Thread structure visualization
  - [ ] Reorder tweets via drag-drop
  - [ ] Edit individual tweets in thread

- **ComposerInspector** (~40 lines of test)
  - [ ] Scheduling date/time picker
  - [ ] Media upload state
  - [ ] Metadata display (char count, media count)

- **ComposeWorkspace** (~100 lines of test)
  - [ ] Full composition workflow (text → format → schedule → publish)
  - [ ] State synchronization across sub-components
  - [ ] Error handling (validation failures, API errors)

### Definition of Done
- [ ] All components have ≥80% test coverage
- [ ] Tests cover happy path, error cases, edge cases
- [ ] Tests are maintainable and documented
- [ ] No flaky tests (retry logic if needed)

### Effort Estimate
- 5–7 days

---

## Task 3.3: Svelte Stores Unit Tests

**Scope:** Test all dashboard stores  
**Status:** NOT STARTED  
**Assigned to:** [QA Lead]  
**Sprint:** 2

### Stores to Test
- [ ] `composerStore` — text, media, scheduling state
- [ ] `approvalStore` — approval queue, filters, bulk actions
- [ ] `analyticsStore` — metrics, charts, historical data
- [ ] `settingsStore` — user config, credentials, preferences
- [ ] `discoveryStore` — search results, scoring, filters

### Test Requirements
Each store test should verify:
- [ ] Initial state is correct
- [ ] Actions update state deterministically
- [ ] Derived values compute correctly
- [ ] Side effects (API calls) happen correctly
- [ ] Subscriptions work (if using subscribe)

### Example: ComposerStore Test
```typescript
// tests/stores/composerStore.test.ts
describe('composerStore', () => {
  test('initializes with empty text', () => {
    const { text, media } = get(composerStore);
    expect(text).toBe('');
    expect(media).toEqual([]);
  });

  test('updateText changes text state', () => {
    composerStore.updateText('Hello world');
    expect(get(composerStore.text)).toBe('Hello world');
  });

  test('addMedia appends to media list', () => {
    composerStore.addMedia(mockMediaFile);
    expect(get(composerStore.media).length).toBe(1);
  });

  test('reset clears all state', () => {
    composerStore.updateText('Text');
    composerStore.reset();
    expect(get(composerStore.text)).toBe('');
  });
});
```

### Definition of Done
- [ ] All stores have ≥80% coverage
- [ ] Tests are clear and maintainable
- [ ] Tests run in <5 seconds

### Effort Estimate
- 5–7 days

---

## Task 3.4: Route Handler Tests

**Scope:** HTTP route integration tests  
**Status:** NOT STARTED  
**Assigned to:** [QA Lead]  
**Sprint:** 2–3

### Routes to Test
- **Content routes:**
  - [ ] `POST /api/content/compose` — draft creation
  - [ ] `PUT /api/content/draft/:id` — draft updates
  - [ ] `POST /api/content/publish` — publishing workflow
  - [ ] `GET /api/content/drafts` — list drafts with filters

- **Approval routes:**
  - [ ] `GET /api/approval/queue` — list queue items
  - [ ] `POST /api/approval/:id/approve` — approve action
  - [ ] `POST /api/approval/:id/reject` — reject with reason
  - [ ] `PUT /api/approval/:id/edit` — edit queued item

- **Discovery routes:**
  - [ ] `POST /api/discovery/search` — search tweets
  - [ ] `GET /api/discovery/feed` — discovery feed
  - [ ] `POST /api/discovery/score` — score a tweet

- **Analytics routes:**
  - [ ] `GET /api/analytics/metrics` — overview metrics
  - [ ] `GET /api/analytics/trends` — historical trends
  - [ ] `GET /api/analytics/top-tweets` — best performing content

### Test Requirements
Each route test should verify:
- [ ] Happy path returns correct status + data
- [ ] Validation errors return 400/422
- [ ] Auth errors return 401
- [ ] Not found returns 404
- [ ] Concurrent requests handled correctly
- [ ] Database state changes are correct

### Example: Compose Route Test
```rust
// tests/routes/content_test.rs
#[tokio::test]
async fn test_compose_tweet_success() {
  let app = setup_test_app().await;
  let req = ComposeRequest {
    text: "Hello, world!",
    media_ids: vec![],
    scheduled_at: None,
  };
  
  let res = app.post("/api/content/compose")
    .json(&req)
    .send()
    .await;
  
  assert_eq!(res.status(), StatusCode::CREATED);
  let draft: DraftResponse = res.json().await;
  assert_eq!(draft.text, "Hello, world!");
}

#[tokio::test]
async fn test_compose_tweet_too_long() {
  let app = setup_test_app().await;
  let req = ComposeRequest {
    text: "a".repeat(300), // exceeds limit
    media_ids: vec![],
    scheduled_at: None,
  };
  
  let res = app.post("/api/content/compose")
    .json(&req)
    .send()
    .await;
  
  assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
```

### Definition of Done
- [ ] All critical routes have integration tests
- [ ] ≥80% coverage for route handlers
- [ ] Tests validate data correctness
- [ ] Error cases covered

### Effort Estimate
- 5–7 days

---

## Task 3.5: Automation Layer Tests (Post-Refactor)

**Scope:** Core automation module tests (after backend refactoring)  
**Status:** BLOCKED until backend tasks complete  
**Assigned to:** [QA Lead]  
**Sprint:** 3–4 (after Task 1.1, 1.2 complete)

### Requirements
- [ ] **content_loop tests:** 
  - Test generation with various LLM configs
  - Test scheduling logic (slot selection, conflicts)
  - Test publishing (retry, idempotency, error handling)
  - Aim for ≥85% coverage

- [ ] **thread_loop tests:**
  - Test thread composition (topic selection, tweet ordering)
  - Test weekly scheduling
  - Test thread publishing (atomic, all-or-nothing)
  - Aim for ≥85% coverage

- [ ] **watchtower tests:**
  - Test connection state management
  - Test reconnection logic
  - Test provider integration
  - Aim for ≥75% coverage

### Definition of Done
- [ ] All automation tests passing
- [ ] Coverage targets met
- [ ] Tests are maintainable (clear test names, fixtures)
- [ ] No flaky tests

### Effort Estimate
- 5–7 days (after refactoring complete)

---

## Task 3.6: Coverage Reporting Infrastructure

**Status:** NOT STARTED  
**Assigned to:** [QA Lead]  
**Sprint:** 2–3 (can run parallel to other testing tasks)

### Requirements
- [ ] **Backend coverage:**
  - [ ] Install `cargo-tarpaulin`
  - [ ] Generate coverage report: `cargo tarpaulin -p tuitbot-core -p tuitbot-server`
  - [ ] Set minimum threshold (≥75%)
  - [ ] Add to CI: generate coverage badge

- [ ] **Frontend coverage:**
  - [ ] Enable coverage in Vitest config
  - [ ] Generate coverage report: `npm run test:coverage`
  - [ ] Set minimum threshold (≥70%)
  - [ ] Add to CI

- [ ] **CI integration:**
  - [ ] Add coverage step to GitHub Actions
  - [ ] Generate coverage badge (shields.io or similar)
  - [ ] Fail CI if coverage drops below threshold
  - [ ] Upload reports to codecov.io (optional)

- [ ] **Documentation:**
  - [ ] Add coverage badge to README
  - [ ] Document coverage targets in CONTRIBUTING.md

### Definition of Done
- [ ] Coverage reports generating in CI
- [ ] Badge visible in README
- [ ] Threshold enforced (CI fails if coverage drops)

### Effort Estimate
- 3–5 days

---

## Task 3.7: Test Data Factories & Fixtures

**Status:** NOT STARTED  
**Assigned to:** [QA Lead]  
**Sprint:** 1–2 (foundational)

### Requirements
Create reusable test fixtures and factories:

**Rust fixtures** (`crates/tuitbot-core/src/testing/`)
- [ ] `tweet_factory` — create realistic test tweets
- [ ] `account_factory` — create test accounts with various profiles
- [ ] `approval_item_factory` — create queued items
- [ ] `config_fixture` — default test config
- [ ] `mock_x_client` — mock X API responses

**TypeScript fixtures** (`dashboard/src/testing/`)
- [ ] `mockStores` — pre-configured store mocks
- [ ] `mockApi` — API response mocks
- [ ] `fixtures.ts` — sample data (tweets, accounts, etc.)
- [ ] `testHelpers.ts` — render helpers, wait utilities

### Definition of Done
- [ ] Factories are used consistently in tests
- [ ] New test code uses factories (no hard-coded data)
- [ ] Fixtures reduce test setup boilerplate by ≥50%

### Effort Estimate
- 3–5 days

---

## Summary & Success Criteria

| Task | Est. Days | Priority | Sprint | Owner | Blocker |
|------|-----------|----------|--------|-------|---------|
| 3.1 Infrastructure | 5–7 | HIGH | 1 | QA | Yes (frontend) |
| 3.2 Composer Tests | 5–7 | HIGH | 2 | QA | Task 2.1 |
| 3.3 Store Tests | 5–7 | HIGH | 2 | QA | No |
| 3.4 Route Tests | 5–7 | MEDIUM | 2–3 | QA | No |
| 3.5 Automation Tests | 5–7 | MEDIUM | 3–4 | QA | Task 1.1/1.2 |
| 3.6 Coverage Reports | 3–5 | MEDIUM | 2–3 | QA | No |
| 3.7 Test Fixtures | 3–5 | HIGH | 1–2 | QA | No |

**Total Estimated Effort:** ~3–4 weeks  
**Expected Outcomes:**
- Comprehensive test infrastructure (Vitest + Playwright)
- ≥75% code coverage on core modules
- Improved test maintainability (factories, fixtures)
- Foundation for confident refactoring (backend/frontend)
- Coverage reporting in CI (track progress over time)

---

## Testing Best Practices Document

As part of Task 3.1, create `TESTING.md` with:
- Component testing patterns (Svelte 5 runes)
- Store testing examples
- Route testing patterns
- Mock strategies (API, filesystem, time)
- Fixture and factory usage
- Flaky test mitigation strategies
- Performance testing guidelines

