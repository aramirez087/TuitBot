# QA Matrix — Backlink Synthesizer

Validated on 2026-03-21 during Session 7. All gates green.

## Test Evidence

| Category | Subcategory | Test Location | Count | Status |
|----------|------------|---------------|-------|--------|
| **Ingestion** | Link extraction | `tuitbot-core` link_extractor tests | 17 | PASS |
| | Edge storage CRUD | `tuitbot-core` tests_graph.rs | 5 | PASS |
| | Tag storage CRUD | `tuitbot-core` tests_graph.rs | 4 | PASS |
| | Shared-tag discovery | `tuitbot-core` tests_graph.rs | 2 | PASS |
| | Re-ingest idempotency | `tuitbot-core` tests_graph.rs | 2 | PASS |
| | E2E chunk-to-edge flow | `tuitbot-core` tests_graph.rs | 3 | PASS |
| | Migration verification | `tuitbot-core` tests_graph.rs | 2 | PASS |
| **Retrieval** | Graph expansion (unit) | `graph_expansion.rs` #[cfg(test)] | 33 | PASS |
| | Scoring/classification | `graph_expansion.rs` (subset) | 15 | PASS |
| | GraphState serialization | `graph_expansion.rs` (subset) | 5 | PASS |
| **API** | Neighbors endpoint | `graph_neighbor_tests.rs` | 11 | PASS |
| | Tag-only neighbors | `graph_neighbor_tests.rs` (new) | 1 | PASS |
| | Intent classification E2E | `graph_neighbor_tests.rs` (new) | 1 | PASS |
| | Response contract completeness | `graph_neighbor_tests.rs` (new) | 1 | PASS |
| | Telemetry endpoint | `tuitbot-server` telemetry tests | 4 | PASS |
| | Privacy (Cloud mode) | `graph_neighbor_tests.rs` | 1 | PASS |
| | Account isolation | `graph_neighbor_tests.rs` | 1 | PASS |
| **UX** | GraphSuggestionCards | `GraphSuggestionCards.test.ts` | 22 | PASS |
| | VaultSelectionReview | `VaultSelectionReview.test.ts` | 55 | PASS |
| | SlotTargetPanel | `SlotTargetPanel.test.ts` | 9 | PASS |
| | CitationChips | `CitationChips.test.ts` | 18 | PASS |
| | FromVaultPanel | `FromVaultPanel.test.ts` | 59 | PASS |
| | draftInsertStore | `draftInsertStore.test.ts` | 21 | PASS |
| | Dismissed recovery flow | `VaultSelectionReview.test.ts` (new) | 2 | PASS |
| | unresolved_links state | `GraphSuggestionCards.test.ts` (new) | 1 | PASS |
| **Analytics** | backlinkFunnel helpers | `backlinkFunnel.test.ts` (new) | 15 | PASS |
| **E2E** | Composer basics | `composer.spec.ts` | 7 | PASS |

## Totals

- **Rust tests (workspace-wide):** 572 passed, 0 failed
- **Frontend unit tests:** 851 passed, 0 failed
- **Test files:** 44 frontend, full Rust workspace

## Session 7 Additions

| Test | File | What it validates |
|------|------|-------------------|
| `neighbors_tag_only_returns_shared_tag_reason` | `graph_neighbor_tests.rs` | Tag-only neighbors return `shared_tag` reason with tag name in label |
| `neighbors_response_fields_complete` | `graph_neighbor_tests.rs` | API response includes all contract fields (node_id, reason, intent, score, etc.) |
| `neighbors_intent_field_from_edge_label` | `graph_neighbor_tests.rs` | Edge label "quick tip" classifies as `pro_tip` intent through full API stack |
| `graph_state_all_variants_serialize` | `graph_expansion.rs` | All 5 GraphState variants serialize to correct snake_case strings |
| `score_tag_only_neighbor` | `graph_expansion.rs` | Score with only shared tags (0 direct, 0 backlinks, 2 tags) = 2.0 |
| `classify_reason_zero_direct_zero_backlink_with_tags` | `graph_expansion.rs` | 0 direct + 0 backlinks + 5 tags = SharedTag |
| `classify_reason_zero_everything_defaults_linked` | `graph_expansion.rs` | Edge case: all zeros defaults to LinkedNote |
| `shows empty message for unresolved_links` | `GraphSuggestionCards.test.ts` | `unresolved_links` state renders empty message |
| `shows dismissed recovery section` | `VaultSelectionReview.test.ts` | "Show skipped" toggle appears after dismissing a card |
| `restoring a dismissed card adds it back` | `VaultSelectionReview.test.ts` | Restore flow: dismiss -> expand -> restore -> card reappears |
| 15 backlinkFunnel tests | `backlinkFunnel.test.ts` | All 11 event helpers + 4 backend relay tests |

## Coverage Gaps (Accepted)

1. **No E2E test for full backlink suggestion-to-insertion flow** — requires live API + Obsidian plugin mock. Unit + integration tests at each layer provide sufficient coverage.
2. **Backend telemetry logs only (no persistence)** — acceptable for initial release; events go to tracing logs.
3. **`flushToBackend()` not auto-wired** — console events sufficient for validation phase.
4. **No multi-hop graph traversal tests** — only 1-hop neighbors are implemented; multi-hop is out of scope.

## Consistency Pass Results

| Check | Result |
|-------|--------|
| Rust `GraphState` (5 variants) matches TypeScript `GraphState` | FIXED — added `unresolved_links` to TS type |
| `GraphSuggestionCards` handles `unresolved_links` | FIXED — added to empty-state branch |
| API response struct fields match `graph-api-contract.md` | PASS |
| Scoring weights match `retrieval-ranking-spec.md` | PASS |
| UI copy matches `ux-copy-and-state-notes.md` | PASS |
| Funnel events match `instrumentation-plan.md` | PASS |
