# Session 07 Handoff: Provenance & Hook Propagation

## What Changed

Hook style and vault provenance now propagate through the full content lifecycle: compose → draft/approval/schedule → publish. The compose endpoint accepts `provenance` and `hook_style` fields, stores provenance links in `vault_provenance_links`, and encodes the hook style into the `source` field. The Draft Studio displays provenance as citation chips and hook style as a metadata badge.

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-server/src/routes/content/compose/mod.rs` | Added `provenance` and `hook_style` fields to `ComposeThreadRequest`, `ComposeTweetRequest`, and `ComposeRequest` |
| `crates/tuitbot-server/src/routes/content/compose/transforms.rs` | Insert provenance links after all 4 `scheduled_content::insert_for()` call sites |
| `crates/tuitbot-server/src/routes/approval/handlers.rs` | Copy provenance links on approval → scheduled_content bridge (2 paths) |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Added `hook_style` to `CreateDraftRequest`, source enrichment logic, `get_draft_provenance` endpoint |
| `crates/tuitbot-server/src/routes/content/draft_studio/mod.rs` | Copy provenance on draft duplication |
| `crates/tuitbot-server/src/routes/content/mod.rs` | Re-export `get_draft_provenance` |
| `crates/tuitbot-server/src/lib.rs` | Register `/drafts/{id}/provenance` routes |
| `crates/tuitbot-core/src/storage/scheduled_content/tests/provenance.rs` | 5 provenance lifecycle regression tests |
| `dashboard/src/lib/api/types.ts` | Added `hook_style` to `ComposeRequest`, added `ProvenanceLink` interface |
| `dashboard/src/lib/api/client.ts` | Added `draftStudio.provenance()` API method |
| `dashboard/src/lib/utils/composeHandlers.ts` | Wire `provenance` and `hookStyle` into `buildComposeRequest()` |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Track vault provenance and hook style, expose getters |
| `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` | Pass provenance and hookStyle to `buildComposeRequest()` on submit |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Pass `hookStyle` in `ongenerate` callback |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Pass `hookStyle` in `ongenerate` callback |
| `dashboard/src/lib/components/composer/InspectorContent.svelte` | Updated `ongeneratefromvault` type signature |
| `dashboard/src/lib/components/drafts/DraftMetadataSection.svelte` | Hook style badge rendering with `getStyleLabel()` |
| `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` | Fetch and pass provenance to details pane |
| `dashboard/src/lib/components/drafts/DraftStudioDetailsPane.svelte` | Convert provenance to CitationChips, render sources section |

### Files Created

| File | Purpose |
|---|---|
| `dashboard/tests/unit/composeHandlers.test.ts` | 7 tests for provenance and hook_style in `buildComposeRequest()` |
| `dashboard/tests/unit/DraftMetadataSection.test.ts` | 6 tests for hook style badge rendering |
| `docs/roadmap/obsidian-ghostwriter-edge/native-workflow-polish.md` | Decision log (6 decisions) |
| `docs/roadmap/obsidian-ghostwriter-edge/session-07-handoff.md` | This file |

## Decisions Made

See `native-workflow-polish.md` for full decision log (6 decisions).

Key decisions:
1. **Source field encodes hook style inline** — `"assist:hook:contrarian_take"` rather than separate column
2. **Provenance copy-on-bridge** — links duplicated when content moves between entities
3. **Non-blocking provenance fetch** — doesn't delay Draft Studio editor hydration
4. **Optional node_id/chunk_id** — allows provenance without vault index
5. **Hook badge in metadata section** — compact accent badge next to "AI Assist"
6. **All four persist paths covered** — no provenance gaps

## Test Coverage

- **Rust:** 5 new provenance lifecycle tests (572 total pass)
- **Frontend:** 13 new tests across 2 files (686 total pass)
- **CI:** All gates green (fmt, clippy, svelte-check, vitest)

## What's Next (Session 8+)

- **Publish audit trail**: Show provenance on posted tweets in the timeline view
- **Analytics by hook style**: Aggregate engagement metrics by hook style to surface which hooks perform best
- **Provenance chain visualization**: Full lineage view from vault note → hook → draft → published tweet
- **Obsidian backlinks**: Deep-link from published tweets back to source notes in Obsidian
