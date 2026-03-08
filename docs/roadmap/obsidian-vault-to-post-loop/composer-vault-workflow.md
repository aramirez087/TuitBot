# Composer "From Vault" Workflow

## Overview

The composer inspector now supports two content-sourcing modes alongside the existing AI generation:

1. **From Notes** — Paste rough notes or outlines; AI expands them into tweets or threads.
2. **From Vault** — Search indexed vault notes, pick up to 3 sections, and generate content grounded in that material.

Both modes coexist as peer buttons in the AI section of the inspector. Only one panel is open at a time.

## Interaction Model

### Opening the Vault Panel

- Click "From vault" in the inspector AI section
- Use Command Palette (`Cmd+K`) → "From vault"
- The panel replaces the notes panel if it was open

### Search and Selection

1. The panel loads recent notes on mount (empty query).
2. The search input auto-focuses and accepts free-text queries.
3. Searches are debounced (300ms) and query note titles via `api.vault.searchNotes`.
4. Clicking a note row expands it to show its indexed chunks (sections).
5. Each chunk displays its heading path and a snippet preview.
6. Checkboxes allow selecting up to **3 chunks** per generation.
7. Additional checkboxes are disabled when the limit is reached.

### Generation

1. Clicking "Generate tweet/thread from vault" triggers content generation.
2. If the editor already has content, a replace confirmation appears first.
3. The selected notes' `node_id` values are passed to `api.assist.tweet` or `api.assist.thread`.
4. The backend uses these IDs to retrieve relevant vault context for generation.
5. The response includes `vault_citations` that are stored in composer state.

### Citation Display

After generation from vault material, **citation chips** appear below the editor:

- Each chip shows a compact label: `NoteTitle › SectionHeading`
- Clicking a chip expands it to show the full heading path and snippet
- Each chip has a remove button (×) to discard individual citations
- Citations are labeled with "Based on:" prefix
- Citations persist through editing until the user removes them or generates new content

### Provenance

When the user submits/schedules the composed content:

- If vault citations exist, they are included as `provenance` refs in the compose request
- Each provenance ref includes: `node_id`, `chunk_id`, `source_path`, `heading_path`, `snippet`
- This enables end-to-end traceability from vault note to published post

### Undo Behavior

- Every generation (from notes or vault) snapshots the current editor state including citations
- The undo banner appears for 10 seconds after generation
- Undoing restores the previous text, thread blocks, and vault citations
- Generating from notes clears any existing vault citations (different provenance source)

### Keyboard Access

| Action | Key |
|--------|-----|
| Open command palette | `Cmd+K` |
| Close panel | `Escape` |
| Search notes | Type in search input (auto-focused) |
| Expand/collapse note | `Enter` or `Space` on note row |
| Toggle chunk selection | `Space` on checkbox |
| Navigate | `Tab` through interactive elements |

### Mobile Behavior

- All touch targets are ≥44px on coarse pointer devices
- Search input uses 16px font to prevent iOS zoom
- Note list scrolls within a max-height container (200px)
- The vault panel renders inside the mobile drawer (ComposerInspector)

## Draft Metadata

When vault-sourced content is submitted, the draft includes:

```typescript
{
  content_type: 'tweet' | 'thread',
  content: '...',
  provenance: [
    {
      node_id: 42,
      chunk_id: 101,
      source_path: 'notes/marketing/launch-plan.md',
      heading_path: 'Launch Plan > Phase 1 > Messaging',
      snippet: 'Focus on developer experience and...'
    }
  ]
}
```

## Empty States

- **No vault sources configured**: Shows a message directing users to Settings
- **No search results**: Shows "No notes match your search"
- **No indexed sections**: Shows "No indexed sections" when a note has no chunks
- **No selections**: Generate button is disabled with "0 of 3 selected" counter

## Component Architecture

```
ComposeWorkspace.svelte
├── ComposerCanvas
│   ├── TweetEditor / ThreadFlowLane
│   ├── ComposerInsertBar
│   ├── CitationChips (new) ← shows after vault generation
│   └── Undo banner
├── InspectorContent
│   ├── TimePicker
│   ├── VoiceContextPanel
│   ├── AI section (From notes / From vault buttons)
│   ├── FromNotesPanel (conditional)
│   └── FromVaultPanel (new, conditional)
└── CommandPalette (includes "From vault" action)
```
