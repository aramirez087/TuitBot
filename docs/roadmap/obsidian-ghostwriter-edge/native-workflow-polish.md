# Native Workflow Polish: Provenance & Hook Propagation

## Decision Log

### Decision 1: Source field encodes hook style inline

**Choice:** Encode chosen hook style into the existing `source` column as `"assist:hook:<style>"` rather than adding a separate `hook_style` column to every content table.

**Why:** The `source` field already exists on drafts, approval items, and scheduled content. Adding a dedicated column would require migrations across multiple tables, FK relationships, and enum management. The tagged format is easy to parse with a regex (`^assist:hook:(\w+)$`) and carries through the full lifecycle without schema changes.

**Trade-off:** Parsing cost is negligible. If the number of source variants grows significantly, a dedicated column or lookup table may be warranted — but for the 5 current hook styles this is proportionate.

### Decision 2: Provenance copy-on-bridge pattern

**Choice:** When content moves between entities (e.g., approval queue → scheduled content), provenance links are copied to the new entity using `provenance::copy_links_for()`.

**Why:** The `vault_provenance_links` table uses `(entity_type, entity_id)` polymorphic keys. When an approval item is approved and a scheduled_content row is created, the approval row's provenance links must be duplicated to the scheduled_content entity so downstream consumers (publish audit, analytics) can trace the content back to source notes.

**Trade-off:** This creates duplicate rows in `vault_provenance_links`. An alternative would be a chain-of-custody pointer (scheduled_content → approval_queue → draft), but that requires multi-hop queries and couples entity lifecycles. Flat copies are simpler and faster to query.

### Decision 3: Non-blocking provenance fetch in Draft Studio

**Choice:** The provenance links for a selected draft are fetched in a non-blocking `.then()` after the main draft fetch, not awaited in the critical path.

**Why:** Provenance display (CitationChips) is supplementary metadata — it should not delay the editor hydration. If the provenance fetch fails, the draft is still fully usable; the sources section simply doesn't render.

### Decision 4: ProvenanceRef uses optional node_id and chunk_id

**Choice:** Both `node_id` and `chunk_id` are `Option<i64>` in `ProvenanceRef`, not required.

**Why:** The `vault_provenance_links` table has FK constraints to `content_nodes(id)` and `content_chunks(id)`. During compose, the caller may only know the `source_path` (e.g., from an Obsidian selection) without a matching node/chunk row. Making both optional allows provenance to be recorded even when the vault index hasn't ingested the specific file yet.

### Decision 5: Hook style badge in DraftMetadataSection

**Choice:** Display the hook style as a small accent badge next to "AI Assist" in the metadata section, using the existing `getStyleLabel()` mapping from `hookStyles.ts`.

**Why:** This gives immediate visual feedback about which hook pattern was used without cluttering the UI. The badge reuses the accent color and compact typography already established for the content-type badge.

### Decision 6: Provenance inserted at all four persist paths

**Choice:** Provenance links are inserted at every point where content is persisted: (1) direct schedule from compose, (2) approval queue from compose, (3) approval → scheduled bridge, (4) draft duplication.

**Why:** Each path creates a new entity that should carry its source lineage. Missing any path would create provenance gaps where content appears to have no sources despite being vault-generated.
