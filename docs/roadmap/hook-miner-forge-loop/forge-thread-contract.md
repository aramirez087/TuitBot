# Forge Thread Contract

**Date:** 2026-03-22
**Session:** 06
**Status:** Active
**Companion to:** `forge-frontmatter-contract.md`

---

## Overview

A thread is **one outcome** in the `tuitbot:` array, not N individual tweets. When a user publishes a thread from their vault note, it produces a single entry with `type: thread`, the root tweet ID as `tweet_id`, and aggregated metrics across all tweets in the chain.

Individual per-tweet metrics are available in the dashboard analytics (via `tweet_performance` and `thread_tweets` tables). The frontmatter captures the aggregate — the note's social outcome as a single unit.

---

## Entry Schema for Threads

A thread entry uses all the same fields as a tweet entry (see `forge-frontmatter-contract.md`), with these thread-specific behaviors:

| Field | Thread Behavior |
|-------|-----------------|
| `tweet_id` | Root tweet ID (the first tweet posted in the chain) |
| `url` | URL of the root tweet |
| `type` | `"thread"` |
| `thread_url` | Same as `url`. Present for thread-aware clients that want to distinguish thread URLs from tweet URLs in display. |
| `child_tweet_ids` | Ordered array of child tweet IDs, position-ordered, **excludes** the root. A 4-tweet thread has 3 child IDs. |
| `impressions` | Sum of impressions across root + all children |
| `likes` | Sum of likes across root + all children |
| `retweets` | Sum of retweets across root + all children |
| `replies` | Sum of replies across root + all children |
| `engagement_rate` | `(total_likes + total_retweets + total_replies) / total_impressions × 100`. `null` if total impressions = 0. |
| `performance_score` | Impression-weighted average of child performance scores. See computation below. |
| `synced_at` | Timestamp of the sync that produced these aggregates |

---

## Aggregation Rules

### Count metrics (impressions, likes, retweets, replies)

**Rule:** Sum across all tweets (root + children).

```
thread.impressions = root.impressions + sum(child.impressions for each child)
thread.likes       = root.likes       + sum(child.likes for each child)
thread.retweets    = root.retweets    + sum(child.retweets for each child)
thread.replies     = root.replies     + sum(child.replies for each child)
```

Only tweets with non-null metrics contribute. If a child has `null` impressions (not yet measured), it is excluded from the sum.

### Engagement rate

**Rule:** Computed from aggregated counts, not averaged across tweets.

```
engagement_rate = (total_likes + total_retweets + total_replies) / total_impressions × 100
```

If `total_impressions` is 0 (or all tweet impressions are null), `engagement_rate` is `null`.

### Performance score

**Rule:** Impression-weighted average.

```
weighted_sum = sum(tweet.performance_score × tweet.impressions for each tweet where both are non-null)
total_weight = sum(tweet.impressions for each tweet where performance_score is non-null)
performance_score = weighted_sum / total_weight
```

If no tweets have both `performance_score` and `impressions`, the thread's `performance_score` is `null`.

**Rationale:** A 4-tweet thread where tweet #1 got 10K impressions and tweet #4 got 200 impressions should weight tweet #1's score more heavily. Simple averaging would give equal weight to a tweet nobody saw.

---

## Matching Rules

### Canonical identifier

The root tweet ID is the canonical `tweet_id` for the thread entry. All matching and deduplication uses this value.

### Publish step matching

When the approval poster publishes a thread:
1. Post root tweet → get `root_tweet_id`.
2. Post children as replies → get `child_tweet_ids`.
3. Call `write_metadata_to_file()` with `tweet_id = root_tweet_id`, `type = "thread"`, `child_tweet_ids = [child1, child2, ...]`.
4. Deduplication check: if `root_tweet_id` is already in the array, skip.

### Forge sync matching

When Forge syncs analytics:
1. Fetch metrics for `root_tweet_id` and each ID in `child_tweet_ids`.
2. Aggregate per the rules above.
3. Match the entry by `tweet_id = root_tweet_id`.
4. Update analytics fields with aggregated values.

If `child_tweet_ids` is not yet populated on the entry (thread was published before this field existed):
1. Look up `thread_tweets` table to find children for this root.
2. Populate `child_tweet_ids` on the entry.
3. Proceed with aggregation.

---

## Lifecycle

### Phase 1: Thread published (approval poster)

```yaml
tuitbot:
  - tweet_id: "1904000000000000001"
    url: "https://x.com/i/status/1904000000000000001"
    published_at: "2026-03-22T18:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/1904000000000000001"
    child_tweet_ids:
      - "1904000000000000002"
      - "1904000000000000003"
      - "1904000000000000004"
```

Analytics fields are all absent (null/omitted).

### Phase 2: First Forge sync

Forge fetches metrics for all 4 tweet IDs and aggregates:

| Tweet | Impressions | Likes | Retweets | Replies | Score |
|-------|-------------|-------|----------|---------|-------|
| Root  | 5200 | 84 | 21 | 15 | 78.2 |
| Child 1 | 3100 | 45 | 10 | 8 | 65.0 |
| Child 2 | 2800 | 38 | 7 | 12 | 60.1 |
| Child 3 | 1900 | 22 | 4 | 5 | 48.5 |

Aggregated:
- `impressions` = 5200 + 3100 + 2800 + 1900 = **13000**
- `likes` = 84 + 45 + 38 + 22 = **189**
- `retweets` = 21 + 10 + 7 + 4 = **42**
- `replies` = 15 + 8 + 12 + 5 = **40**
- `engagement_rate` = (189 + 42 + 40) / 13000 × 100 = **2.08**
- `performance_score` = (78.2×5200 + 65.0×3100 + 60.1×2800 + 48.5×1900) / (5200 + 3100 + 2800 + 1900) = **66.2**

### Phase 3: Subsequent Forge syncs

Same process. All analytics fields are overwritten with the latest aggregated values. `synced_at` is updated.

---

## YAML Examples

### Thread with 3 tweets, no analytics

```yaml
---
title: "The Hidden Cost of Premature Abstraction"
tags: [software-engineering, refactoring]
tuitbot:
  - tweet_id: "1904100000000000001"
    url: "https://x.com/i/status/1904100000000000001"
    published_at: "2026-03-22T20:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/1904100000000000001"
    child_tweet_ids:
      - "1904100000000000002"
      - "1904100000000000003"
---

Premature abstraction costs more than duplication...
```

### Thread after Forge sync with aggregated metrics

```yaml
---
title: "The Hidden Cost of Premature Abstraction"
tags: [software-engineering, refactoring]
tuitbot_social_performance: high
tuitbot_best_post_impressions: 18500
tuitbot_best_post_url: "https://x.com/i/status/1904100000000000001"
tuitbot_last_synced_at: "2026-03-24T02:00:00Z"
tuitbot:
  - tweet_id: "1904100000000000001"
    url: "https://x.com/i/status/1904100000000000001"
    published_at: "2026-03-22T20:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/1904100000000000001"
    child_tweet_ids:
      - "1904100000000000002"
      - "1904100000000000003"
    impressions: 18500
    likes: 420
    retweets: 112
    replies: 58
    engagement_rate: 3.19
    performance_score: 88.4
    synced_at: "2026-03-24T02:00:00Z"
---

Premature abstraction costs more than duplication...
```

### Note with both a standalone tweet and a thread

```yaml
---
title: "Database Migration Anti-Patterns"
tags: [databases, devops]
tuitbot_social_performance: high
tuitbot_best_post_impressions: 31200
tuitbot_best_post_url: "https://x.com/i/status/1904200000000000001"
tuitbot_last_synced_at: "2026-03-25T02:00:00Z"
tuitbot:
  - tweet_id: "1904200000000000099"
    url: "https://x.com/i/status/1904200000000000099"
    published_at: "2026-03-18T09:00:00Z"
    type: tweet
    status: posted
    impressions: 2100
    likes: 28
    retweets: 5
    replies: 3
    engagement_rate: 1.71
    performance_score: 52.0
    synced_at: "2026-03-25T02:00:00Z"
  - tweet_id: "1904200000000000001"
    url: "https://x.com/i/status/1904200000000000001"
    published_at: "2026-03-22T15:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/1904200000000000001"
    child_tweet_ids:
      - "1904200000000000002"
      - "1904200000000000003"
      - "1904200000000000004"
      - "1904200000000000005"
    impressions: 31200
    likes: 780
    retweets: 195
    replies: 94
    engagement_rate: 3.43
    performance_score: 93.1
    synced_at: "2026-03-25T02:00:00Z"
---

The three most dangerous migration patterns...
```

Note: `tuitbot_best_post_impressions` is 31200 (the thread) — not 2100 (the standalone tweet). Summary fields always reflect the highest-impression entry.

---

## Edge Cases

### Partial thread (some tweets failed to post)

If only the root and first child posted successfully (child 2 failed):

```yaml
tuitbot:
  - tweet_id: "1904300000000000001"
    url: "https://x.com/i/status/1904300000000000001"
    published_at: "2026-03-22T21:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/1904300000000000001"
    child_tweet_ids:
      - "1904300000000000002"
```

Only successfully posted children appear in `child_tweet_ids`. The thread entry represents what was actually published, not what was intended.

### Thread where only root has analytics

If Forge fetches metrics but children haven't been measured yet (X API delay):

- `impressions` = root impressions only (children contribute 0)
- `likes`, `retweets`, `replies` = root values only
- `engagement_rate` = computed from root values only
- `performance_score` = root score only

On next sync, once children have metrics, the aggregates update to include them.

### Child tweet added later (thread extension)

If the user manually extends a thread after initial publication:
1. The new child tweet ID is not automatically added to `child_tweet_ids`.
2. On next Forge sync, if the `thread_tweets` table contains the new child (because it was posted via Tuitbot), Forge adds it to `child_tweet_ids` and includes it in aggregation.
3. If the extension was posted outside Tuitbot, Forge has no visibility and the child is excluded.

### Thread with zero total impressions

All analytics fields are `null`. `engagement_rate` is `null`. `performance_score` is `null`. Summary tier is `"none"`.

### Thread entry without `child_tweet_ids` (legacy)

Pre-Forge thread entries may exist without `child_tweet_ids`. Forge populates this field from the `thread_tweets` table on first sync:

```sql
SELECT tweet_id FROM thread_tweets
WHERE thread_id = (SELECT id FROM threads WHERE root_tweet_id = ?)
ORDER BY position ASC;
```

If no `thread_tweets` rows exist (thread record was never created), `child_tweet_ids` remains absent and analytics reflect only the root tweet.

### Root tweet ID changed (deleted and reposted)

This does not happen in the current architecture. The root tweet ID is immutable once the thread is posted. If the root is deleted, the entry's `status` is updated to `"deleted"` and Forge stops syncing analytics for it.

---

## Relationship to Dashboard Analytics

The frontmatter contains **aggregated** thread metrics. The dashboard shows **per-tweet** breakdown:

| Data Point | Frontmatter | Dashboard |
|------------|-------------|-----------|
| Thread impressions | Sum (one number) | Per-tweet bar chart |
| Thread engagement | Aggregate rate | Per-tweet rates |
| Thread score | Weighted average | Per-tweet scores |
| Child tweet IDs | Listed | Linked to individual tweet views |

This separation is intentional: frontmatter is for Obsidian Dataview queries (simple, aggregate); dashboard is for analysis (detailed, interactive).
