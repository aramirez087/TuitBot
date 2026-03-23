# Thread Publish Normalization Contract

**Date:** 2026-03-22
**Session:** 07
**Status:** Implemented

---

## Overview

All three thread publish paths (approval poster, direct compose, scheduled) now produce identical storage artifacts when a thread is posted. This enables Forge to treat every thread as a single source-note outcome with reliable persistence and provenance.

## Storage Invariants

When a Ghostwriter thread posts successfully, the system creates:

1. **1 `threads` row** — `root_tweet_id` = first posted tweet ID, `status` = "sent", `tweet_count` = total posted tweets
2. **N `thread_tweets` rows** — one per posted tweet, with `position` 0..N-1 and the tweet's X ID
3. **1 `original_tweets` row** — for the root tweet (position 0), enabling existing topic/analytics queries to work unchanged

For partial failures (some tweets posted before an error):

1. **1 `threads` row** — `status` = "partial", `tweet_count` = number actually posted
2. **M `thread_tweets` rows** — only for successfully posted tweets
3. **1 `original_tweets` row** — root tweet (always position 0, always the first to post)

## Canonical Key

The **root tweet ID** is the canonical key for all thread operations:

- `threads.root_tweet_id` — the primary thread identifier
- `original_tweets.tweet_id` — matches root for analytics/topic compatibility
- `LoopBackEntry.tweet_id` — root ID in frontmatter
- Forge matches by `tweet_id` in the `tuitbot:` frontmatter array

## Provenance Chain

```
approval_queue (or scheduled_content)
  ├── copy_links_for → original_tweet (root OT row)
  └── copy_links_for → thread (thread row)
```

Both entity types receive identical provenance links. This is additive:

- Existing code querying `entity_type = 'original_tweet'` continues to work
- New Forge thread aggregation can query `entity_type = 'thread'`

## Loopback Contract

Thread entries written to source note frontmatter include:

```yaml
tuitbot:
  - tweet_id: "root_123"
    url: "https://x.com/i/status/root_123"
    published_at: "2026-03-22T10:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/i/status/root_123"
    child_tweet_ids:
      - "child_456"
      - "child_789"
```

- `child_tweet_ids` excludes the root, contains only children in posting order
- `thread_url` equals the root tweet URL
- `type` is always `"thread"` for thread entries
- The `execute_loopback_thread` function handles this; regular tweets continue using `execute_loopback`

## Publish Path Normalization

| Path | Reply Chain | Thread Records | OT Record | Provenance | Loopback |
|------|-------------|----------------|-----------|------------|----------|
| Approval poster | Yes (root standalone, children reply to previous) | Yes | Yes (root) | Yes (OT + thread entities) | Yes (with child_tweet_ids) |
| Direct compose | Yes (existing behavior) | Yes (new) | Yes (new) | Yes (OT + thread entities) | No (compose path lacks node_id) |
| Scheduled publish | Uses ThreadPoster trait | Not normalized yet | Not normalized yet | Not normalized yet | Not normalized yet |

## Thread Content Parsing

The `parse_thread_content()` function (in `approval_poster.rs`) handles both formats:

1. **Block JSON** — `{"version":1,"blocks":[{"id":"a","text":"...","order":0}, ...]}` — primary format from structured compose
2. **Legacy string array** — `["tweet 1","tweet 2"]` — backward compatibility

Blocks are sorted by `order` field before posting to ensure correct sequence.

## Query Helpers

### `get_thread_tweet_ids_by_root_for(pool, account_id, root_tweet_id)`

Returns child tweet IDs (position > 0) for a thread, ordered by position. Used by Forge sync as fallback when `child_tweet_ids` is not available in frontmatter.

### `persist_thread_records(pool, account_id, topic, tweet_ids, tweet_contents, status)`

Atomic helper that creates all three record types (thread + thread_tweets + original_tweets). Used by both approval poster and direct compose paths.

## Remaining Limitations

1. **Scheduled thread path (`content_loop/publisher.rs`)** is not yet normalized. It uses a `ThreadPoster` trait abstraction that differs from the approval poster's direct X API calls. Normalization requires a follow-up.

2. **Loopback for direct compose threads** — The compose path doesn't go through the provenance loopback system (no `node_id` context). Thread record persistence works, but frontmatter writeback requires provenance links with `node_id` which are only available when content originates from a vault note.

3. **Media attachments for child tweets** — The approval poster currently only attaches media to the root tweet (`media_ids` passed to first `post_tweet` call, `&[]` for children). Per-block media from structured compose is flattened at approval queue insertion time.
