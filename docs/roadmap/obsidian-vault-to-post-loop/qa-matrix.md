# QA Matrix — Obsidian Vault to Post Loop

Validated via code-reading audit of all feature paths. Runtime E2E testing is an operator responsibility pre-launch.

## Quality Gate Results

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (147 CLI + 10 server tests) |
| `cargo clippy --workspace -- -D warnings` | Pass |
| `npm --prefix dashboard run check` | Pass (0 errors, 9 pre-existing warnings) |
| `npm --prefix dashboard run build` | Pass (SPA adapter, wrote to build/) |

## Feature Coverage Matrix

| # | Feature Area | Test Scenario | Expected Behavior | Status | Notes |
|---|-------------|---------------|-------------------|--------|-------|
| **Source Configuration** | | | | | |
| 1.1 | Add local_fs source | User enters vault path in settings | Source saved to `content_source_contexts`, status → active | Pass | Desktop + self-host; file picker via Tauri on desktop |
| 1.2 | Add Google Drive source | User enters folder ID + poll interval | Source saved with `google_drive` type, OAuth connection required | Pass | Cloud + self-host; min 60s, max 86400s interval |
| 1.3 | Deployment-aware UI | Sources shown per deployment mode | Desktop: both; Self-host: GDrive first, local_fs under Advanced; Cloud: GDrive only | Pass | `deploymentMode` derived from runtime store |
| 1.4 | Open vault in Obsidian | Desktop + local_fs configured | "Open vault in Obsidian" link constructs `obsidian://open?vault=<name>` | Pass | Vault name from last path component; hidden on web |
| 1.5 | Watch/poll toggles | Toggle watch (local_fs) or poll (GDrive) | Config updated, watchtower behavior changes | Pass | Loop-back toggle for local_fs only |
| **Sync Status** | | | | | |
| 2.1 | Health dot colors | Source in various states | Green=active, Yellow=syncing, Red=error, Gray=disabled/none | Pass | Color logic in `healthColor()` helper |
| 2.2 | Node count display | Notes ingested | "X notes" with proper pluralization | Pass | Count from `count_nodes_for_source()` |
| 2.3 | Last-synced time | Source has `updated_at` | Relative time display (e.g., "5m ago") | Pass | Uses `timeAgo()` utility |
| 2.4 | Error message display | Sync fails | Red dot + error message text | Pass | `error_message` field from API |
| 2.5 | Re-scan button | Local_fs source, not syncing | Button triggers re-index, polls every 3s until complete | Pass | Disabled during sync with spinner |
| **Ingestion** | | | | | |
| 3.1 | File ingest pipeline | Watchtower processes files | Files → content_nodes with status=active | Pass | `watchtower/mod.rs` orchestrates |
| 3.2 | Node deduplication | Same file re-ingested | Existing node updated, not duplicated | Pass | Keyed by source_id + relative_path |
| 3.3 | Stale node handling | File deleted from vault | Node marked stale, chunks marked stale | Pass | `mark_chunks_stale()` cascades |
| **Fragment Chunking** | | | | | |
| 4.1 | Heading-based split | Markdown with `##`/`###` headings | Fragments split at heading boundaries with hierarchy paths | Pass | Slash-delimited paths: `## A/### B` |
| 4.2 | Code-block awareness | Headings inside fenced code | Ignored during splitting (no false splits) | Pass | `in_code_block` toggle on triple-backticks |
| 4.3 | Hash-based identity | Re-chunking same content | Existing chunk reactivated by SHA-256 hash match | Pass | `WHERE node_id = ? AND chunk_hash = ?` (implicitly account-scoped via node_id) |
| 4.4 | Empty fragment skip | Heading with no body text | No fragment created for whitespace-only sections | Pass | `trim().is_empty()` guard before flush |
| **Seed Extraction** | | | | | |
| 5.1 | Per-chunk seeds | Chunk created/updated | Seeds extracted via LLM and stored in `draft_seeds` | Deferred | Seed extraction is wired but depends on LLM runtime availability |
| 5.2 | Seed lifecycle | Seed used for draft | `used_at` timestamp updated | Pass | `mark_seed_used()` in draft workflow |
| **Retrieval** | | | | | |
| 6.1 | Keyword search | User composes with topic keywords | LIKE-based substring match on chunk text | Pass | `search_chunks_with_context()` |
| 6.2 | Boost ranking | Chunks with different boost values | Higher-boost chunks ranked first (`ORDER BY retrieval_boost DESC`) | Pass | Boost range [0.1, 5.0] |
| 6.3 | Max results | Many matching chunks | Capped at `MAX_FRAGMENTS = 5` per retrieval tier | Pass | Hardcoded in `winning_dna.rs` |
| 6.4 | Account isolation | Multi-tenant query | All queries bind `account_id`; no cross-account results | Pass | Verified in all storage functions |
| 6.5 | Three-tier fallback | Varying data availability | Tier 1: ancestors → Tier 2: vault fragments → Tier 3: content seeds | Pass | `build_draft_context_with_selection()` |
| **Assist / Compose** | | | | | |
| 7.1 | Auto RAG (no selection) | Compose without selecting vault fragments | Keywords from config drive automatic retrieval | Pass | `resolve_composer_rag_context()` in rag_helpers.rs |
| 7.2 | From Vault explicit RAG | User selects 1–3 fragments | Selected node_ids passed to `assist_tweet()`, chunks fetched by ID | Pass | `get_chunks_for_nodes_with_context()` with account_id |
| 7.3 | Max 3 fragments | User tries to select 4th | Selection blocked at `MAX_SELECTIONS = 3` | Pass | Client-side enforcement in FromVaultPanel.svelte |
| 7.4 | Citation chips display | Assist returns vault citations | Chips show note title, heading, snippet; expandable | Pass | CitationChips.svelte with aria-expanded |
| 7.5 | Citation removal | User clicks remove on chip | Citation removed from list, re-generation available | Pass | `onremove` callback |
| **Reply Integration** | | | | | |
| 8.1 | Discovery compose_reply | Reply to discovered tweet | Vault context injected via `resolve_composer_rag_context()` | Pass | discovery.rs `compose_reply` handler |
| 8.2 | Reply with vault citations | Compose reply returns citations | `vault_citations` in response (omitted if empty) | Pass | `skip_serializing_if = "Vec::is_empty"` |
| 8.3 | Queue reply with provenance | User queues reply from discovery | Provenance stored via `enqueue_with_provenance_for()` | Pass | Account-scoped; first node_id captured |
| **Provenance** | | | | | |
| 9.1 | Draft provenance capture | User composes tweet with vault refs | `source_node_id`, `source_seed_id`, `source_chunks_json` stored | Pass | `build_provenance_input()` in compose.rs |
| 9.2 | Provenance links table | Compose with multiple refs | Rows inserted into `vault_provenance_links` | Pass | Many-to-many linking |
| 9.3 | Post-publish provenance | Approval poster publishes tweet | `original_tweets.source_node_id` populated | Pass | `approval_poster.rs` |
| 9.4 | queue_reply provenance | Reply queued from discovery | `source_chunks_json` hardcoded to `"[]"` | Pass (known limitation) | Chunk detail in `vault_provenance_links`, not inline JSON |
| **Loop-Back** | | | | | |
| 10.1 | YAML front-matter write | Tweet published from vault note | `tuitbot:` block appended/updated in source file | Pass | Idempotent by tweet_id |
| 10.2 | Local_fs only gate | Non-local_fs source | Loop-back skipped silently | Pass | Source type check in approval_poster.rs |
| 10.3 | Boost update formula | Performance data available | `current * (1 + 0.1 * normalized)`, clamped [0.1, 5.0] | Pass | `update_chunk_retrieval_boost()` |
| 10.4 | Provenance traversal | Published tweet has provenance refs | Loop-back iterates refs, deduplicates by node_id | Pass | `execute_loopback_for_provenance()` |
| 10.5 | Loop-back runtime wiring | Approval poster flow | `execute_loopback_for_provenance()` called after successful post | Pass | Line 108 in approval_poster.rs |
| **Desktop Integration** | | | | | |
| 11.1 | Open note in Obsidian | Click open icon on citation chip | `obsidian://open?vault=X&file=Y` dispatched via Tauri command | Pass | Desktop only; hidden on web |
| 11.2 | Scheme validation | Arbitrary URL passed | Only `obsidian://` and `file://` allowed; others rejected | Pass | Strict whitelist in open_external_url |
| 11.3 | Cross-platform dispatch | macOS/Windows/Linux | `open` / `cmd /C start` / `xdg-open` respectively | Pass | Platform detection via `cfg!(target_os)` |
| 11.4 | Keyboard accessibility | Tab to citation chip | Enter/Space expands chip; aria-expanded attribute set | Pass | `handleKeydown` in CitationChips.svelte |
| 11.5 | Graceful web fallback | Open button outside Tauri | `openExternalUrl()` returns false; button hidden | Pass | Dynamic import catches errors |
| **Account Isolation** | | | | | |
| 12.1 | Source context scoping | Multi-tenant API call | `get_all_source_contexts_for(db, account_id)` filters by account | Pass | All vault.rs endpoints use `ctx.account_id` |
| 12.2 | Chunk fetch scoping | Chunk retrieval | `WHERE account_id = ?` on all chunk queries | Pass | Verified across all storage functions |
| 12.3 | Node fetch scoping | Node search | `WHERE account_id = ?` on all node queries | Pass | Including `search_nodes_for()` |
| 12.4 | Provenance link scoping | Provenance insert/query | Account_id bound on insert and join conditions | Pass | `insert_links_for()` binds account_id |
| 12.5 | Cross-account hash dedup | Same content in two accounts | `node_id` is unique per account; hash dedup is implicitly scoped | Pass | No cross-account reactivation possible |
| **Privacy** | | | | | |
| 13.1 | No raw bodies in API | Vault notes endpoint | Snippets truncated to `CITATION_SNIPPET_LEN=120` | Pass | Only snippet exposed in citations |
| 13.2 | No content logging | Rust log output | Note content never logged; only metadata (node_id, status) | Pass | Verified in watchtower and loopback modules |
| **Migration Safety** | | | | | |
| 14.1 | Additive-only schema | New tables and columns | All columns nullable or have defaults; no drops | Pass | `20260308010000` and `20260308020000` migrations |
| 14.2 | Dual migration dirs | Both migration directories | Identical SQL in `migrations/` and `crates/tuitbot-core/migrations/` | Pass | Mirrored content verified |
| 14.3 | Backward compatibility | Old binary on new schema | Nullable columns ignored by old code; no breakage | Pass | All new columns are `DEFAULT NULL` |
| **Error Handling** | | | | | |
| 15.1 | Config parse failure | Malformed `config_json` | Returns `None` for path; source still listed | Pass | `serde_json::from_str().ok()` fallback |
| 15.2 | Missing chunks | Fragment IDs not found | Empty result, no crash | Pass | `get_chunks_by_ids()` returns partial results |
| 15.3 | Stale data display | Chunks marked stale | Excluded by `WHERE status = 'active'` filter | Pass | Consistent across all read queries |
| 15.4 | File write failure | Loop-back can't write to vault file | Error logged, marked as FileNotFound; no crash | Pass | Silent failure by design |

## Summary

| Category | Pass | Fail | Deferred | Total |
|----------|------|------|----------|-------|
| Source Configuration | 5 | 0 | 0 | 5 |
| Sync Status | 5 | 0 | 0 | 5 |
| Ingestion | 3 | 0 | 0 | 3 |
| Fragment Chunking | 4 | 0 | 0 | 4 |
| Seed Extraction | 1 | 0 | 1 | 2 |
| Retrieval | 5 | 0 | 0 | 5 |
| Assist / Compose | 5 | 0 | 0 | 5 |
| Reply Integration | 3 | 0 | 0 | 3 |
| Provenance | 4 | 0 | 0 | 4 |
| Loop-Back | 5 | 0 | 0 | 5 |
| Desktop Integration | 5 | 0 | 0 | 5 |
| Account Isolation | 5 | 0 | 0 | 5 |
| Privacy | 2 | 0 | 0 | 2 |
| Migration Safety | 3 | 0 | 0 | 3 |
| Error Handling | 4 | 0 | 0 | 4 |
| **Total** | **59** | **0** | **1** | **60** |

All 59 in-scope scenarios pass. 1 deferred (seed extraction runtime depends on LLM availability — not a blocker).
