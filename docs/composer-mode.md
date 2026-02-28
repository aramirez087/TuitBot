# Composer Mode

Composer Mode transforms Tuitbot from a fully autonomous agent into a user-driven writing tool with on-demand AI intelligence. The same scoring engine, LLM integration, and safety guardrails power both modes — the difference is who decides when to post.

The composer UX is designed to be faster, more structurally powerful, and more accessible than any comparable tool. It gives you keyboard-first control over every aspect of content creation while providing real-time preview, auto-save recovery, and AI-assisted writing — all in a single modal.

## Enabling Composer Mode

### Dashboard Settings

Open the Settings page and select **Composer** from the Operating Mode dropdown.

### config.toml

```toml
mode = "composer"
```

### Environment variable

```bash
export TUITBOT_MODE=composer
```

The default mode is `autopilot`, which preserves the fully autonomous behavior.

## What Changes

| Capability | Autopilot | Composer |
|---|---|---|
| Discovery loop | Active — finds and queues replies autonomously | Read-only — scores tweets for the Discovery Feed, never queues |
| Mentions loop | Active | Disabled |
| Target monitoring loop | Active | Disabled |
| Content posting loop | Active | Disabled |
| Thread publishing loop | Active | Disabled |
| Posting queue | Active | Active |
| Approval poster | Active | Active |
| Analytics snapshots | Active | Active |
| Token refresh | Active | Active |
| Approval mode | Configurable (`approval_mode`) | Always on (implicit) |
| AI Assist | Available | Available |
| Drafts | Available | Available |
| Discovery Feed | Available | Available |

In Composer mode, you write and schedule content yourself. Tuitbot provides AI assistance on demand, surfaces interesting conversations through the Discovery Feed, and handles the mechanics of posting, scheduling, and analytics.

## Thread Composer

The thread composer uses a card-based editor where each tweet in the thread is a visual card with its own textarea, character counter, and media slot. A two-pane layout shows the editor on the left and a live tweet-style preview on the right (stacked vertically on mobile).

### Data Model

Each thread card is a `ThreadBlock`:

```json
{
  "id": "uuid-v4",
  "text": "Tweet content",
  "media_paths": ["path/to/image.jpg"],
  "order": 0
}
```

Threads are stored as a `ThreadBlocksPayload`: `{ "version": 1, "blocks": [...] }`. The server also accepts the legacy `content` string format (JSON-stringified text array) for backwards compatibility.

### Validation

- Minimum 2 cards (single-card content should use tweet mode)
- Maximum 280 characters per card (weighted: URLs count as 23 characters, emoji as 2)
- No empty cards allowed
- Per-card media limits apply independently

### Power Actions

Four structural operations give you fine-grained control over thread composition — all accessible via keyboard shortcuts, drag-and-drop, or the command palette:

| Action | Shortcut (Mac) | Description |
|--------|----------------|-------------|
| Reorder | `⌥↑` / `⌥↓` or drag handle | Move a card up or down in the thread order |
| Duplicate | `⌘D` | Copy the current card (text + media) as a new card below |
| Split | `⌘⇧S` | Split the current card into two cards at the cursor position |
| Merge | `⌘⇧M` | Combine the current card with the card below it |

Typefully offers only a single reorder action (drag-and-drop). Tuitbot provides 4 keyboard-accessible power actions that let you restructure threads without lifting your hands from the keyboard.

### Per-Tweet Media

Each thread card has its own media slot supporting file picker and drag-and-drop attachment. Media constraints per card:

- **Images:** Up to 4 images, max 5 MB each (JPEG, PNG, WebP)
- **GIF:** 1 GIF, max 15 MB (exclusive — cannot combine with images or video)
- **Video:** 1 video, max 512 MB (MP4, exclusive — cannot combine with images or GIF)

Media follows its card on reorder, duplicate, and split operations.

## Distraction-Free Mode

Toggle with `⌘⇧F` (Mac) / `Ctrl+Shift+F` (Windows/Linux) or the focus mode button in the modal header.

Focus mode expands the compose modal to fill the entire viewport, hiding surrounding UI chrome. The editor and preview panes are preserved — all functionality, shortcuts, command palette, and AI assist remain fully accessible.

Press `Escape` to exit focus mode (the modal stays open). This follows the escape cascade: pressing Escape repeatedly closes layers in order — command palette, from-notes panel, focus mode, then the modal itself.

## Command Palette

Press `⌘K` (Mac) / `Ctrl+K` (Windows/Linux) to open the command palette.

The palette provides fuzzy search over 13 compose actions organized into 4 categories: **Mode**, **Compose**, **AI**, and **Thread**. Thread-specific actions are only visible when in thread mode. Each action that has a direct keyboard shortcut displays the hint inline.

Navigate with `↑` / `↓` arrow keys, execute with `Enter`, close with `Escape`.

For the full list of palette actions, see the [Keyboard Shortcuts](#keyboard-shortcuts).

## Keyboard Shortcuts

14 keyboard shortcuts cover all compose operations. Shortcuts are platform-aware (`⌘` on Mac, `Ctrl` on Windows/Linux) and are active only while the Compose Modal is open.

### Quick Reference

| Action | Mac | Win/Linux | When |
|--------|-----|-----------|------|
| Submit / Post | `⌘↩` | `Ctrl+Enter` | Always |
| Command palette | `⌘K` | `Ctrl+K` | Always |
| Focus mode | `⌘⇧F` | `Ctrl+Shift+F` | Always |
| AI improve | `⌘J` | `Ctrl+J` | Always |
| Tweet mode | `⌘⇧N` | `Ctrl+Shift+N` | Always |
| Thread mode | `⌘⇧T` | `Ctrl+Shift+T` | Always |
| Close | `Esc` | `Esc` | Always |
| Move card up/down | `⌥↑` / `⌥↓` | `Alt+↑/↓` | Thread |
| Duplicate card | `⌘D` | `Ctrl+D` | Thread |
| Split at cursor | `⌘⇧S` | `Ctrl+Shift+S` | Thread |
| Merge with next | `⌘⇧M` | `Ctrl+Shift+M` | Thread |
| Next / prev card | `Tab` / `⇧Tab` | `Tab` / `Shift+Tab` | Thread |

Full reference with descriptions: [Keyboard Shortcuts](#keyboard-shortcuts).

Typefully provides only `Cmd+Enter` for submission. Tuitbot provides 14 shortcuts covering every compose operation — you can create, restructure, and submit a thread without touching the mouse.

## Auto-Save & Recovery

Content is automatically saved to `localStorage` with a 500 ms debounce. If you close the modal without submitting, a recovery prompt appears the next time you open the Compose Modal.

- **Recover:** Restores the saved content (mode, tweet text, thread blocks)
- **Discard:** Clears the saved draft and starts fresh

Auto-save uses the storage key `tuitbot:compose:draft` with a 7-day TTL. Saved content is cleared on successful submit.

### Edge Cases

- Multiple browser tabs share the same `localStorage` key; the last write wins
- If `localStorage` quota is exceeded, auto-save fails silently (no data loss, just no recovery)
- Content older than 7 days is automatically discarded on the next compose open

## AI Assist

AI Assist provides on-demand content generation powered by your configured LLM. It uses the same persona, content frameworks, and topic knowledge as the autonomous loops — but only generates content when you ask.

### Inline AI Improve (⌘J)

Select text in the tweet editor and press `⌘J` to improve just the selection. If no text is selected, the entire tweet content is improved. In thread mode, the improvement targets the focused card.

### Generate from Notes

Click the notes button in the modal footer or select "Generate from notes" from the command palette. Paste rough notes or bullet points, and AI generates a polished tweet or thread from them. If existing content is present, you are prompted before replacement.

### AI Assist Button

The footer AI Assist button has context-aware behavior:
- **Tweet mode with content:** Runs AI Improve on the full text
- **Tweet mode without content:** Generates a new tweet on a general topic
- **Thread mode:** Generates a full thread outline

### API Endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/assist/tweet` | Generate a tweet for a given topic |
| `POST` | `/api/assist/reply` | Generate a reply to a specific tweet |
| `POST` | `/api/assist/thread` | Generate a thread outline for a topic |
| `POST` | `/api/assist/improve` | Improve or rephrase existing draft text |
| `GET` | `/api/assist/topics` | Get suggested topics based on your profile and recent performance |
| `GET` | `/api/assist/optimal-times` | Get recommended posting times based on historical engagement |
| `GET` | `/api/assist/mode` | Get the current operating mode (`autopilot` or `composer`) |

## Compose Endpoint

The primary submission endpoint for tweets and threads:

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/content/compose` | Submit a tweet or thread for posting |

### Request Body

```json
{
  "content_type": "tweet" | "thread",
  "content": "string",
  "blocks": [{"id": "uuid", "text": "...", "media_paths": [], "order": 0}],
  "scheduled_for": "2026-03-01T14:30:00",
  "media_paths": ["path/to/file.jpg"]
}
```

| Field | Required | Notes |
|-------|----------|-------|
| `content_type` | Yes | `"tweet"` or `"thread"` |
| `content` | Yes | Tweet text, or JSON-stringified text array for threads (backwards compat) |
| `blocks` | No | Structured `ThreadBlock[]` for threads; takes precedence over `content` when present |
| `scheduled_for` | No | ISO 8601 datetime (without trailing `Z`); omit for immediate posting |
| `media_paths` | No | Server-side paths from `/api/media/upload`; for threads, per-card media is in `blocks[].media_paths` |

## Media Upload

Upload media files before attaching them to tweets or thread cards:

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/media/upload` | Upload a media file (multipart form data) |
| `GET` | `/api/media/file` | Serve an uploaded media file for preview |

Accepted types: JPEG, PNG, WebP, GIF, MP4. Size limits: images 5 MB, GIF 15 MB, video 512 MB.

## Drafts

Drafts give you a workspace for content that is not yet ready to post. Create drafts manually, generate them with AI Assist, or save Discovery Feed replies for later editing.

### Workflow

1. **Create** a draft — manually or via AI Assist.
2. **Edit** the draft text, adjust the topic, or attach media. Thread drafts use the structured blocks format.
3. **Schedule** the draft for a specific time, or **publish** it immediately (routes through the approval queue and posting pipeline).
4. **Delete** drafts you no longer need.

Editing a draft opens the Compose Modal pre-filled with the draft content, including thread blocks and media.

### API Endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/content/drafts` | Create a new draft |
| `GET` | `/api/content/drafts` | List all drafts |
| `PATCH` | `/api/content/drafts/{id}` | Update a draft |
| `DELETE` | `/api/content/drafts/{id}` | Delete a draft |
| `POST` | `/api/content/drafts/{id}/publish` | Publish a draft (queue for posting) |
| `POST` | `/api/content/drafts/{id}/schedule` | Schedule a draft for future posting |

## Discovery Feed

The Discovery Feed surfaces scored tweets from your configured keywords — the same tweets the autonomous discovery loop would find. In Composer mode, discovery runs in read-only mode: it scores and indexes tweets but never queues replies automatically.

### Workflow

1. **Browse** the feed — tweets are ranked by the 6-signal scoring engine.
2. **Compose** a reply using AI Assist or write your own.
3. **Queue** the reply for posting through the approval queue.

### API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/discovery/feed` | Get scored tweets from recent discovery runs |
| `GET` | `/api/discovery/keywords` | Get configured discovery keywords |
| `POST` | `/api/discovery/{tweet_id}/compose-reply` | Compose a reply to a discovered tweet |
| `POST` | `/api/discovery/{tweet_id}/queue-reply` | Queue a reply for posting |

## MCP Tools

Four MCP tools are available for Composer mode workflows:

| Tool | Description | Key parameters |
|---|---|---|
| `get_mode` | Returns the current operating mode (`autopilot` or `composer`) | None |
| `compose_tweet` | Generate a tweet using AI Assist | `topic`, `format` (optional) |
| `get_discovery_feed` | Retrieve scored tweets from the Discovery Feed | `limit`, `min_score` (optional) |
| `suggest_topics` | Get topic suggestions based on profile and performance data | `count` (optional) |

## Switching Between Modes

You can switch between Autopilot and Composer at any time. Here is what happens to in-flight items:

- **Approval queue**: Items already in the queue are preserved and will be posted regardless of mode. Switching to Autopilot does not auto-approve pending items.
- **Drafts**: Drafts are mode-independent. They persist across mode switches and can be published in either mode.
- **Scheduled content**: Scheduled posts remain scheduled. The posting queue and approval poster run in both modes.
- **Discovery data**: Scored tweets from previous discovery runs remain available in the Discovery Feed. Switching to Autopilot resumes autonomous reply queuing.

Switching modes does not restart the runtime. The change takes effect on the next loop iteration (typically within one interval cycle).

## Accessibility

The composer is built for full keyboard accessibility and meets WCAG AA standards.

- **Full keyboard navigation:** Every compose action is accessible without a mouse via 14 shortcuts and the command palette
- **Focus trap:** Tab cycles within the modal boundary and never escapes to the page behind it
- **Focus return:** Closing the modal returns focus to the element that triggered it (e.g., the Compose button)
- **ARIA:** `role="dialog"`, `aria-modal="true"`, `aria-live="polite"` on character counters and error messages
- **Contrast:** All text meets WCAG AA (4.5:1 minimum contrast ratio) in both light and dark themes
- **Reduced motion:** `prefers-reduced-motion` media query disables all CSS transitions and animations globally
- **Mobile responsive:** Full-viewport modal below 640px with 44px minimum touch targets, wrapped footer with full-width submit button, 16px textarea font size (prevents iOS Safari auto-zoom)
- **Touch devices:** Interactive elements expand to 44px targets on `pointer: coarse` devices; thread card actions are always visible on `hover: none` devices

## Migration Notes

If you are upgrading from a pre-thread-composer version, here is what changed:

1. **Thread editing is card-based.** Each tweet in a thread is a visual card with its own textarea, character counter, and media slot. The old sequential textarea array is replaced.

2. **Thread data uses structured blocks.** Threads are stored as `{ "version": 1, "blocks": [...] }` JSON. The server still accepts the legacy `content` string format for backwards compatibility — existing API integrations continue to work unchanged.

3. **Media can be attached per-tweet in threads.** Previously, media was only available in tweet mode. Now each thread card has its own media slot.

4. **Keyboard shortcuts are available.** 14 shortcuts cover all compose operations. See the [Keyboard Shortcuts](#keyboard-shortcuts).

5. **Auto-save protects your work.** Content is saved to `localStorage` every 500ms. If you close the modal without submitting, a recovery prompt appears next time.

6. **Command palette for power users.** Press `⌘K` / `Ctrl+K` to search and execute any compose action without touching the mouse.

7. **API consumers:** The `blocks` field in compose and draft endpoints is optional. Existing integrations using the `content` string field continue to work unchanged. When `blocks` is present, it takes precedence for thread content.

## Troubleshooting

### Common Compose Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Maximum 4 images allowed per tweet" | Attempting to attach a 5th image | Remove an image before adding another |
| "GIF/video cannot be combined with other media" | Attaching an image after a GIF or video | X API limitation: GIF and video attachments are exclusive |
| "Cannot add images when GIF/video is attached" | Attaching an image when a GIF/video exists | Remove the GIF/video first, then add images |
| "File exceeds maximum size" | Image > 5 MB, GIF > 15 MB, or video > 512 MB | Compress or resize the file before uploading |
| "Failed to upload media" | Server unreachable or disk full | Verify `tuitbot-server` is running; check available disk space |

### Thread Validation Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Character count exceeds 280 | Tweet card text too long | Use Split (`⌘⇧S`) to break into two cards, or edit the text |
| Single-card thread | Only one card in thread mode | Add more cards, or switch to tweet mode for single-tweet content |
| Empty card | Card with no text content | Type content or delete the empty card |
| Submission returns 400 | Empty cards, single-card thread, or malformed blocks | Ensure at least 2 non-empty cards with unique IDs |

### Auto-Save Recovery Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| No recovery prompt on reopen | Auto-save expired (> 7 days) or was cleared | Content beyond TTL is permanently discarded |
| Recovery restores wrong content | Multiple browser tabs writing concurrently | Auto-save uses a single `localStorage` key; last write wins |
| "Recover" button appears to do nothing | `localStorage` corrupted or at quota | Clear browser storage for the site (`Application > Storage` in DevTools) |

### Media in Threads

| Issue | Cause | Solution |
|-------|-------|----------|
| Cannot attach media to thread card | Card media slot at per-card limit | Check per-card media limit (4 images or 1 GIF/video) |
| Media not visible in preview | Uploaded path not yet available | Media preview loads from `localStorage` blob URLs; refresh if stale |
