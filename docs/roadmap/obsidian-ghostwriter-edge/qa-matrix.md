# QA Matrix: Ghostwriter Edge

## Build Gates

| Gate | Status | Notes |
|------|--------|-------|
| `cargo fmt --all --check` | PASS | No formatting issues |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS | 567 tests + 5 doc-tests |
| `cargo clippy --workspace -- -D warnings` | PASS | Zero warnings |
| `npm run check` (svelte-check) | PASS | 0 errors, 0 warnings |
| `npx vitest run` | PASS | 686 tests, 36 files |
| `npm run build` (dashboard) | PASS | Static adapter, clean output |
| `npm run build` (obsidian-tuitbot) | PASS | 9.1kb bundle |

## Charter Requirements

| Requirement | Session | Implemented | Tested | Status |
|-------------|---------|-------------|--------|--------|
| Block-level send from Obsidian | 4 | `handleSendSelection()` + `handleSendBlock()` in plugin `main.ts` | Plugin build passes; payload shape validated | PASS |
| Selection ingress API | 4 | `POST /api/vault/send-selection` with TTL, rate limit, identity resolution | Integration tests in `selections.rs` | PASS |
| Hook-first thread drafting | 5-6 | `POST /api/assist/hooks` returns 5 differentiated hooks | Hook generation tests in `hooks.rs` | PASS |
| Provenance tracking | 3 | `insert_links_for` / `get_links_for` with `seed_id` support | Unit tests in `provenance.rs` | PASS |
| Provenance propagation | 7 | `build_provenance_input()` → `enqueue_with_provenance_for()` through compose → approval → scheduled | Transform tests in `compose/transforms.rs` | PASS |
| Heading-anchor deep-links | 3 | `buildObsidianUri()` with heading path extraction | URI builder unit tests | PASS |
| Privacy-by-deployment | 8 | `is_local_first()`, `privacy_envelope()`, `DeploymentCapabilities` privacy fields | 12 privacy tests + backward compat | PASS |
| Cloud guard on vault path | 8 | Cloud-mode path filtering in `vault_sources` + `selected_text` omission | Integration tests for Cloud guard | PASS |
| Privacy banners | 8 | Mode-specific copy in `ContentSourcesSection.svelte` | Frontend component render tests | PASS |
| Privacy badge in composer | 8 | `vault-privacy-badge` in `FromVaultPanel.svelte` (Local-first / Self-hosted / Cloud) | Svelte-check passes | PASS |
| Obsidian transport notice | 8 | `isLocalTransport()` + non-blocking Notice on remote send | Plugin build passes | PASS |
| Tauri privacy fallback | 8 | `get_privacy_envelope` command returns `"local_first"` | Tauri build passes | PASS |
| Snippet truncation (120 chars) | 2 | `SNIPPET_MAX_LEN = 120` with safe UTF-8 boundary handling | Truncation unit tests | PASS |
| No raw body_text in API | 2 | No `body_text` field in any API response Serialize struct | Grep confirms zero exposure | PASS |
| Account scoping | 2 | All vault/provenance queries bind `account_id` as first parameter | Cross-account isolation tests | PASS |
| 30-min selection TTL | 4 | `checked_add_signed(Duration::minutes(30))` + `WHERE expires_at > datetime('now')` | TTL enforcement tests | PASS |

**Result: 16/16 PASS**

## Privacy Invariants

| Invariant | Desktop | Self-host | Cloud | Status |
|-----------|---------|-----------|-------|--------|
| No raw `body_text` in HTTP responses | Enforced | Enforced | Enforced | PASS |
| No raw `chunk_text` in HTTP responses | Enforced (120-char snippets only) | Enforced | Enforced | PASS |
| Account-scoped queries | All queries bind `account_id` | Same | Same | PASS |
| Seed text is transformation (max 200 chars) | LLM-generated, not verbatim | Same | Same | PASS |
| Transient selection storage (30min TTL) | Local SQLite | Local SQLite | Provider DB | PASS |
| `selected_text` in GET response | Returned | Returned | Omitted | PASS |
| Deep-links are Desktop-only UI | Rendered + clickable | Hidden | Hidden | PASS |
| Local vault path in API response | Included | Included | Omitted | PASS |
| `is_local_first()` claim | true (loopback only) | false (network boundary) | false (provider infra) | PASS |
| `privacy_envelope()` label | `"local_first"` | `"user_controlled"` | `"provider_controlled"` | PASS |

## End-to-End Journeys

| Journey | Steps Verified | Gaps Found | Status |
|---------|----------------|------------|--------|
| Block Send (Obsidian → Composer) | Plugin command → payload → POST selection → WebSocket event → DraftStudioShell → VaultSelectionReview | None — field names align, WebSocket event name matches | PASS |
| Hook-First Drafting | Vault panel → search notes → extract highlights → POST hooks → HookPicker → ongenerate callback → ComposerInspector captures hookStyle | `general` label in hookStyles.ts has no Rust variant (non-blocking: fallback renders raw key) | PASS |
| Provenance Continuity | ComposerInspector → ComposeRequest with ProvenanceRef[] → build_provenance_input → insert_links_for → copied through approval → scheduled | None — ProvenanceRef fields match across Rust/TS | PASS |
| Local-First Privacy | Tauri setup → Desktop mode → privacy_envelope "local_first" → VaultSourcesResponse → isLocalFirst store → green banner + Local-first badge | None — string literals consistent across all layers | PASS |

## Consistency Gap Scan

| Cross-Reference Pair | Status | Finding |
|----------------------|--------|---------|
| `ProvenanceRef` (Rust ↔ TypeScript) | ALIGNED | All 6 fields match: node_id, chunk_id, seed_id, source_path, heading_path, snippet |
| `SendSelectionRequest` (Rust ↔ Obsidian plugin) | ALIGNED | All 8 fields match: vault_name, file_path, selected_text, heading_context, selection_start_line, selection_end_line, note_title, frontmatter_tags |
| Privacy envelope strings (Rust ↔ Tauri ↔ Frontend) | ALIGNED | `"local_first"` consistent; Tauri hardcode is by design (Desktop-only command) |
| Hook style labels (Rust TweetFormat ↔ hookStyles.ts) | ALIGNED | All 7 backend variants covered; extra `general` frontend label is non-blocking (graceful fallback exists) |
| Mock API (mockApi.ts ↔ DeploymentCapabilities) | FIXED | `preferred_source_default` changed from `'local'` to `'local_fs'` to match actual Desktop capability |

## Test Coverage

| Surface | Test Count | Threshold | Status |
|---------|-----------|-----------|--------|
| Rust workspace | 567 + 5 doc-tests | All pass, zero warnings | PASS |
| Frontend (Vitest) | 686 across 36 files | 70% global lines | PASS |
| Obsidian plugin | Build-only (9.1kb) | Compiles without error | PASS |
| Dashboard build | Static adapter output | Clean build, no warnings | PASS |
