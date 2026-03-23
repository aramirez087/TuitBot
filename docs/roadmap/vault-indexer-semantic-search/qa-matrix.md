# QA Matrix — Semantic Evidence Search

## Scenario x Deployment Mode

| # | Scenario | Desktop | Self-Host | Cloud | Verified By |
|---|----------|---------|-----------|-------|-------------|
| 1 | Fresh install, no embedding config | Rail hidden | Rail hidden | Rail hidden | `EvidenceRail.test.ts`: "renders nothing when provider not configured" |
| 2 | Ollama running, vault indexed | Full semantic search | Full semantic search | N/A | Manual + `evidence.rs` hybrid_mode test |
| 3 | OpenAI configured, vault indexed | Full semantic search | Full semantic search | Full semantic search | `evidence.rs`: keyword/hybrid/semantic mode tests |
| 4 | Provider unreachable mid-session | Keyword fallback + badge warning | Keyword fallback + badge warning | Keyword fallback + badge warning | `evidence.rs`: `hybrid_mode_falls_back_without_provider`, `IndexStatusBadge.test.ts`: "shows keyword fallback" |
| 5 | Stale index (<50% fresh) | Amber badge + stale warning banner | Amber badge + stale warning | Amber badge + stale warning | `EvidenceRail.test.ts`: "shows stale warning when freshness < 50%", `IndexStatusBadge.test.ts`: "renders amber dot" |
| 6 | Empty vault (0 chunks) | Gray badge, "Building index..." | Gray badge, "Building..." | Gray badge, "Building..." | `EvidenceRail.test.ts`: "shows building state when index is empty", `IndexStatusBadge.test.ts`: "renders gray dot when no index" |
| 7 | Model switch mid-session | Re-index triggered, old purged | Re-index triggered | Re-index triggered | `SemanticIndex::rebuild()` + `EmbeddingWorker` (Session 2) |
| 8 | Cloud privacy invariants | N/A | N/A | No `relative_path`, snippet-only | `evidence.rs`: `cloud_mode_omits_relative_path` |
| 9 | Pin evidence | Pin added, badge count increments | Same | Same | `evidenceStore.test.ts`, `EvidenceRail.test.ts`: "shows pinned section" |
| 10 | Dismiss evidence | Card removed from results | Same | Same | `evidenceStore.test.ts` |
| 11 | Apply to slot | LLM refines block, undo available | Same | Same | `ComposerInspector.svelte`: `handleApplyEvidence()` |
| 12 | Auto-query in tweet mode | 800ms debounce, semantic query | Same | Same | `EvidenceRail.svelte` $effect auto-query |
| 13 | Auto-query in thread mode | Focused block text used as query | Same | Same | `focusedText` derived in `EvidenceRail.svelte` |
| 14 | Strengthen draft (tweet) | Single block refined with all pinned evidence | Same | Same | `ComposerInspector.svelte`: `handleStrengthenDraft()` |
| 15 | Strengthen draft (thread) | Per-block inserts with per-block undo | Same | Same | Same handler, iterates thread blocks |
| 16 | Undo single slot insert | Previous text restored, insert removed | Same | Same | `draftInsertStore.test.ts`: undo tests |
| 17 | Provenance chain integrity | ProvenanceRef attached through inspector -> workspace -> API -> approval | Same | Same | `ComposerInspector.svelte`: `vaultProvenance` tracking |
| 18 | Limit=0 defaults to 8 | 8 results max | Same | Same | `evidence.rs`: `limit_zero_defaults_to_8` |
| 19 | Limit>20 clamped to 20 | 20 results max | Same | Same | `evidence.rs`: `limit_over_max_clamped_to_20` |
| 20 | Invalid selection scope | Empty results, no error | Same | Same | `evidence.rs`: `invalid_scope_returns_ok_with_empty_results` |

## Build Gates

| Gate | Command | Status |
|------|---------|--------|
| Rust format | `cargo fmt --all --check` | Pass |
| Rust lint | `cargo clippy --workspace -- -D warnings` | Pass |
| Rust tests | `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (6401 tests) |
| Frontend type check | `npm --prefix dashboard run check` | Pass (0 errors) |
| Frontend unit tests | `npx vitest run` (dashboard) | Pass (1114 tests) |

## Privacy Invariants

| # | Invariant | Desktop | Self-Host | Cloud |
|---|-----------|---------|-----------|-------|
| P1 | Raw note bodies never exposed in read APIs beyond existing rules | Yes | Yes | Yes |
| P2 | `relative_path` omitted in Cloud mode | N/A | N/A | Enforced in `evidence_to_item()` |
| P3 | Embeddings stored locally in Desktop mode | Yes — SQLite | N/A | N/A |
| P4 | Embeddings stored on user's server in Self-host mode | N/A | Yes — SQLite | N/A |
| P5 | Snippets truncated in Cloud mode | N/A | N/A | Via existing chunk truncation |
| P6 | Account scoping on all queries | Yes — `account_id` filter | Yes | Yes |
| P7 | Selection TTL enforced (30 min default) | Yes | Yes | Yes |
| P8 | Privacy envelope label accurate in settings | Yes — "locally" | Yes — "your server" | Yes — "server-side" |
| P9 | Privacy envelope label accurate in badge popover | Yes | Yes | Yes |
| P10 | No cross-account embedding leakage | Enforced by `get_index_stats_for(account_id)` | Same | Same |
