# Competitive UX Notes: Scheduling Patterns

_Research compiled 2026-03-10 for ReplyGuy scheduling overhaul_

---

## 1. Typefully

### What they do well

**Queue-based scheduling**: Typefully's primary model is a publishing queue. Users write drafts and add them to a queue with configurable time slots. The queue auto-assigns the "next free slot" — users don't have to pick exact times unless they want to.

**Time slot suggestions**: Based on account analytics and engagement patterns, Typefully suggests optimal posting times. Users configure a weekly schedule (e.g., Mon 9:00, 12:00, 17:00; Tue 10:00, 14:00) and the queue fills slots in order.

**Thread-first composer**: Typefully treats threads as a first-class content type with a vertical card-based editor. Each card has its own character count and media controls. Scheduling applies to the entire thread as a unit.

**Drag-to-reschedule**: In the calendar/queue view, users can drag posts between time slots to reschedule. This is a single atomic operation — there is no intermediate "unschedule then reschedule" step.

**"Next free slot" default**: When no time is selected, the default action is "add to queue" which finds the next available slot based on the posting schedule. This avoids forcing a time picker interaction for every post.

### Implications for ReplyGuy

1. **Adopt "next free slot" as default scheduling behavior**: ReplyGuy already has `preferred_times` in `ScheduleConfig` but only uses them as suggestions in `TimePicker`. The compose flow should default to assigning the next free preferred slot when the user clicks "Schedule" without selecting a time.

2. **Unified queue view**: ReplyGuy's calendar shows scheduled items but doesn't expose a queue concept. Adding a queue/list view alongside the calendar would make the scheduling model more intuitive for users who schedule many posts.

3. **Atomic reschedule**: ReplyGuy's two-call reschedule pattern (G3 in audit) should become a single PATCH operation, matching Typefully's drag-to-reschedule behavior.

---

## 2. Buffer

### What they do well

**Multi-platform queue with per-channel timezone**: Buffer allows different posting schedules per social account, each with its own timezone. The queue view shows times in the channel's timezone, not the user's browser timezone. This eliminates timezone confusion.

**Calendar grid view**: Buffer's calendar provides a grid view where each day shows all scheduled posts across channels. Clicking an empty slot opens a compose dialog pre-filled with that date/time. Posts can be dragged between days.

**Bulk scheduling**: Buffer supports uploading a CSV of posts to schedule, and "shuffle queue" to randomize order. Useful for content batching workflows.

**Simple three-state model**: Content in Buffer is either a Draft (not in queue), Queued (in queue, waiting for slot), or Sent. There is no separate "scheduled" vs "queued" distinction — all content in the queue posts at its assigned slot.

**Timezone per account**: Each connected social account has a configured timezone. All scheduling and display uses that account's timezone, not browser-local. Buffer explicitly shows "Times shown in [timezone]" in the UI.

### Implications for ReplyGuy

1. **Account timezone must be canonical**: Buffer's model of "timezone per social account" directly maps to ReplyGuy's existing `ScheduleConfig.timezone` field, which is already stored but not used for datetime construction. Adopting this pattern is the highest-priority fix.

2. **"Times shown in [timezone]" indicator**: Currently ReplyGuy shows a timezone badge in the calendar but doesn't use it for calculations. Adding a persistent "Times shown in America/New_York" indicator and using it for all datetime construction would prevent the timezone bugs identified in the audit.

3. **Calendar slot-click compose**: ReplyGuy already redirects calendar slot clicks to Draft Studio with a `?prefill_schedule=` param. This could be simplified to open a lightweight compose popover directly in the calendar, similar to Buffer, avoiding a full navigation.

---

## 3. Planable

### What they do well

**Approval workflows integrated with scheduling**: Planable's core differentiator is team approval built into the scheduling flow. Content can be in: Draft -> Pending Approval -> Approved -> Scheduled -> Published. The scheduling UI changes based on approval status — you can't schedule unapproved content.

**Calendar collaboration**: Multiple team members see the same calendar view and can leave comments on scheduled posts. Drag-to-reschedule is visible to all collaborators in real-time.

**Visual preview as default**: Planable renders posts as they would appear on the target platform. The preview is always visible, not hidden behind a toggle. This makes the scheduling experience feel concrete — you're scheduling something you can see.

**Unified timeline across content types**: Planable shows all content types (posts, stories, reels, threads) on the same calendar timeline. Content type is indicated by icons and card styling but doesn't affect the scheduling interaction.

**Labels and categories**: Posts can be tagged with color-coded labels for content planning. The calendar can be filtered by label, making it easy to see the distribution of content types over time.

### Implications for ReplyGuy

1. **Approval-aware scheduling UI**: ReplyGuy has an approval queue but it's separate from the scheduling flow. In the compose flow, approval mode sends content to the approval queue — it never reaches `scheduled_content`. The scheduling UI should make it clear when content will go through approval before being scheduled, similar to Planable's staged flow.

2. **Content tags on calendar**: ReplyGuy's Draft Studio already has tags. Extending tag visibility to the calendar view would help users with content planning.

3. **Preview-forward design**: ReplyGuy's composer has a preview toggle (`ComposerPreviewSurface`). Making the preview more prominent during scheduling — showing what the tweet will look like at the scheduled time — would increase scheduling confidence.

---

## 4. Synthesis: Patterns to Adopt

### Must adopt (directly addresses audit gaps)

| Pattern | Source | Addresses | Priority |
|---------|--------|-----------|----------|
| Account timezone as canonical for all datetime ops | Buffer | G1 (timezone broken) | P0 |
| Explicit "Times shown in [tz]" indicator | Buffer | G1 | P0 |
| Separate publish/schedule/draft actions | Typefully, Planable | G2 (implicit intent) | P0 |
| Atomic reschedule (single API call) | Typefully | G3 (non-atomic) | P1 |
| "Next free slot" default scheduling | Typefully | UX improvement | P1 |

### Should adopt (improves experience)

| Pattern | Source | Benefit |
|---------|--------|---------|
| Unified scheduling component (date+time+slot picker) | Typefully | Replaces two incompatible UIs (G5) |
| Approval-aware scheduling states | Planable | Clarifies compose flow behavior (G2, G4) |
| Calendar inline compose (popover) | Buffer | Faster than Draft Studio redirect |
| Queue/list view alongside calendar | Typefully | Better for batch scheduling |

### Skip for now

| Pattern | Source | Why skip |
|---------|--------|----------|
| Multi-platform scheduling | Buffer | ReplyGuy is X-only |
| Team collaboration on calendar | Planable | Single-user product currently |
| Bulk CSV import | Buffer | Low priority for ReplyGuy's use case |
| Real-time collaboration | Planable | Single-user product |

## 5. UI Component Recommendations

### Unified SchedulePicker component

Replace `TimePicker.svelte` and `DraftScheduleSection.svelte` with a single `SchedulePicker.svelte` that:

1. Shows the account timezone prominently ("Scheduling in America/New_York")
2. Defaults to "Next free slot" when no time is selected
3. Shows preferred time slots as quick-select buttons (from Typefully)
4. Includes a date picker for scheduling on other days (from DraftScheduleSection)
5. Includes a custom time input (from TimePicker)
6. Constructs all datetimes using the account timezone, not browser-local
7. Is usable in both the composer inspector panel and the Draft Studio details panel

### Calendar enhancements

1. Show timezone indicator persistently in calendar header
2. Support drag-to-reschedule for scheduled items (single atomic API call)
3. Show content tags as colored dots on calendar items
4. Add a popover compose for quick scheduling from calendar slots
