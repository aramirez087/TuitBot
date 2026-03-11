# Current State Audit: Scheduling System

_Audited 2026-03-10 against `main` branch at commit `22fd9f8`_

---

## 1. Scheduling Surfaces Inventory

| Surface | Frontend Component | API Endpoint | Backend Handler | Storage |
|---------|-------------------|--------------|-----------------|---------|
| Home Composer (embedded) | `ComposeWorkspace.svelte:153,492-499` via `HomeComposerHeader.svelte` | `POST /api/content/compose` | `compose.rs::compose()` | `scheduled_content` table |
| Modal Composer (dialog) | `ComposeWorkspace.svelte` (non-embedded) | `POST /api/content/compose` | `compose.rs::compose()` | `scheduled_content` table |
| Draft Studio schedule | `DraftStudioShell.svelte:317-329` via `DraftScheduleSection.svelte` | `POST /api/drafts/:id/schedule` | `draft_studio.rs::schedule_studio_draft()` | `scheduled_content` table |
| Draft Studio unschedule | `DraftStudioShell.svelte:332-345` | `POST /api/drafts/:id/unschedule` | `draft_studio.rs::unschedule_studio_draft()` | `scheduled_content` table |
| Draft Studio reschedule | `DraftStudioShell.svelte:347-352` | Unschedule then schedule (2 calls) | Two sequential handlers | `scheduled_content` table |
| Calendar time-slot click | `content/+page.svelte:74-76` | Redirects to Draft Studio with `?prefill_schedule=` | N/A (URL param) | N/A |
| Calendar day click | `content/+page.svelte:78-80` | Redirects to Draft Studio with `?new=true` | N/A | N/A |
| Calendar cancel | `content/+page.svelte:82-84` | `DELETE /api/content/scheduled/:id` | `cancel_for()` | `scheduled_content` table |
| Calendar inline edit | `content/+page.svelte:86-88` | `PATCH /api/content/scheduled/:id` | `update_content_for()` | `scheduled_content` table |
| Legacy tweet compose | N/A (possibly external callers) | `POST /api/content/tweets` | `compose.rs::compose_tweet()` | `approval_queue` or echo |
| Legacy thread compose | N/A (possibly external callers) | `POST /api/content/threads` | `compose.rs::compose_thread()` | `approval_queue` or echo |

## 2. State Machine

```
                ┌─────────────────┐
                │   (new insert)  │
                └────┬───────┬────┘
                     │       │
              insert_draft  insert_for (compose flow)
                     │       │
                     v       v
              ┌──────────┐  ┌───────────┐
              │  draft    │  │ scheduled │◄──── compose w/ scheduled_for
              │           │  │           │
              └──┬──┬─────┘  └──┬──┬──┬──┘
                 │  │           │  │  │
      schedule_  │  │ delete_   │  │  │ cancel_for
      draft_for  │  │ draft_for │  │  │
                 │  │           │  │  │
                 │  │   ┌───────┘  │  │
                 v  │   │          │  v
           ┌─────────┐ │    ┌──────────┐
           │scheduled │◄┘   │cancelled │
           └────┬─────┘     └──────────┘
                │ unschedule_draft_for
                v
           ┌──────────┐
           │  draft    │  (returns to draft)
           └──────────┘

                │ (posting engine picks up due items)
                v
           ┌──────────┐
           │  posted   │
           └──────────┘
```

Transitions:
- `draft` -> `scheduled`: via `schedule_draft_for()` (Draft Studio) or `insert_for()` with `scheduled_for` (compose flow)
- `scheduled` -> `draft`: via `unschedule_draft_for()` (Draft Studio only)
- `scheduled` -> `posted`: via posting engine (`update_status_for()`)
- `scheduled` -> `cancelled`: via `cancel_for()` (calendar)
- `draft` -> `cancelled`: via `delete_draft_for()` (Draft Studio)

Note: The compose flow creates items directly in `scheduled` status (never `draft`). Draft Studio creates items in `draft` status and transitions them.

## 3. Timezone Handling Analysis

### 3.1 Compose Flow (Home/Modal Composer)

**`composeHandlers.ts:36-41`** — datetime construction:
```typescript
if (selectedTime) {
    const scheduled = new Date(targetDate);
    const [h, m] = selectedTime.split(':').map(Number);
    scheduled.setHours(h, m, 0, 0);
    data.scheduled_for = scheduled.toISOString().replace('Z', '');
}
```

Issue: `new Date(targetDate)` creates a date in browser-local timezone, then `setHours(h, m)` sets hours in browser-local time, then `.toISOString()` converts to UTC and `.replace('Z', '')` strips the timezone marker. The server receives a string that looks like it could be UTC but is actually the UTC-converted version of the user's browser-local time. If the user is in UTC-5 and schedules for "14:00", the server receives "19:00:00.000" (UTC) without any indication that a timezone conversion occurred.

### 3.2 Draft Studio Schedule Section

**`DraftScheduleSection.svelte:28-29`** — display initialization:
```typescript
const d = new Date(draftSummary.scheduled_for);
scheduleDate = d.toISOString().slice(0, 10);  // UTC date
scheduleTime = d.toTimeString().slice(0, 5);   // LOCAL time
```

Issue: `toISOString()` returns UTC date (e.g., "2026-03-11" if the UTC date crosses midnight), while `toTimeString()` returns browser-local time. Mixing UTC date with local time means the displayed value may be a date/time combination that doesn't match what was scheduled.

**`DraftScheduleSection.svelte:49`** — building ISO for submission:
```typescript
return new Date(`${scheduleDate}T${scheduleTime}`).toISOString();
```

Issue: `new Date("2026-03-11T14:00")` is parsed as local time, then `.toISOString()` converts to UTC. The constructed datetime uses the mixed UTC-date/local-time values from above, compounding the error.

### 3.3 Calendar Store

**`calendar.ts:87-88`** — date formatting:
```typescript
function formatDateISO(d: Date): string {
    return d.toISOString().replace('Z', '');
}
```

Issue: `weekStart` and `monthStart` are computed using `setDate()` and `setHours(0,0,0,0)` in browser-local time, then `formatDateISO` converts to UTC. A user in UTC+9 looking at "Monday" will query a range starting from Sunday 15:00 UTC. The server's `BETWEEN` comparison against stored datetimes (which are also timezone-confused) happens to work by accident when all users are in roughly the same timezone, but breaks for any timezone offset.

### 3.4 Calendar Route (Server)

**`calendar.rs:133-147`** — scheduled content query:
```rust
let scheduled = scheduled_content::get_in_range_for(
    &state.db, &account_id, &query.from, &query.to
).await.map_err(ApiError::Storage)?;
```

**`scheduled_content/mod.rs:181-186`** — SQL comparison:
```sql
WHERE (scheduled_for BETWEEN ? AND ?)
   OR (scheduled_for IS NULL AND created_at BETWEEN ? AND ?)
```

Issue: The `BETWEEN` comparison is string-based. The stored `scheduled_for` values are bare ISO strings without timezone markers. The query parameters are also bare ISO strings derived from browser-local-to-UTC conversion. This works accidentally when all clients share the same timezone offset but produces wrong results when:
- A user travels and their browser timezone changes
- The account timezone differs from the browser timezone
- Multiple users access the same account from different timezones

### 3.5 Schedule Config (Unused)

**`calendar.rs:156-173`** — the `/api/content/schedule` endpoint returns a `ScheduleConfig` with a `timezone` field. This timezone is displayed as a badge in the calendar UI (`content/+page.svelte:116-118`) but is never used for datetime construction or conversion anywhere in the frontend.

### 3.6 TimePicker

**`TimePicker.svelte:21-30`** — date display:
```typescript
const dateDisplay = $derived(() => {
    if (!targetDate) return 'today';
    // ... compares against new Date() (browser-local)
});
```

Issue: "today" vs "tomorrow" is computed in browser-local time. If the account timezone differs, the label may be wrong.

## 4. Publish vs. Schedule Flow Analysis

### 4.1 Intent Inference

The compose flow infers user intent from the presence/absence of `scheduled_for`:

| `scheduled_for` | `approval_mode` | `can_post` | Result |
|-----------------|----------------|-----------|--------|
| Some(time) | true | any | Queued for approval |
| Some(time) | false | any | Inserted as `scheduled` |
| None | true | any | Queued for approval |
| None | false | true | Direct post via X API |
| None | false | false | **Silent fallback**: scheduled at `Utc::now()` |

**`compose.rs:476-507`** — the silent fallback:
```rust
if !can_post {
    let scheduled_for = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let id = scheduled_content::insert_for(
        &state.db, &ctx.account_id, &body.content_type, content,
        Some(&scheduled_for),
    ).await?;
    // ... returns { "status": "scheduled" }
}
```

The frontend shows "Published" toast (via `ComposeWorkspace.svelte:528-529`: `canPublish && !selectedTime ? "Published." : "Saved to calendar."`) but the content was actually scheduled. The user believes their content is live when it is not.

### 4.2 No Explicit Action Field

`ComposeRequest` has no `action` field to distinguish:
- "Publish this right now"
- "Schedule this for later"
- "Save as draft without scheduling"

Intent is inferred from `scheduled_for` presence, which conflates "when" with "what to do."

### 4.3 HomeComposerHeader Dual Buttons

**`HomeComposerHeader.svelte:61-88`** — when `selectedTime` is set:
- Primary button: "Schedule" (calls `onsubmit`)
- Secondary button: Send icon (calls `onpublishnow ?? onsubmit`)

`onpublishnow` is declared as an optional prop but is never wired up in `ComposeWorkspace.svelte` — it defaults to calling the same `onsubmit` handler. Both buttons do the same thing.

## 5. Data Model

### `scheduled_content` table (from `mod.rs:23-65`)

| Column | Type | Notes |
|--------|------|-------|
| `id` | i64 | Auto-increment PK |
| `content_type` | String | "tweet" or "thread" |
| `content` | String | Text for tweets, JSON array for threads |
| `scheduled_for` | Option<String> | Bare ISO-8601, no timezone |
| `status` | String | draft, scheduled, posted, cancelled |
| `posted_tweet_id` | Option<String> | X tweet ID after posting |
| `created_at` | String | `datetime('now')` — SQLite UTC |
| `updated_at` | String | `datetime('now')` — SQLite UTC |
| `qa_report` | String | JSON QA data |
| `qa_hard_flags` | String | JSON |
| `qa_soft_flags` | String | JSON |
| `qa_recommendations` | String | JSON |
| `qa_score` | f64 | 0-100 |
| `title` | Option<String> | Draft Studio title |
| `notes` | Option<String> | Internal notes |
| `archived_at` | Option<String> | Soft delete |
| `source` | String | manual, assist, discovery, autonomous |
| `account_id` | String | Foreign key |

Note: `created_at` and `updated_at` use SQLite's `datetime('now')` which is always UTC. But `scheduled_for` is whatever the client sends — which is a timezone-confused bare string.

## 6. Identified Gaps

### G1: Timezone handling is fundamentally broken (Severity: High)
Every datetime construction path mixes browser-local and UTC conversions, strips timezone markers, and sends bare strings to the server. The account timezone from `ScheduleConfig` is displayed but never used. A user who travels or whose browser timezone differs from their account timezone will schedule content at the wrong time.

### G2: Publish vs. schedule intent is implicit (Severity: High)
No explicit action field exists. Intent is inferred from `scheduled_for` presence. When X API credentials are unavailable, the "Publish now" flow silently falls back to scheduling at `Utc::now()` — the user sees "Published" but content is actually queued. The `onpublishnow` button in `HomeComposerHeader` is not wired up.

### G3: Non-atomic reschedule (Severity: Medium)
`draftStudio.svelte.ts` `rescheduleDraft()` calls `unscheduleDraft()` then `scheduleDraft()` as two sequential API calls. If the second call fails, the draft loses its scheduled time and reverts to `draft` status silently. Two unnecessary revision snapshots are created.

### G4: Dual API paths for the same operation (Severity: Medium)
The compose flow (`POST /api/content/compose`) creates items directly in `scheduled` status. The Draft Studio flow creates items in `draft` status and promotes them via `POST /api/drafts/:id/schedule`. Both write to the same table but with different validation, revision tracking, and activity logging. The compose flow does not create revision snapshots or activity logs.

### G5: Two incompatible scheduling UIs (Severity: Medium)
`TimePicker.svelte` handles time-only selection (preferred slots + custom time input) and is used in the inspector panel of the home/modal composer. `DraftScheduleSection.svelte` handles full date+time with native HTML inputs and is used in the Draft Studio details panel. They have different UX patterns, different datetime construction logic, and different bugs.

### G6: Calendar queries are timezone-naive (Severity: Medium)
`calendar.ts` computes week/month boundaries in browser-local time, converts to UTC-ish bare strings, and sends them as query parameters. The server does string comparison against stored datetimes that are themselves timezone-confused. Off-by-one-day and off-by-hours errors are possible.

### G7: Legacy endpoints still exist (Severity: Low)
`POST /api/content/tweets` and `POST /api/content/threads` are still routed. They use the approval queue differently (no scheduled_content path) and may be called by automation or external integrations. Their behavior differs from the unified compose endpoint.

### G8: `DraftScheduleSection` mixes UTC and local time (Severity: Medium)
When displaying an existing scheduled time, the date is derived from `.toISOString()` (UTC) and the time from `.toTimeString()` (local). When the user edits and resubmits, the mixed values produce a different datetime than the original.

### G9: Compose flow skips Draft Studio features (Severity: Low)
Content created via the compose flow (home/modal composer) does not get: revision snapshots, activity logs, title/notes, tags, or archive capability. It goes straight into `scheduled_content` as a `scheduled` item with no draft lifecycle.
