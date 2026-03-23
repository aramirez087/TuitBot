# Ghostwriter Evidence UX Specification

## Design Goal

Semantic search should feel like a power tool inside the existing compose flow — not a parallel workflow. Evidence appears alongside graph suggestions and vault search, feeds into the same hook/angle/slot-refinement pipeline, and uses the same visual language (cards, badges, pin/dismiss actions).

---

## 1. Evidence Rail Placement

### Location

The Evidence Rail is a collapsible section inside the existing `InspectorContent` component, rendered between the Voice Context panel and the From Vault panel. It is always available when the composer is open, regardless of whether a vault selection session is active.

### Component Hierarchy

```
ComposerInspector
└── InspectorContent
    ├── SchedulePanel
    ├── VoiceContextPanel
    ├── EvidenceRail          ← NEW
    │   ├── EvidenceSearchBar
    │   ├── IndexStatusBadge
    │   ├── EvidenceCardList
    │   │   └── EvidenceCard (×N)
    │   └── AutoQueryToggle
    ├── FromVaultPanel (existing)
    │   ├── VaultSelectionReview
    │   │   ├── GraphSuggestionCards
    │   │   └── SlotTargetPanel
    │   └── VaultNoteList / HookPicker
    └── NotesPad
```

### Visibility Rules

| State | Evidence Rail Visible | Collapsed by Default |
|-------|----------------------|---------------------|
| No embedding config | Hidden entirely | — |
| Config present, index empty | Visible | Expanded (shows empty state) |
| Config present, index building | Visible | Expanded (shows progress) |
| Config present, index ready | Visible | Collapsed until user opens or auto-query fires |
| Selection session active | Visible | Collapsed (selection review takes priority) |

---

## 2. Evidence Rail Anatomy

### Header Row

```
┌─────────────────────────────────────────────┐
│ ◆ Evidence                    [●] [▾]       │
│                                              │
│ [●] = IndexStatusBadge (green/yellow/gray)  │
│ [▾] = Collapse/expand toggle                │
└─────────────────────────────────────────────┘
```

- **Label:** "Evidence" — uppercase, 12px, `--color-text-muted`, matching existing section labels
- **IndexStatusBadge:** 8px dot, color-coded by freshness state (see Section 6)
- **Collapse toggle:** Chevron, same pattern as existing collapsible sections

### Search Bar

```
┌─────────────────────────────────────────────┐
│ 🔍 Search your vault...                     │
└─────────────────────────────────────────────┘
```

- Same styling as `FromVaultPanel`'s `.vault-search-input`
- Placeholder: "Search your vault..."
- Debounce: 300ms (matching existing vault search)
- Keyboard shortcut: `Cmd+Shift+E` (Mac) / `Ctrl+Shift+E` (Win/Linux) to focus

### Result Cards

Each result is an `EvidenceCard`:

```
┌─────────────────────────────────────────────┐
│ ┌──────────┐                                │
│ │ Semantic │  "Distributed Systems > CAP"   │
│ └──────────┘                                │
│                                              │
│ "The CAP theorem states that a distributed  │
│ system can provide at most two of three..."  │
│                                              │
│ notes/distributed-systems.md                 │
│                                              │
│ [Pin] [Apply to slot ▾] [Dismiss]           │
└─────────────────────────────────────────────┘
```

**Card fields:**
- **Match reason badge:** Pill-shaped badge, color-coded:
  - `Semantic` — purple, `color-mix(in srgb, #8B5CF6 15%, transparent)`
  - `Graph` — blue, `color-mix(in srgb, var(--color-accent) 15%, transparent)`
  - `Keyword` — gray, `color-mix(in srgb, var(--color-text-subtle) 10%, transparent)`
  - `Hybrid` — gradient purple-blue
- **Heading path:** `heading_path` from chunk, truncated with ellipsis. 11px, `--color-accent`, font-weight 600.
- **Snippet:** First 120 characters of chunk text. 12px, `--color-text`, line-height 1.5.
- **Source path:** `relative_path` of parent note. 10px, `--color-text-subtle`. Hidden in Cloud mode.
- **Actions:** Pin, Apply to slot, Dismiss (see Section 5)

### Auto-Query Toggle

Below the search bar when the evidence rail is expanded:

```
┌─────────────────────────────────────────────┐
│ [✓] Auto-suggest while editing              │
└─────────────────────────────────────────────┘
```

- Toggle button, same styling as `synthesis-toggle` in `VaultSelectionReview`
- Default: off
- When on: semantic search fires automatically from draft text (see Section 4)

---

## 3. Search-Before-Generation Flow

This flow is for users who want to find relevant evidence **before** generating a draft.

### Steps

1. User opens the Evidence Rail (click header or `Cmd+Shift+E`)
2. Types a search query in the search bar
3. After 300ms debounce, a request fires:
   ```
   GET /api/vault/evidence?q={query}&limit=8&mode=hybrid
   ```
4. Results render as `EvidenceCard` list, sorted by blended score (descending)
5. User reviews results and takes actions:
   - **Pin** — Locks this evidence into the compose context. Pinned evidence persists across search queries and draft edits within the session.
   - **Dismiss** — Removes this result. It won't reappear for this session.
   - **Apply to slot** — Opens a slot picker (reuses `SlotTargetPanel` pattern) to apply this evidence to a specific thread block.
6. When the user generates a draft (via hooks, angles, or direct generate), pinned evidence is included in the LLM context alongside vault fragments.

### Pinned Evidence Section

When any results are pinned, a "Pinned" section appears above the search results:

```
┌─────────────────────────────────────────────┐
│ Pinned (2)                                   │
│ ┌───────────────────────────────────────┐    │
│ │ 📌 "CAP theorem..."   [Unpin] [Apply] │    │
│ └───────────────────────────────────────┘    │
│ ┌───────────────────────────────────────┐    │
│ │ 📌 "Raft consensus..." [Unpin] [Apply]│    │
│ └───────────────────────────────────────┘    │
│                                              │
│ Search results (5)                           │
│ ...                                          │
└─────────────────────────────────────────────┘
```

Pinned cards are compact (snippet only, no heading path) to save space.

---

## 4. Search-During-Editing Flow (Auto-Query)

This flow is for users who want evidence to surface **while they're editing** a draft.

### Behavior

1. User enables "Auto-suggest while editing" toggle
2. As the user types in the tweet textarea or a thread block:
   - After 800ms of no typing (debounce), extract the text of the focused block
   - Fire a semantic search using that text as the query:
     ```
     GET /api/vault/evidence?q={blockText}&limit=5&mode=semantic
     ```
   - Cancel any in-flight auto-query request (superseded by new text)
3. Results appear in the Evidence Rail with a "Suggested" badge (distinguished from manual search results):
   ```
   ┌──────────────────────────────────────────┐
   │ ┌───────────┐ ┌───────────┐              │
   │ │ Suggested │ │ Semantic  │ "CAP..."     │
   │ └───────────┘ └───────────┘              │
   │ ...                                       │
   └──────────────────────────────────────────┘
   ```
4. "Suggested" badge: 10px, `--color-text-subtle`, dotted border. Disappears if the user pins the result.
5. Auto-suggested results are cleared when:
   - The user types new text (replaced by new suggestions after debounce)
   - The user switches to a different thread block
   - The user disables auto-suggest

### Performance Constraints

- **Debounce:** 800ms after last keystroke
- **Cancel-on-new:** Previous in-flight request is aborted via `AbortController`
- **Loading skeleton:** While auto-query is in flight, show a shimmer skeleton (matching existing `vault-loading-shimmer` pattern)
- **No results state:** If auto-query returns 0 results, show nothing (don't display an empty state for auto-suggestions — it's distracting)

---

## 5. Result Actions

### Pin

- **Icon:** Pin icon (lucide `Pin`)
- **Behavior:** Moves the result to the "Pinned" section. Pinned results are included in the LLM context when generating or refining drafts.
- **Persistence:** Per-session only. Pinned results are cleared when the composer closes.
- **Limit:** Maximum 5 pinned results to avoid context bloat.
- **Visual:** Pinned card gets a left border accent (`2px solid var(--color-accent)`)

### Dismiss

- **Icon:** X icon (lucide `X`), same as existing dismiss patterns
- **Behavior:** Removes the result from the list. Dismissed chunk_ids are tracked per-session and excluded from future queries.
- **No undo:** Consistent with existing `GraphSuggestionCards` dismiss behavior.

### Apply to Slot

- **Icon:** Arrow-down-to-line icon (lucide `ArrowDownToLine`)
- **Behavior:** Opens a dropdown showing available thread slots (or "Tweet" for single-tweet mode). Selecting a slot calls the existing `handleSlotInsert()` pattern:
  1. Calls `api.assist.improve(slotText, evidenceContext)` where `evidenceContext` includes the evidence snippet and heading path
  2. Replaces the slot text with the improved version
  3. Pushes to the undo stack via `draftInsertStore`
  4. Adds provenance ref with `source_role: 'semantic_evidence'`
- **Availability:** Only shown when `hasExistingContent` is true (a draft exists to apply to)

### View Source

- **Icon:** External-link icon (lucide `ExternalLink`)
- **Behavior:** Opens the source note in Obsidian via `obsidian://open?vault={vault}&file={path}` deep link (Desktop only, hidden in Cloud mode)
- **Fallback:** When vault source path is not available, show note title as a tooltip only

---

## 6. Index Status Affordances

### IndexStatusBadge

An 8px colored dot in the Evidence Rail header, with tooltip on hover:

| State | Color | Tooltip |
|-------|-------|---------|
| Fresh (≥95%) | `#22C55E` (green) | "Index up to date — 1200 of 1234 chunks indexed" |
| Indexing (50-94%) | `#F59E0B` (amber), pulsing | "Indexing... 800 of 1234 chunks (65%)" |
| Stale (<50%) | `#EF4444` (red) | "Index stale — 600 of 1234 chunks need re-indexing" |
| Empty | `#9CA3AF` (gray) | "No index — search by keyword only" |
| Error | `#EF4444` (red), static | "Embedding provider unavailable" |

### Expanded Status (optional)

When the user clicks the status badge, a status popover shows:

```
┌─────────────────────────────────────────────┐
│ Semantic Index Status                        │
│                                              │
│ Indexed:  1200 / 1234 chunks (97%)          │
│ Model:    text-embedding-3-small             │
│ Updated:  2 minutes ago                      │
│ Provider: OpenAI (healthy)                   │
│                                              │
│ [Reindex Now]                                │
└─────────────────────────────────────────────┘
```

"Reindex Now" triggers a full re-embedding of all chunks. Shows confirmation first: "This will re-embed all 1234 chunks. Continue?"

---

## 7. Empty States

### No Embedding Config

The Evidence Rail is hidden entirely. No UI element rendered. The user sees the existing compose flow unchanged.

### Config Present, Index Empty

```
┌─────────────────────────────────────────────┐
│ ◆ Evidence                         [○]      │
│                                              │
│       📊                                     │
│  Building your semantic index...             │
│  This happens automatically in the           │
│  background. Search is available once         │
│  indexing completes.                          │
│                                              │
│  0 of 1234 chunks indexed                    │
│  ████░░░░░░░░░░░░ 0%                        │
└─────────────────────────────────────────────┘
```

### Config Present, Provider Unavailable

```
┌─────────────────────────────────────────────┐
│ ◆ Evidence                         [●]      │
│                                              │
│ 🔍 Search your vault...                     │
│                                              │
│ ⚠ Semantic search unavailable.              │
│   Results below use keyword matching only.   │
│                                              │
│ [Results from keyword search...]             │
└─────────────────────────────────────────────┘
```

The search bar is still functional — it falls back to keyword+graph results. The warning is subtle (11px, `--color-text-subtle`), not blocking.

### No Results

```
┌─────────────────────────────────────────────┐
│ No matching evidence found.                  │
│ Try different search terms or check that     │
│ your vault has indexed content.              │
└─────────────────────────────────────────────┘
```

For auto-query no-results: nothing is shown (silent, no empty state).

---

## 8. Degraded States

### Embedding Provider Down

- Evidence Rail switches to keyword+graph-only mode
- Subtle warning: "Semantic search unavailable"
- Search bar still works — falls back to `retrieve_vault_fragments()` keyword path
- No error modal, no blocking UI
- Auto-query stops firing (no point querying with keyword for every keystroke)

### Index Stale (< 50% fresh)

- Yellow badge in header
- Warning below search bar: "Index may show outdated results"
- Search still works — returns results from whatever is indexed
- No blocking behavior

### Watchtower Not Running

- Evidence Rail shows whatever was last indexed
- Status badge shows last-indexed timestamp
- No active indexing indicator

---

## 9. Interaction with Existing Components

### Graph Suggestion Cards

When both graph neighbors and semantic results exist for the same compose session:
- Graph neighbors appear in `VaultSelectionReview` → `GraphSuggestionCards` (existing flow, unchanged)
- Semantic results appear in the Evidence Rail (new, separate section)
- If a result appears in both (same `chunk_id`), the graph version takes precedence — the semantic duplicate is hidden from the evidence rail
- Deduplication happens client-side by `chunk_id`

### FromVaultPanel Manual Search

The existing manual vault search in `FromVaultPanel` is **unchanged**. It continues to search by note title. The Evidence Rail's search is a separate, chunk-level semantic search. They coexist:
- FromVaultPanel: "Find a specific note by title" → select chunks → generate hooks
- Evidence Rail: "Find relevant evidence by concept" → pin results → enrich context

### Hook / Angle Generation

When generating hooks or angles, the LLM prompt context includes:
1. Primary selection text (existing)
2. Accepted graph neighbor fragments (existing)
3. **Pinned evidence fragments (new)** — appended after neighbor fragments, within the `MAX_FRAGMENT_CHARS` budget

Pinned evidence uses the same `format_fragments_prompt()` formatting, with a `[E1]`, `[E2]` citation prefix to distinguish from vault citations `[1]`, `[2]`.

### Provenance

Evidence-sourced fragments add `ProvenanceRef` entries with:
```typescript
{
  node_id: number,
  edge_type: "semantic",
  edge_label: "Similar content (0.87 similarity)",
  source_role: "semantic_evidence",
  // Optional: which search produced this
  similarity_score?: number,
  match_reason?: "semantic" | "keyword" | "hybrid"
}
```

---

## 10. Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+E` / `Ctrl+Shift+E` | Toggle Evidence Rail open/closed |
| `Cmd+Shift+F` / `Ctrl+Shift+F` | Focus Evidence search bar (opens rail if closed) |
| `Escape` (while search focused) | Clear search, close rail |
| `Enter` (on result) | Pin the result |
| `Delete` / `Backspace` (on focused result) | Dismiss the result |
| `Tab` / `Shift+Tab` | Navigate between evidence cards |

---

## 11. Responsive Behavior

### Mobile (< 640px)

- Evidence Rail renders inside the mobile drawer (existing `inspector-drawer` pattern)
- Cards stack vertically, full width
- "Apply to slot" action uses a full-width bottom sheet instead of dropdown
- Auto-query toggle hidden on mobile (typing + auto-query is jarring on small screens)
- Search bar uses 16px font (matching existing mobile vault search)

### Desktop

- Evidence Rail renders inline in the inspector sidebar
- Cards have fixed max-width matching existing inspector content
- "Apply to slot" uses a compact dropdown

---

## 12. Analytics Events

| Event | Properties | When |
|-------|-----------|------|
| `evidence_rail_opened` | `session_id`, `has_selection` | User expands evidence rail |
| `evidence_search_executed` | `query_length`, `result_count`, `mode` (semantic/keyword/hybrid) | Search fires (manual or auto) |
| `evidence_pinned` | `chunk_id`, `match_reason`, `similarity_score` | User pins a result |
| `evidence_dismissed` | `chunk_id`, `match_reason` | User dismisses a result |
| `evidence_applied_to_slot` | `chunk_id`, `slot_index`, `slot_label`, `match_reason` | User applies evidence to a thread slot |
| `evidence_auto_query_toggled` | `enabled` | User toggles auto-suggest |
| `evidence_contributed_to_draft` | `pinned_count`, `applied_count`, `session_id` | Draft generated with evidence context |
