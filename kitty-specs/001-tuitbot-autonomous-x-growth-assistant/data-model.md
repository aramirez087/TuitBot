# Data Model: ReplyGuy — Autonomous X Growth Assistant

**Feature**: `001-replyguy-autonomous-x-growth-assistant`
**Date**: 2026-02-21

## Entity Relationship Overview

```
BusinessProfile (config — not persisted in DB)
    │
    ├── used by ──► ScoringEngine
    ├── used by ──► ContentGenerator
    └── used by ──► DiscoveryLoop

DiscoveredTweet ◄── 1:N ──► ReplySent
OriginalTweet (standalone)
Thread ◄── 1:N ──► ThreadTweet
RateLimitState (per action type)
ActionLog (append-only audit trail)
```

## Entities

### DiscoveredTweet

A tweet retrieved from X search matching configured keywords.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | X tweet ID |
| `author_id` | TEXT | NOT NULL | X user ID of tweet author |
| `author_username` | TEXT | NOT NULL | @handle of tweet author |
| `content` | TEXT | NOT NULL | Full tweet text |
| `like_count` | INTEGER | NOT NULL, DEFAULT 0 | Likes at discovery time |
| `retweet_count` | INTEGER | NOT NULL, DEFAULT 0 | Retweets at discovery time |
| `reply_count` | INTEGER | NOT NULL, DEFAULT 0 | Replies at discovery time |
| `impression_count` | INTEGER | DEFAULT 0 | Impressions if available |
| `relevance_score` | REAL | NULL | Computed score (0-100) |
| `matched_keyword` | TEXT | NULL | Which keyword triggered discovery |
| `discovered_at` | TEXT | NOT NULL, DEFAULT NOW | ISO-8601 UTC timestamp |
| `replied_to` | INTEGER | NOT NULL, DEFAULT 0 | 0=no, 1=yes |

**Indexes**: `discovered_at`, `matched_keyword`, `(replied_to, relevance_score DESC)`

**Uniqueness**: `id` (tweet ID) is globally unique.

**Retention**: Unreplied tweets pruned after 7 days. Replied tweets retained for 90 days.

---

### ReplySent

A reply generated and posted by the agent.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Internal ID |
| `target_tweet_id` | TEXT | NOT NULL | Tweet ID we replied to (may be a discovered tweet or a mention) |
| `reply_tweet_id` | TEXT | NULL | Our reply's X tweet ID (NULL if post failed) |
| `reply_content` | TEXT | NOT NULL | Generated reply text |
| `llm_provider` | TEXT | NULL | Which LLM generated this |
| `llm_model` | TEXT | NULL | Which model was used |
| `created_at` | TEXT | NOT NULL, DEFAULT NOW | When reply was sent |
| `status` | TEXT | NOT NULL, DEFAULT 'sent' | sent / failed / deleted |
| `error_message` | TEXT | NULL | Error details if failed |

**Indexes**: `created_at`, `target_tweet_id`

**State transitions**: `sent` → `deleted` (if user requests removal). `failed` is terminal.

**Retention**: 90 days (needed for deduplication: never reply to same tweet twice).

---

### OriginalTweet

An educational tweet generated and posted by the agent.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Internal ID |
| `tweet_id` | TEXT | NULL | X tweet ID after posting (NULL if failed) |
| `content` | TEXT | NOT NULL | Tweet text |
| `topic` | TEXT | NULL | Industry topic this covers |
| `llm_provider` | TEXT | NULL | Which LLM generated this |
| `created_at` | TEXT | NOT NULL, DEFAULT NOW | When tweet was posted |
| `status` | TEXT | NOT NULL, DEFAULT 'sent' | sent / failed |
| `error_message` | TEXT | NULL | Error details if failed |

**Indexes**: `created_at`

**Retention**: 90 days.

---

### Thread

A series of connected tweets posted as a thread.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Internal ID |
| `topic` | TEXT | NOT NULL | Thread topic |
| `tweet_count` | INTEGER | NOT NULL, DEFAULT 0 | Number of tweets in thread |
| `root_tweet_id` | TEXT | NULL | X tweet ID of first tweet |
| `created_at` | TEXT | NOT NULL, DEFAULT NOW | When thread was posted |
| `status` | TEXT | NOT NULL, DEFAULT 'sent' | sent / partial / failed |

**State transitions**: `partial` (some tweets posted, error mid-thread) → manual resolution. `sent` and `failed` are terminal.

---

### ThreadTweet

An individual tweet within a thread.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Internal ID |
| `thread_id` | INTEGER | NOT NULL, FK → Thread, ON DELETE CASCADE | Parent thread |
| `position` | INTEGER | NOT NULL | 0-indexed order in thread |
| `tweet_id` | TEXT | NULL | X tweet ID after posting |
| `content` | TEXT | NOT NULL | Tweet text |
| `created_at` | TEXT | NOT NULL, DEFAULT NOW | When this tweet was posted |

**Constraints**: UNIQUE(`thread_id`, `position`)

---

### RateLimitState

Tracks current usage against configured limits per action type.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `action_type` | TEXT | PRIMARY KEY | 'reply', 'tweet', 'thread', 'search', 'mention_check' |
| `request_count` | INTEGER | NOT NULL, DEFAULT 0 | Actions taken in current period |
| `period_start` | TEXT | NOT NULL, DEFAULT NOW | When current period began |
| `max_requests` | INTEGER | NOT NULL | Limit per period (from config) |
| `period_seconds` | INTEGER | NOT NULL | Period duration in seconds |

**Behavior**: When `now - period_start >= period_seconds`, reset `request_count` to 0 and update `period_start`.

**Retention**: Never deleted. Counters reset on period expiry.

---

### ActionLog

Append-only audit trail of every action taken by the agent.

| Field | Type | Constraints | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Internal ID |
| `action_type` | TEXT | NOT NULL | 'search', 'reply', 'tweet', 'thread', 'mention_check', 'cleanup', 'auth_refresh' |
| `status` | TEXT | NOT NULL, DEFAULT 'success' | success / failure / skipped |
| `message` | TEXT | NULL | Human-readable description |
| `metadata` | TEXT | NULL | JSON blob for flexible extra data |
| `created_at` | TEXT | NOT NULL, DEFAULT NOW | ISO-8601 UTC timestamp |

**Indexes**: `created_at`, `(action_type, created_at)`

**Retention**: 14 days (configurable). Not needed for deduplication.

## Configuration Entities (not persisted in DB)

### BusinessProfile (from config.toml)

| Field | Type | Required | Description |
|---|---|---|---|
| `product_name` | String | Yes | Name of the user's product |
| `product_description` | String | Yes | One-line description |
| `product_url` | String | No | URL to the product |
| `target_audience` | String | Yes | Who the product is for |
| `product_keywords` | Vec\<String\> | Yes | Keywords for tweet discovery |
| `competitor_keywords` | Vec\<String\> | No | Competitor-related keywords |
| `industry_topics` | Vec\<String\> | Yes | Topics for content generation |

### ScoringWeights (from config.toml)

| Field | Type | Default | Description |
|---|---|---|---|
| `keyword_relevance_max` | f32 | 40.0 | Max points for keyword match |
| `follower_count_max` | f32 | 20.0 | Max points for author reach |
| `recency_max` | f32 | 15.0 | Max points for tweet freshness |
| `engagement_rate_max` | f32 | 25.0 | Max points for engagement ratio |
| `threshold` | u32 | 70 | Minimum score to trigger reply |
