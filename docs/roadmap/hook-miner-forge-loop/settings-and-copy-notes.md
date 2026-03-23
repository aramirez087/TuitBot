# Settings & Copy Notes — Forge Analytics Sync

**Date:** 2026-03-22
**Session:** 09

---

## Two Distinct Data Flows

| Flow | Setting | Trigger | What It Writes |
|------|---------|---------|----------------|
| **Publish writeback** | `loop_back_enabled` (default: true) | Immediately after publish | tweet_id, url, published_at |
| **Analytics sync** | `analytics_sync_enabled` (default: false) | Periodic (15–60 min delay) | impressions, likes, retweets, engagement_rate, performance_score, performance_tier |

These are independent settings on `ContentSourceEntry`. Analytics sync is an opt-in superset that requires publish writeback to be active first.

---

## User-Facing Copy Decisions

### Loop Back Toggle (updated)

**Old:** "Write tweet performance data back into note frontmatter. Currently tracks which notes were used — file write-back coming soon."

**New:** "Write publish metadata (tweet ID, URL, timestamp) back into note frontmatter after posting. This records which notes were used and when."

**Rationale:** The old copy was outdated (writeback is implemented). The new copy is accurate and distinct from the analytics sync description.

### Analytics Sync Toggle

**Copy:** "Periodically enrich note frontmatter with engagement metrics (impressions, likes, retweets, engagement rate, performance score). Writes are local-only — data stays in your vault files. Stats typically arrive 15–60 minutes after posting."

**Design decisions:**
- Only visible when `loop_back_enabled` is true (analytics sync is a superset)
- Only available for `local_fs` sources
- Mentions "local-only" explicitly to address privacy concerns
- Sets expectations on timing ("15–60 minutes")

### Google Drive Unsupported Notice

**Copy:** "Analytics sync (writing performance data back to notes) is only available for local filesystem sources. Google Drive sources receive publish metadata only."

**Rationale:** Informational, not an error. Users shouldn't feel they're doing something wrong — just explaining a capability boundary.

### One-Time Consent Prompt (Activity Page)

**Title:** "Enable Analytics Sync?"

**Body:**
> Your note was published successfully. TuitBot can enrich your source note with engagement metrics (impressions, likes, performance score) as they arrive — typically 15–60 minutes after posting.
>
> All writes are local-only and stay in your vault. No data leaves your machine.

**Actions:** [Enable in Settings] [Not now]

**Design decisions:**
- Banner, not modal — modals block workflow and violate the Hook Miner principle of no hidden magic
- Appears on the Activity page (natural destination after publish), not the compose view
- "Enable in Settings" navigates to Settings#sources so the user sees the full context
- "Not now" suppresses the prompt via localStorage until the user revisits Settings > Content Sources

---

## Prompt Behavior Rules

1. **Trigger:** Set pending prompt when a publish succeeds for content with `local_fs` provenance, `loop_back_enabled: true`, and `analytics_sync_enabled: false`.
2. **Display:** Show on next Activity page load if pending flag is set, prompt not dismissed, and analytics sync still disabled.
3. **Dismiss ("Not now"):** Sets localStorage flag. Prompt won't reappear.
4. **Reset:** Visiting Settings > Content Sources resets the dismissal flag (via `onMount`), giving users a fresh prompt opportunity after they've reviewed settings.
5. **Enable:** Navigates to `/settings#sources`. User can toggle analytics sync directly.
6. **No spam:** Only one prompt at a time. Flag is per-browser, not per-publish.

---

## Unsupported Source Type Handling

| Source Type | Publish Writeback | Analytics Sync | UI Behavior |
|-------------|-------------------|----------------|-------------|
| `local_fs` | Supported | Supported | Both toggles shown |
| `google_drive` | Metadata only | Not supported | Info notice shown, no analytics toggle |

Google Drive sources cannot receive analytics sync because the sync engine writes to local filesystem paths. The notice explains this without suggesting the user is misconfigured.
