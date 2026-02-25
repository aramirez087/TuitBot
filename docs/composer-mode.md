# Composer Mode

Composer Mode transforms Tuitbot from a fully autonomous agent into a user-driven writing tool with on-demand AI intelligence. The same scoring engine, LLM integration, and safety guardrails power both modes — the difference is who decides when to post.

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

## AI Assist

AI Assist provides on-demand content generation powered by your configured LLM. It uses the same persona, content frameworks, and topic knowledge as the autonomous loops — but only generates content when you ask.

Use AI Assist from the dashboard by clicking the AI Assist button in the Compose Modal, or call the API endpoints directly.

### API endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/assist/tweet` | Generate a tweet for a given topic |
| `POST` | `/api/assist/reply` | Generate a reply to a specific tweet |
| `POST` | `/api/assist/thread` | Generate a thread outline for a topic |
| `POST` | `/api/assist/improve` | Improve or rephrase existing draft text |
| `GET` | `/api/assist/topics` | Get suggested topics based on your profile and recent performance |
| `GET` | `/api/assist/optimal-times` | Get recommended posting times based on historical engagement |
| `POST` | `/api/assist/compose` | General-purpose compose endpoint (auto-detects content type) |

## Drafts

Drafts give you a workspace for content that is not yet ready to post. Create drafts manually, generate them with AI Assist, or save Discovery Feed replies for later editing.

### Workflow

1. **Create** a draft — manually or via AI Assist.
2. **Edit** the draft text, adjust the topic, or attach media.
3. **Schedule** the draft for a specific time, or **publish** it immediately (routes through the approval queue and posting pipeline).
4. **Delete** drafts you no longer need.

### API endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/drafts` | Create a new draft |
| `GET` | `/api/drafts` | List all drafts |
| `GET` | `/api/drafts/{id}` | Get a specific draft |
| `PUT` | `/api/drafts/{id}` | Update a draft |
| `POST` | `/api/drafts/{id}/publish` | Publish a draft (queue for posting) |
| `DELETE` | `/api/drafts/{id}` | Delete a draft |

## Discovery Feed

The Discovery Feed surfaces scored tweets from your configured keywords — the same tweets the autonomous discovery loop would find. In Composer mode, discovery runs in read-only mode: it scores and indexes tweets but never queues replies automatically.

### Workflow

1. **Browse** the feed — tweets are ranked by the 6-signal scoring engine.
2. **Compose** a reply using AI Assist or write your own.
3. **Queue** the reply for posting through the approval queue.

### API endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/discovery/feed` | Get scored tweets from recent discovery runs |
| `POST` | `/api/discovery/feed/{tweet_id}/reply` | Compose and queue a reply to a discovered tweet |
| `GET` | `/api/discovery/feed/stats` | Get discovery feed statistics |

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
