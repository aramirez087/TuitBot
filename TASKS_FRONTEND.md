# Frontend Board — TuitBot Component Decomposition Tasks

**Board Lead Inbox:** SvelteKit component refactoring and UX polish  
**Audit Date:** 2026-03-13  
**Total Effort:** ~3–4 weeks (staged across 2–3 sprints)  
**Success Metric:** All Svelte components ≤ 400 lines; improved testability and UX

---

## Task 2.1: Decompose Composer Workspace

**File:** `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`  
**Current Size:** 892 lines (2.2x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 1

### Requirements
Refactor monolithic composer into focused, reusable components:

```
dashboard/src/lib/components/composer/
├── ComposeWorkspace.svelte (orchestrator, ~150 lines)
│   ├── ComposerCanvas.svelte (editor surface, ~200 lines)
│   ├── ComposerToolbar.svelte (formatting/actions, ~180 lines)
│   ├── ThreadPreviewRail.svelte (thread visualization, ~150 lines)
│   └── ComposerInspector.svelte (metadata/settings, ~150 lines)
```

### Decomposition Plan

**ComposerCanvas** (~200 lines)
- [ ] Extract editor surface and text input logic
- [ ] Handle text state, selection, undo/redo
- [ ] Expose `text`, `cursorPos` as props
- [ ] Emit `update`, `format`, `insert` events

**ComposerToolbar** (~180 lines)
- [ ] Extract formatting buttons: bold, italic, link, quote, emoji
- [ ] Extract quick actions: AI assist, draft, schedule, publish
- [ ] Receive `text` as prop; emit `format`, `action` events

**ThreadPreviewRail** (~150 lines)
- [ ] Extract thread visualization (preview of full thread)
- [ ] Show thread structure with reply chain
- [ ] Allow reordering/edit of tweets in thread
- [ ] Receive `thread` as prop; emit `reorder`, `edit` events

**ComposerInspector** (~150 lines)
- [ ] Extract metadata panel: character count, media, scheduling
- [ ] Show URL preview, media uploads
- [ ] Scheduling UI (date/time picker)
- [ ] Receive content metadata; emit `schedule`, `mediaUpload` events

**ComposeWorkspace** (~150 lines)
- [ ] Orchestrator that wires sub-components together
- [ ] Manage shared state (`text`, `media`, `scheduling`)
- [ ] Route events from children to stores/API
- [ ] Minimal direct DOM manipulation

### Definition of Done
- [ ] All components ≤ 400 lines
- [ ] Type safety: full TypeScript with `$props()`
- [ ] No prop drilling; use stores for shared state (composerStore)
- [ ] All existing features work identically
- [ ] Svelte 5 runes used throughout (`$props()`, `$state()`, `$derived()`, `$effect()`)
- [ ] Components are independently testable

### Testing
- [ ] Playwright component tests for each sub-component
- [ ] Focus on: text input, toolbar actions, thread preview interaction
- [ ] E2E test for full compose workflow

### Risks & Mitigations
- **Risk:** Breaking existing user workflows during split  
  **Mitigation:** Use feature flag; test extensively on dev environment before release
- **Risk:** Prop/event complexity across components  
  **Mitigation:** Document event contracts; use Svelte stores for complex state

---

## Task 2.2: Refactor Settings — Accounts & Credentials

**Files:**  
- `dashboard/src/routes/(app)/settings/AccountsSection.svelte` (731 lines)  
- `dashboard/src/routes/(app)/settings/CredentialCard.svelte` (713 lines)

**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 1–2

### Task 2.2a: AccountsSection Decomposition

**Current Size:** 731 lines (1.8x limit)  
**Target:** Split into:
- `AccountList.svelte` (~180 lines) — account list table/grid
- `AddAccountModal.svelte` (~200 lines) — add account form
- `ProfileEditForm.svelte` (~220 lines) — edit profile (persona, keywords, etc.)
- `AccountsSection.svelte` (~120 lines) — orchestrator

### Requirements
- [ ] Move account table into `AccountList` component
- [ ] Extract add account modal logic into `AddAccountModal`
- [ ] Extract profile editing into `ProfileEditForm`
- [ ] All components ≤ 400 lines
- [ ] Type-safe props and events
- [ ] Form validation preserved

### Definition of Done
- [ ] All components created and integrated
- [ ] All existing account management features work
- [ ] No API changes
- [ ] Unit tests for form validation

---

### Task 2.2b: CredentialCard Refactoring

**Current Size:** 713 lines (1.78x limit)  
**Target:** Split into credential-specific forms:
- `XApiCredentialForm.svelte` (~180 lines)
- `GoogleDriveCredentialForm.svelte` (~180 lines)
- `LlmProviderForm.svelte` (~180 lines) — includes OpenAI, Anthropic, Gemini, local Ollama
- `CredentialCard.svelte` (~120 lines) — tabs orchestrator

### Requirements
- [ ] Extract credential forms by provider
- [ ] Each form handles its own validation and submission
- [ ] Use shared credential store for state
- [ ] All forms ≤ 400 lines
- [ ] Error handling and loading states per form

### Definition of Done
- [ ] All credential types editable in their own forms
- [ ] All tests passing (validation, submission)
- [ ] API integration unchanged

---

## Task 2.3: MCP Policy Section Refactor

**File:** `dashboard/src/routes/(app)/mcp/PolicySection.svelte`  
**Current Size:** 887 lines (2.2x limit)  
**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 2

### Decomposition Plan
Split into three focused sections:

- `PolicyRulesSection.svelte` (~250 lines)
  - [ ] Rule list with CRUD actions
  - [ ] Rule editor modal
  - [ ] Drag-to-reorder rules
  
- `PolicyTemplatesSection.svelte` (~200 lines)
  - [ ] Template gallery/list
  - [ ] Template preview
  - [ ] Apply template action
  
- `PolicyEvaluationPanel.svelte` (~250 lines)
  - [ ] Evaluation mode visualization
  - [ ] Test policy with sample requests
  - [ ] Decision tree visualization
  
- `PolicySection.svelte` (~100 lines)
  - [ ] Tabs orchestrator (Rules, Templates, Evaluation)
  - [ ] State management

### Definition of Done
- [ ] All components ≤ 400 lines
- [ ] Policy workflow unchanged (can still create, edit, test rules)
- [ ] Type-safe across components
- [ ] Component tests for rule CRUD, template application

### Notes
- This is a complex domain; ensure each component is independently understandable.

---

## Task 2.4: Approval Queue UX Improvements

**Files:**  
- `dashboard/src/lib/components/ApprovalCard.svelte` (722 lines)  
- `dashboard/src/routes/(app)/approval/+page.svelte` (443 lines)

**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 2–3

### Task 2.4a: ApprovalCard Decomposition

**Current Size:** 722 lines (1.8x limit)  
**Target:** Extract concerns:
- `ApprovalCardHeader.svelte` (~120 lines) — title, metadata, status
- `ApprovalCardContent.svelte` (~200 lines) — tweet content, preview, edit
- `ApprovalCardActions.svelte` (~180 lines) — approve, reject, edit buttons
- `ApprovalHistory.svelte` (~150 lines) — edit history, decision log
- `RejectionDialog.svelte` (~120 lines) — rejection reason form
- `ApprovalCard.svelte` (~80 lines) — orchestrator

### Requirements
- [ ] All sub-components ≤ 400 lines
- [ ] Preserve all approval actions (approve, reject, edit)
- [ ] Edit history remains visible
- [ ] Type-safe props and events

### Definition of Done
- [ ] Components split and integrated
- [ ] All approval actions work
- [ ] Component tests for actions and state transitions

---

### Task 2.4b: Approval Queue Pagination & Performance

**Current Issue:** Large approval queues (100+ items) render slowly  
**Goal:** Add pagination/virtualization

### Requirements
- [ ] Implement pagination (default 20 items/page) OR virtual scrolling
- [ ] Add filter/search: by status, by type (tweet/reply/thread), by date
- [ ] Add bulk actions: approve multiple, reject multiple
- [ ] Load time for 1000+ approval items should be <2s

### Definition of Done
- [ ] Pagination or virtualization working
- [ ] Filters and search functional
- [ ] No performance regression on small queues
- [ ] Unit tests for filtering, pagination

---

## Task 2.5: Discovery Interface Improvements

**File:** `dashboard/src/routes/(app)/discovery/+page.svelte` (557 lines)  
**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 2–3 (LOW priority, can be deferred)

### Requirements
- [ ] Add inline tweet preview pane (split view)
  - Left: search results, scoring visualization
  - Right: selected tweet preview, compose reply modal
- [ ] Reduce navigation: no need to click into each tweet to reply
- [ ] Add keyboard navigation (arrow keys, enter to reply)

### Definition of Done
- [ ] Split-pane layout working
- [ ] Reply composition works in preview pane
- [ ] Keyboard shortcuts documented
- [ ] No performance regression

---

## Task 2.6: Dashboard Navigation Improvements

**Scope:** Global dashboard navigation  
**Status:** NOT STARTED  
**Assigned to:** [Frontend Lead]  
**Sprint:** 3 (LOW priority)

### Requirements
- [ ] Add breadcrumb navigation to key pages:
  - Settings → Accounts → (account edit)
  - Approval → (approval item)
  - Discovery → (tweet details)
- [ ] Add "back" button that preserves filter state
- [ ] Document navigation hierarchy

### Definition of Done
- [ ] Breadcrumbs visible on nested pages
- [ ] Back navigation preserves state
- [ ] No visual clutter

---

## Summary & Success Criteria

| Task | Est. Days | Priority | Sprint | Owner |
|------|-----------|----------|--------|-------|
| 2.1 Composer | 7–10 | HIGH | 1 | Frontend |
| 2.2a AccountsSection | 5–7 | HIGH | 1–2 | Frontend |
| 2.2b CredentialCard | 5–7 | HIGH | 1–2 | Frontend |
| 2.3 PolicySection | 7–10 | HIGH | 2 | Frontend |
| 2.4a ApprovalCard | 5–7 | MEDIUM | 2–3 | Frontend |
| 2.4b Pagination | 3–5 | MEDIUM | 2–3 | Frontend |
| 2.5 Discovery | 3–5 | LOW | 3+ | Frontend |
| 2.6 Navigation | 2–3 | LOW | 3+ | Frontend |

**Total Estimated Effort:** ~3–4 weeks  
**Expected Outcomes:**
- All Svelte components ≤ 400 lines
- Improved developer experience (smaller, testable components)
- Better user experience (pagination, previews, navigation)
- Foundation for comprehensive component testing

---

## Frontend Testing Checklist

For each decomposed component, ensure:
- [ ] Playwright component test created
- [ ] Props and events documented
- [ ] No console errors/warnings
- [ ] Accessibility (keyboard nav, ARIA labels)
- [ ] TypeScript strict mode compliant
- [ ] SvelteKit runes used correctly

