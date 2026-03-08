# Obsidian Vault to Post Loop — UX Blueprint

## Onboarding (SourcesStep.svelte)

No changes needed. Current onboarding UX is adequate:

- **Desktop:** "Obsidian Vault / Notes Folder" label, native Browse button,
  Watch + Loop Back toggles.
- **SelfHost:** Google Drive primary, local folder collapsed under "Advanced".
- **Cloud:** Google Drive only.

The deployment-aware content source setup epic already handles mode-specific
defaults via `preferred_source_default`.

## Settings (ContentSourcesSection.svelte) — Extend

### Vault Health Summary (inline)

Add a compact health summary below each configured source:

```
┌──────────────────────────────────────────────┐
│ Obsidian Vault                    [Re-scan]  │
│ ~/notes/marketing                            │
│                                              │
│ ● Synced · 12 notes · 47 fragments · 23 seeds│
│ Last sync: 2 minutes ago                     │
│                                              │
│ [Watch for changes: ON] [Loop back: ON]      │
└──────────────────────────────────────────────┘
```

States:
- **Green dot + "Synced"** — source status is `active`, sync within last hour.
- **Yellow dot + "Syncing..."** — source status is `syncing`.
- **Red dot + error message** — source status is `error`, show `error_message`.
- **Gray dot + "Disabled"** — source status is `disabled`.

"Re-scan Now" button calls `POST /api/ingest` with `force=true` and
shows a spinner until the next sync completes.

Desktop mode: if vault path is invalid (directory doesn't exist), show a
warning icon with "Vault path not found — check your settings".

### Data source

API call: `GET /api/vault/health` returns per-source stats.

## Vault Health Page (NEW — `/vault`)

Full dashboard page for vault inspection and debugging.

### Layout

```
┌─────────────────────────────────────────────────────┐
│ Vault                                               │
├─────────────────────────────────────────────────────┤
│ [Source Cards Row]                                   │
│ ┌─────────────┐ ┌─────────────┐                    │
│ │ Local Vault  │ │ Google Drive│                    │
│ │ ● Synced     │ │ ● Synced    │                    │
│ │ 12 notes     │ │ 3 notes     │                    │
│ │ 47 fragments │ │ 8 fragments │                    │
│ │ 23 seeds     │ │ 5 seeds     │                    │
│ └─────────────┘ └─────────────┘                    │
├─────────────────────────────────────────────────────┤
│ Notes                                    [Search]   │
│ ┌───────────────────────────────────────────────┐   │
│ │ ▶ Marketing Playbook      8 frags · 5 seeds  │   │
│ │ ▶ Product Launch Notes    3 frags · 2 seeds  │   │
│ │ ▶ Weekly Reflections      12 frags · 8 seeds │   │
│ │   ...                                         │   │
│ └───────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│ [Expanded Note: Marketing Playbook]                 │
│ Fragments:                                          │
│   ## Key Insight          boost: 2.3  [used 3×]    │
│   ## Competitor Analysis  boost: 1.0  [unused]     │
│   ## Launch Strategy      boost: 1.5  [used 1×]    │
│                                                     │
│ Seeds:                                              │
│   "Most startups underestimate..." [pending]       │
│   "The data shows that..." [used 2026-03-01]       │
└─────────────────────────────────────────────────────┘
```

### Interaction Model

- **Source cards:** Click to filter notes by source. Click again to clear
  filter.
- **Note list:** Click to expand/collapse. Shows fragment count and seed count.
  Supports text search filtering.
- **Fragment view:** Shows heading path, chunk preview (first 120 chars),
  retrieval_boost score, and usage count (how many times referenced in
  `source_chunks_json`).
- **Seed list:** Shows seed text, archetype, engagement_weight, status
  (pending/used), and used_at timestamp.

### Keyboard Navigation

- `j` / `k` — move focus down/up in note list.
- `Enter` — expand/collapse focused note.
- `Tab` — cycle between source cards, note list, and detail panels.
- `/` — focus search input.

### Mobile Responsive

Single-column layout. Source cards stack vertically. Note list takes full
width. Fragment and seed details appear inline below the expanded note
(no side panel).

### Data Sources

- `GET /api/vault/health` — per-source aggregate stats.
- `GET /api/vault/notes?source_id=N&status=S&limit=50&offset=0` — paginated
  note list.
- `GET /api/vault/fragments?node_id=N` — fragments for a specific note.
- `GET /api/vault/seeds?status=pending&limit=50` — seed browser.

## From Vault in Composer (NEW)

### Concept

Add a "From Vault" toggle to the composer that lets users explicitly select
notes and fragments to ground generation in.

### Layout

```
┌─────────────────────────────────────────────────────┐
│ Compose                                             │
│                                                     │
│ Topic: [________________________]                   │
│                                                     │
│ [Generate Tweet] [Generate Thread] [Improve]        │
│                                                     │
│ ○ Auto (vault context applied automatically)        │
│ ● From Vault                                        │
│   ┌─────────────────────────────────────────────┐   │
│   │ Search notes... [_______________]            │   │
│   │                                              │   │
│   │ ☑ Marketing Playbook                        │   │
│   │   ☑ ## Key Insight                          │   │
│   │   ☐ ## Competitor Analysis                  │   │
│   │   ☐ ## Launch Strategy                      │   │
│   │                                              │   │
│   │ ☐ Product Launch Notes                      │   │
│   │   ...                                        │   │
│   │                                              │   │
│   │ Selected: 1 fragment (max 3)                 │   │
│   └─────────────────────────────────────────────┘   │
│                                                     │
│ [Generate with Selected Context]                    │
└─────────────────────────────────────────────────────┘
```

### Behavior

- Default state: "Auto" radio selected — automatic RAG continues as today.
- When "From Vault" is selected:
  - Note list appears (fetched from `GET /api/vault/notes`).
  - Search input filters notes by title.
  - Clicking a note expands its fragment list (fetched from
    `GET /api/vault/fragments?node_id=N`).
  - User checks up to 3 fragments.
  - "Selected: N fragments (max 3)" counter updates.
  - Generation buttons change to "Generate with Selected Context".
- Request body gains optional `fragment_ids: number[]`.
- Backend fetches chunk text for selected IDs, uses as explicit RAG context
  (skips automatic retrieval).
- Response includes `citations` array for selected fragments.

### Constraints

- Maximum 3 fragments per request (keeps prompt within `RAG_MAX_CHARS`).
- Fragment checkbox disabled when 3 are already selected.
- If no fragments are selected and "From Vault" is active, show hint:
  "Select at least one fragment to generate with vault context."

## Reply Assistance (Discovery Feed) — Extend

### Current State

`compose_reply` in `discovery.rs` calls `generate_reply` without vault
context. The reply is generated from the tweet content and business profile
only.

### Change

- `compose_reply` calls `resolve_composer_rag_context()` to get vault RAG
  context.
- Passes context to `generate_reply_with_context()`.
- Response gains optional `citations` array.

### UX

- Discovery card shows a small "Vault" badge when the reply was generated
  with RAG context (i.e., `citations` array is non-empty).
- Clicking the badge expands a citation list below the generated reply.
- Badge is a subtle indicator (e.g., small book icon) — not intrusive.

## Citation Display (CitationPill.svelte)

### Component

Reusable component for displaying citations below generated text.

```
┌──────────────────────────────────────────┐
│ Based on:                                │
│ [📄 Marketing Playbook > Key Insight]    │
│ [📄 Launch Notes > Timeline]             │
└──────────────────────────────────────────┘
```

### Behavior

- Each citation is a clickable pill.
- **Click to expand:** Shows chunk preview text (first 200 chars).
- **Desktop + local_fs source:** Click opens `obsidian://open?vault={name}&file={path}` URI.
  Pill shows a small external-link icon to indicate it will open Obsidian.
- **Other modes:** Pill is read-only (expand for preview only).

### Obsidian URI Construction

```
obsidian://open?vault={vault_name}&file={relative_path}
```

- `vault_name`: last directory component of the source's configured path
  (e.g., `/Users/alice/notes/marketing` → `marketing`).
- `relative_path`: the note's `relative_path` from `content_nodes`, URL-encoded.
- Only generated when `deployment_mode === Desktop` and source type is `local_fs`.

### Accessibility

- Pills are keyboard-focusable (`tabindex="0"`).
- `Enter` or `Space` toggles expand/collapse.
- `aria-label` includes full citation text.
- Screen reader: "Citation: Marketing Playbook, section Key Insight".

## Navigation

Add "Vault" to the main sidebar navigation in `(app)/+layout.svelte`:

```
Dashboard
Compose
Discovery
Queue
Vault        ← NEW
Settings
```

Position: after Queue, before Settings. Icon: a book or vault icon.

## Error States

### Vault Not Configured

If no sources are configured, vault health page shows:

```
No content sources configured.
[Set up a vault in Settings →]
```

Link goes to `/settings` and scrolls to content sources section.

### Sync Error

If a source is in error state, vault health page shows the error inline on
the source card with a "Retry" button.

### Empty Vault

If sources are configured but no notes are ingested yet:

```
Watching ~/notes/marketing for changes...
No notes found yet. Add markdown files to your vault to get started.
```

### No Fragments for "From Vault"

If user clicks "From Vault" but no notes have fragments:

```
No fragments available. Notes are being processed — check back shortly.
```
