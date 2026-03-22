# End-to-End UX Journey: Backlink Synthesizer

This document defines every user-facing state in the Ghostwriter flow with graph-aware related-note suggestions, from Obsidian selection through draft review.

## State Machine

```
                    ┌─────────────┐
                    │  Selection   │ (Obsidian)
                    └──────┬──────┘
                           │ POST /api/vault/send-selection
                           ▼
                    ┌─────────────┐
                    │   Loading    │ (Dashboard)
                    └──────┬──────┘
                           │ resolve_selection_rag_context()
                           │ expand_graph_neighbors()
                           ▼
                    ┌──────┴──────┐
                    │             │
              ┌─────▼─────┐ ┌────▼──────────┐
              │ Empty Graph│ │ Suggestions   │
              │ (fallback) │ │ Available     │
              └─────┬─────┘ └────┬──────────┘
                    │            │ user accepts/dismisses
                    │            ▼
                    │     ┌─────────────┐
                    │     │  Partial     │
                    │     │  Accept      │
                    │     └──────┬──────┘
                    │            │
                    └─────┬──────┘
                          │ "Generate hooks"
                          ▼
                   ┌─────────────┐
                   │ Hooks Gen   │
                   └──────┬──────┘
                          │ pick hook → thread
                          ▼
                   ┌─────────────┐
                   │ Thread Slot │
                   │ Insertion   │
                   └──────┬──────┘
                          │
                          ▼
                   ┌─────────────┐
                   │ Draft Review│
                   └──────┬──────┘
                          │ click citation
                          ▼
                   ┌─────────────┐
                   │ Return to   │
                   │ Source      │
                   └─────────────┘
```

## States

### 1. Selection (Obsidian)

**Trigger:** User selects text in Obsidian, invokes "Send to TuitBot" command.

**What the user sees:**
- Obsidian notice: "Sent to TuitBot" with a clickable link to the composer
- The notice persists for 5 seconds

**What happens:**
- Plugin sends `GhostwriterPayload` to `POST /api/vault/send-selection`
- Server stores selection in `vault_selections` with 30-min TTL
- Server resolves `node_id` and `chunk_id` (best-effort)
- WebSocket event `SelectionReceived` emitted to dashboard

**No changes to this state.** Existing behavior preserved.

### 2. Loading (Dashboard)

**Trigger:** Dashboard opens `/compose?selection={session_id}` (via WebSocket notification or manual navigation).

**What the user sees:**
- Selected text preview (the text they highlighted in Obsidian)
- Note title and file path displayed above the preview
- Below the preview: a skeleton loading area with text "Finding related notes..."
- Spinner animation in the skeleton area

**What happens:**
- Dashboard fetches `GET /api/vault/selection/{session_id}` for the selection data
- Dashboard calls `GET /api/vault/notes/{node_id}/neighbors` to expand the graph
- Both requests run in parallel

**Duration:** Typically < 500ms for graph expansion (single SQL query). Loading state is brief.

### 3. Empty Graph (Fallback)

**Trigger:** The selected note has zero edges — no wikilinks to indexed notes, no shared tags.

**What the user sees:**
- The suggestion panel does not appear
- Below the selected text preview, a subtle info label:
  > "This note doesn't link to other indexed notes. You can still generate from this selection alone."
- The "Generate hooks" button is immediately available
- Standard flow (today's behavior) proceeds unchanged

**UX copy:**
- Info label: "This note doesn't link to other indexed notes. You can still generate from this selection alone."

**Privacy note:** This label does not reveal which notes exist in the vault, only that none are linked to the selected note.

### 4. Suggestions Available

**Trigger:** Graph expansion found 1 or more related notes.

**What the user sees:**
- A "Related Notes" panel appears below the selected text preview
- Panel header: "Related notes from your vault" with a count badge (e.g., "3 found")
- Each related note is a suggestion card containing:
  - **Note title** (or file path if untitled)
  - **Reason badge** — a colored label explaining the connection:
    - "Linked from your note" (wikilink outgoing)
    - "Links to your note" (backlink incoming)
    - "Shares tag #topic-name" (shared tag)
    - "Both linked from [Note Title]" (co-citation, future v2)
  - **Snippet preview** — first 120 characters of the best chunk
  - **Accept button** — "Include" (primary style)
  - **Dismiss button** — "Skip" (ghost/secondary style)
- Cards are ordered by composite ranking score (highest first)
- Maximum 8 cards displayed

**UX copy:**
- Panel header: "Related notes from your vault"
- Accept button: "Include"
- Accept tooltip: "Include insights from this note in your draft."
- Dismiss button: "Skip"
- Dismiss tooltip: "This note won't be used for this draft. You can bring it back later."
- Reason badges:
  - Wikilink outgoing: "Linked from your note"
  - Backlink incoming: "Links to your note"
  - Shared tag: "Shares tag #[tag-name]"

### 5. Partial Accept

**Trigger:** User has accepted some suggestion cards and dismissed others.

**What the user sees:**
- Accepted cards collapse into compact "included notes" chips above the "Generate hooks" button. Each chip shows the note title and a small "x" to un-include.
- Dismissed cards fade out with a brief animation (200ms). They move to a collapsed "Skipped" section at the bottom of the panel.
- The "Skipped" section is collapsed by default, with a toggle: "Show skipped (N)"
- Any remaining suggestion cards that haven't been acted on stay in their original position
- The "Generate hooks" button becomes enabled once at least one card is accepted (or the user proceeds with selected note only)

**State transitions:**
- Clicking "x" on an included chip moves the note back to a suggestion card (un-accept)
- Expanding "Show skipped" and clicking "Undo" on a skipped card restores it as a suggestion card
- "Undo skip" is available indefinitely within the session (no 30s timer — simpler)

**UX copy:**
- Included chip tooltip: "Click to remove from draft context"
- Skipped toggle: "Show skipped (N)"
- Undo button on skipped card: "Undo"

### 6. Hooks Generation

**Trigger:** User clicks "Generate hooks" with the selected note and any accepted related notes.

**What the user sees:**
- Loading state: "Generating hooks from your notes..."
- Hooks are generated with context from the selected note + all accepted related notes
- Each hook displays as today, with an addition: a small citation chip below each hook showing which source note(s) contributed
- Citation chips show note title, abbreviated

**What happens:**
- Server receives hook generation request with `selected_node_ids` (original + accepted neighbors)
- `build_draft_context_with_selection()` is called with the expanded node ID set
- Vault fragments are drawn from all included notes
- Citations are built for all contributing chunks

**UX copy:**
- Loading: "Generating hooks from your notes..."
- Citation chip format: "From: [Note Title]"

### 7. Thread-Slot Insertion

**Trigger:** User picks a hook and initiates thread generation.

**What the user sees:**
- Thread slots are generated as today
- Each slot may draw from a different accepted note's chunks
- Below each thread slot, a small citation chip shows which source note contributed to that slot's content
- Citation chips are clickable (see State 10: Return to Source)

**What happens:**
- Thread generation receives the full `DraftContext` including fragments from all accepted notes
- The LLM may draw on different fragments for different slots
- Per-slot provenance is recorded: `vault_provenance_links` rows are created for each slot with the contributing `node_id`, `chunk_id`, `edge_type`, and `edge_label`

**UX copy:**
- Citation chip: "[Note Title] · [Heading]"
- If multiple notes contributed to one slot: "[Note A], [Note B]"

### 8. Draft Review

**Trigger:** Thread/tweet generation is complete and the user reviews the output.

**What the user sees:**
- The generated content (tweet or thread) with inline citation chips
- A "Provenance" panel (existing, extended) showing all contributing notes with:
  - Note title and file path
  - Edge type reason: "Linked from your note", "Shares tag #topic"
  - Heading path of the contributing chunk
  - 120-char snippet excerpt
- "Return to source" deep-link on each provenance entry (Desktop only)

**What happens:**
- Provenance data includes the new `edge_type` and `edge_label` fields
- API response includes `vault_citations` with full graph provenance

### 9. Dismissal Recovery

**Trigger:** User wants to re-include a previously dismissed note.

**What the user sees:**
- In the compose view (before hook generation), the "Show skipped (N)" toggle at the bottom of the suggestions panel
- Expanding it reveals dismissed cards in a muted visual style
- Each dismissed card has an "Undo" button
- Clicking "Undo" restores the card to the active suggestion list

**Behavior:**
- Dismissed state is session-scoped — refreshing the page resets dismissals
- Dismissed notes are excluded from `selected_node_ids` when generating hooks
- There is no permanent block list

### 10. Return to Source

**Trigger:** User clicks a citation chip or "Return to source" link.

**What the user sees:**
- **Desktop (Tauri):** Opens the note in Obsidian via deep link: `obsidian://open?vault={vault_name}&file={file_path}`
  - If a heading is available: `obsidian://open?vault={vault_name}&file={file_path}&heading={heading}`
- **Web (browser):** Shows the file path and heading as a copyable reference (no Obsidian deep link available)
- **Cloud mode:** Citation chips show source path and heading but no deep link (privacy: vault name not exposed)

**UX copy:**
- Desktop tooltip: "Open in Obsidian"
- Web tooltip: "Copy reference path"
- Cloud tooltip: "Source: [file_path]"

## Error States

### Graph Expansion Failure

**Trigger:** `GET /api/vault/notes/{node_id}/neighbors` returns an error.

**What the user sees:**
- The suggestion panel does not appear
- A subtle warning: "Couldn't load related notes. Generating from this selection alone."
- Standard flow proceeds (fail-open)

### Selection Expired

**Trigger:** 30-min TTL has elapsed on the vault selection.

**What the user sees:**
- "This selection has expired. Please send a new selection from Obsidian."
- No compose UI is shown

### Node Not Indexed

**Trigger:** The selected note's file path doesn't match any `content_nodes` row.

**What the user sees:**
- Standard flow proceeds with `selected_text` as direct context (existing behavior)
- Info label: "This note hasn't been indexed yet. Generating from your selected text."

## Privacy-Aware Rendering

| Element | Desktop | Self-host | Cloud |
|---------|---------|-----------|-------|
| Selected text preview | Shown | Shown | Hidden |
| Related note titles | Shown | Shown | Shown (titles are metadata) |
| Reason badges | Shown | Shown | Shown |
| Snippet previews (120 char) | Shown | Shown | Shown |
| Return-to-source deep link | Obsidian link | Obsidian link | Disabled |
| Vault name in deep link | Included | Included | Omitted |
