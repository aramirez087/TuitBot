# TuitBot Dashboard — Test Infrastructure

## Stack

| Layer | Tool |
|---|---|
| Unit / component | [Vitest](https://vitest.dev/) + [@testing-library/svelte](https://testing-library.com/docs/svelte-testing-library/intro/) |
| E2E | [Playwright](https://playwright.dev/) |
| Test environment | jsdom |

## Structure

```
tests/
  setup.ts              — Global setup: @testing-library/jest-dom, SvelteKit mocks
  unit/                 — Vitest unit tests for stores, utils, logic
    infra.test.ts       — Smoke tests: verifies infra itself works
  e2e/                  — Playwright end-to-end tests (requires dev server)
  helpers/
    fixtures.ts         — Shared test data factories (ApprovalItem, TargetAccount, etc.)
    mockStores.ts       — Pre-configured Svelte store mocks for all major stores
    mockApi.ts          — Mock API client with vi.fn() stubs + fixture defaults
    testHelpers.ts      — Render helpers, assertion utilities, async helpers
```

## Running Tests

```bash
# Unit tests (watch mode)
npm run test:unit

# Unit tests (single run, for CI)
npm run test:unit -- --run

# E2E tests (requires `npm run dev` in another terminal)
npm run test:e2e

# All tests
npm test
```

## Writing Unit Tests

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { resetAll, renderWithStores, assertStoreValue } from '../helpers/testHelpers';
import { mockApprovalStore } from '../helpers/mockStores';
import { mockApi } from '../helpers/mockApi';
import { fixtures } from '../helpers/fixtures';
import MyComponent from '../../src/lib/components/MyComponent.svelte';

beforeEach(() => resetAll());

describe('MyComponent', () => {
  it('renders approval items', () => {
    const { getByText } = renderWithStores(MyComponent);
    expect(getByText('pending')).toBeTruthy();
  });
});
```

## Simulating API Errors

```typescript
mockApi.approval.list.mockRejectedValueOnce(apiError('Network failure', 503));
```

## Factory Usage

```typescript
// Create one item with custom fields
const item = fixtures.approvalItem({ status: 'approved', score: 0.99 });

// Use the pre-built array
const items = fixtures.approvalItems; // 4 items in various states
```
