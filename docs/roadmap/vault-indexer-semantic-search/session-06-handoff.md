# Session 06 Handoff: Observability and Release Readiness

## What Changed

Instrumented semantic evidence search for production monitoring, added runtime and settings surfaces for index status and privacy across all deployment modes, ran end-to-end validation, and produced release documentation.

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-server/src/routes/telemetry.rs` | Added `"evidence."` to `ALLOWED_PREFIXES` + `evidence_prefix_accepted` test |
| `crates/tuitbot-server/src/routes/vault/index_status.rs` | Added `deployment_mode`, `search_available`, `provider_name` fields to `IndexStatusResponse` + 1 new test |
| `crates/tuitbot-server/src/routes/vault/evidence.rs` | Added `tracing::info!` span for search latency/fallback + 3 edge case tests |
| `dashboard/src/lib/api/types.ts` | Added 3 optional fields to `IndexStatusResponse` |
| `dashboard/src/lib/analytics/evidenceFunnel.ts` | Added backend relay (buffer + flush) + 3 new event helpers: `trackEvidenceSearchLatency`, `trackEvidenceDraftMode`, `trackEvidenceStrengthen` |
| `dashboard/src/lib/components/composer/EvidenceRail.svelte` | Added latency measurement in `executeSearch()` |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Added `trackEvidenceStrengthen` call in `handleStrengthenDraft()` |
| `dashboard/src/lib/components/composer/IndexStatusBadge.svelte` | Added `deploymentMode` prop, privacy label row, search availability row, provider name row |
| `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` | Added semantic index status section with privacy notice |
| `dashboard/tests/unit/EvidenceRail.test.ts` | Fixed type cast (`as unknown as`), added `trackEvidenceSearchLatency` to mock |
| `dashboard/tests/unit/IndexStatusBadge.test.ts` | Updated `makeStatus` helper with new fields, added 7 new tests for privacy/search/provider |

### Files Created

| File | Purpose |
|---|---|
| `dashboard/tests/unit/evidenceFunnel.test.ts` | 14 tests: event helpers + backend relay |
| `docs/roadmap/vault-indexer-semantic-search/metrics-and-rollout.md` | Event catalog, success metrics, rollout plan |
| `docs/roadmap/vault-indexer-semantic-search/qa-matrix.md` | 20-scenario QA matrix across 3 deployment modes + 10 privacy invariants |
| `docs/roadmap/vault-indexer-semantic-search/release-readiness.md` | Go/no-go report with explicit evidence |
| `docs/roadmap/vault-indexer-semantic-search/session-06-handoff.md` | This file |

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | No feature flag — ship default-on | Semantic search is additive and fail-open. `provider_configured` is the natural gate. |
| D2 | Backend tracing spans + frontend funnel events | Matches existing patterns (`backlink.` prefix, `tracing::info!`). No new infrastructure. |
| D3 | Privacy labels from `deployment_mode` in `IndexStatusResponse` | Avoids extra API calls — badge and settings compute labels from the single status endpoint. |
| D4 | No separate `semantic_search_enabled` config flag | `embedding_provider.is_some()` backend / `provider_configured` frontend is the natural gate. Redundant flag adds config migration burden without safety benefit. |
| D5 | Extend `IndexStatusResponse` rather than new endpoint | Three lightweight fields (`deployment_mode`, `search_available`, `provider_name`) keep API surface minimal. |
| D6 | Backend relay for evidence telemetry | Buffer at 10 events, flush via `POST /api/telemetry/events`. Matches `backlinkFunnel.ts` pattern exactly. |

## Exit Criteria Met

- [x] Latency, fallback, and adoption are measurable in production (10 event types + backend tracing)
- [x] Supported deployment modes communicate index status honestly (IndexStatusBadge popover, ContentSourcesSection)
- [x] Supported deployment modes communicate privacy envelope honestly (3 labels per mode)
- [x] Release report ends with explicit GO decision backed by evidence
- [x] `cargo fmt --all --check` passes
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `RUSTFLAGS="-D warnings" cargo test --workspace` passes (6401 tests)
- [x] `npm --prefix dashboard run check` passes (0 errors)
- [x] `npx vitest run` passes (1114 tests)

## Test Summary

| Scope | Count | Status |
|-------|-------|--------|
| Rust workspace | 6401 | All pass |
| Frontend unit tests | 1114 | All pass |
| New Rust tests (this session) | 5 | All pass |
| New frontend tests (this session) | 21 | All pass |

## Residual Risks

1. **Cloud E2E not runtime-validated**: Cloud mode privacy paths are tested at the unit level (`cloud_mode_omits_relative_path`, privacy label assertions) but lack a running cloud environment test. Mitigated by unit coverage and the fail-open design.

2. **Embedding provider latency is provider-dependent**: OpenAI and Ollama have different latency profiles. The 800ms auto-query debounce may need tuning based on real-world latency data. Monitor `evidence.search_latency` events after rollout.

3. **ContentSourcesSection line count**: Now ~400 lines with the semantic section. At the 400-line Svelte limit. Future additions should consider extracting a `SemanticIndexSection.svelte` component.

## Follow-Up Work

1. **Hook picker UI** (Session 5 carryover): Wire `api.assist.hooks()` to a hook picker component. Session 5 built the backend + frontend types; the UI component and compose flow remain.

2. **Selection cleanup wiring** (Session 4 carryover): `vault_selections::cleanup_expired()` should be wired into the server's hourly cleanup loop.

3. **Latency dashboard**: Once `evidence.search_latency` and `evidence_search_completed` events are flowing, build a Grafana/metrics view for P50/P95 monitoring.

4. **Embedding provider health check**: Add periodic `provider.health_check()` calls to proactively detect provider outages rather than waiting for search failures.

5. **ContentSourcesSection extraction**: Consider splitting the semantic index section into a standalone `SemanticIndexSection.svelte` to stay under the 400-line limit.
