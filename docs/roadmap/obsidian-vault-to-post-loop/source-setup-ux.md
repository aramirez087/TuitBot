# Source Setup UX — Knowledge Vault

> Describes the onboarding step, settings section, and inline health summary for the Knowledge Vault feature.

## Terminology

Throughout the UI we use **"Knowledge Vault"** (or just "Vault") instead of the old "Content Sources" label. This reflects the product intent: notes are split into fragments, used as RAG context during generation, cited in outputs, and (in future) receive performance data back. Users see "vault" and "notes" language, never generic "content source" or "file sync" language.

## Onboarding Step

| Aspect | Value |
|--------|-------|
| Step label | **Vault** (was "Sources") |
| Header | **Knowledge Vault** (was "Content Source (Optional)") |
| Skippable | Yes — step always returns `true` from `canAdvance()`. A "skip" hint reads: *"You can skip this step and set it up later in Settings."* |

### Copy guidelines

- Describe what the vault does: "split your notes into fragments, use them as context when generating tweets and replies, and track which notes contributed to each post."
- Google Drive Folder ID is labelled **without** "(optional)". The hint reads: *"Required. Find the folder ID in your Google Drive URL after `/folders/`."*
- Watch toggle hints are mode-aware: local says "Re-index automatically when local files are added or modified"; Drive says "Polls your Drive folder at the configured interval."
- Loop-back toggle (local only, defaults off): *"Write tweet performance data back into note frontmatter (local vaults only). Currently tracks which notes were used — file write-back coming soon."*

### Mode-aware behavior

| Mode | Source selector | Local path | Google Drive | Notes |
|------|----------------|-----------|-------------|-------|
| Desktop | Dropdown (local default) | Browse + text input | Connect card | Full feature set |
| Self-host | Dropdown (Drive default) | Behind "Advanced" toggle | Connect card | Local path is manual text input |
| Cloud | Hidden (forced Drive) | Not available | Connect card + info notice | No local sources |

## Settings Section

| Aspect | Value |
|--------|-------|
| Section title | **Knowledge Vault** (was "Content Sources") |
| Description | *"Your vault feeds notes into generation as context and tracks what performs"* |
| Section icon | `FolderOpen` for local, `Cloud` for Drive |

### Inline health summary

Appears at the top of the section when at least one vault source is configured. Shows:

```
● Synced · 12 notes
Last synced 2m ago                    [Re-scan]
```

#### Status states

| Status | Dot color | Label | Re-scan | Notes |
|--------|-----------|-------|---------|-------|
| `active` | Green | Synced | Shown (local_fs only) | Normal operating state |
| `syncing` | Yellow | Syncing... | Hidden | In-progress scan |
| `error` | Red | Error | Shown (local_fs only) | Error message shown below |
| `disabled` | Gray | Disabled | Hidden | Source exists but disabled |
| (no source) | — | — | — | Shows "Not configured" notice instead |

#### Re-scan button

- Only shown for `local_fs` sources (Google Drive relies on its poll cycle).
- Calls `POST /api/sources/{id}/reindex`.
- Shows spinner while the rescan runs; polls `GET /api/vault/sources` every 3s to detect completion.

### Google Drive Folder ID

- Label: **Google Drive Folder ID** (no "(optional)").
- Hint: *"Required. Find the folder ID in your Google Drive URL after /folders/."*
- Warning shown when empty: *"A folder ID is required for Google Drive syncing to work."*
- No frontend validation gate — saves still work, but the server will fail to sync without it.

### Loop-back toggle

- Hint: *"Write tweet performance data back into note frontmatter. Currently tracks which notes were used — file write-back coming soon."*
- Defaults to **off** in onboarding (changed from on). The provenance tracking pipeline works end-to-end, but the actual file-write step (writing performance data into frontmatter) is not yet implemented.

## Data flow

```
onboarding.ts (store)
  └─ SourcesStep.svelte (onboarding UI)
       └─ submits config via api.settings.init()

settings store (draft)
  └─ ContentSourcesSection.svelte (settings UI)
       ├─ reads vault health via api.vault.sources()
       ├─ triggers rescan via api.sources.reindex(id)
       └─ saves config via api.settings.patch()
```

## API endpoints used

| Endpoint | Purpose |
|----------|---------|
| `GET /api/vault/sources` | Fetch source status + note counts for health display |
| `POST /api/sources/{id}/reindex` | Trigger a full rescan of a local_fs source |
| `GET /api/runtime/status` | Capabilities and deployment mode |
| `GET /api/settings/status` | Pre-auth capabilities fallback during onboarding |

## What "Loop back" means today vs future

**Today (Session 11):** Provenance is tracked end-to-end. When a tweet is generated with vault context, the `VaultCitation` records which chunk/node contributed. This data is stored in the approval queue and draft records. The toggle enables the config flag, but **no data is written back to source files yet**.

**Future:** The write-back step will update note frontmatter with performance data (likes, impressions, engagement scores) from tweets that cited the note. This creates the full learning loop: notes → tweets → performance → notes.
