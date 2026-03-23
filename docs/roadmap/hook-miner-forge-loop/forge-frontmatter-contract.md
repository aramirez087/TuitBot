# Forge Frontmatter Contract

**Date:** 2026-03-22
**Session:** 06
**Status:** Active

---

## Overview

The Forge sync writes social performance data back into the Obsidian vault note that originated a published tweet or thread. Data lives in two locations within each note's YAML frontmatter:

1. **`tuitbot:` array** — one entry per published outcome (tweet or thread), carrying per-outcome analytics.
2. **Top-level `tuitbot_*` keys** — note-level summaries computed from all entries, optimized for Obsidian Dataview queries.

This contract is a strict superset of the existing `LoopBackEntry` and `TuitbotFrontMatter` structs. All new fields are optional. Existing notes with the current schema remain valid with no migration.

---

## Entry Schema

Each element of the `tuitbot:` array has the following fields.

| Field | Type | Required | Source | Description |
|-------|------|----------|--------|-------------|
| `tweet_id` | `String` | Yes | X API | Post ID. For threads, this is the **root** tweet ID. |
| `url` | `String` | Yes | Constructed | `https://x.com/i/status/{tweet_id}` |
| `published_at` | `String` | Yes | System clock | ISO-8601 UTC (`2026-03-22T14:30:00Z`) |
| `type` | `String` | Yes | Approval poster | `"tweet"`, `"thread"`, or `"reply"` |
| `status` | `String?` | No | System | `"posted"`, `"deleted"`. Default omitted (= posted). |
| `thread_url` | `String?` | No | Thread posting | URL of the root tweet when `type: thread`. Same as `url`. |
| `child_tweet_ids` | `String[]?` | No | Thread posting / Forge sync | Ordered child tweet IDs (excludes root). Only present when `type: thread`. |
| `impressions` | `Integer?` | No | X API | Total views across the outcome. |
| `likes` | `Integer?` | No | X API | Like count. |
| `retweets` | `Integer?` | No | X API | Retweet count. |
| `replies` | `Integer?` | No | X API | Reply count. |
| `engagement_rate` | `Float?` | No | Computed | `(likes + retweets + replies) / impressions × 100`. `null` when `impressions` is 0 or absent. |
| `performance_score` | `Float?` | No | `tweet_performance` table | Tuitbot's composite scoring metric. |
| `synced_at` | `String?` | No | System clock | ISO-8601 UTC of the last analytics sync for this entry. |

### Rust type extension

`LoopBackEntry` gains these optional fields with `#[serde(default, skip_serializing_if = "Option::is_none")]`:

```rust
pub child_tweet_ids: Option<Vec<String>>,
pub impressions: Option<i64>,
pub likes: Option<i64>,
pub retweets: Option<i64>,
pub replies: Option<i64>,
pub engagement_rate: Option<f64>,
pub performance_score: Option<f64>,
pub synced_at: Option<String>,
```

The `PartialEq, Eq` derive on `LoopBackEntry` must change to `PartialEq` only (since `f64` does not implement `Eq`). Callers that need equality checks on the non-float fields can compare field-by-field.

---

## Note-Level Summary Fields

These are top-level frontmatter keys (siblings of `tuitbot:`, not nested inside it).

| Key | Type | Description |
|-----|------|-------------|
| `tuitbot_social_performance` | `String` | Human-readable tier: `"high"`, `"medium"`, `"low"`, `"none"` |
| `tuitbot_best_post_impressions` | `Integer` | Impression count of the highest-performing entry |
| `tuitbot_best_post_url` | `String` | URL of the highest-performing entry |
| `tuitbot_last_synced_at` | `String` | ISO-8601 UTC of the most recent `synced_at` across all entries |

### Computation rules

After any Forge sync updates an entry's analytics fields:

1. Find the entry with the maximum `impressions` value. If tied, use the entry with the latest `published_at`.
2. Set `tuitbot_best_post_impressions` to that entry's `impressions`.
3. Set `tuitbot_best_post_url` to that entry's `url`.
4. Set `tuitbot_social_performance` based on the performance tier (see below).
5. Set `tuitbot_last_synced_at` to the most recent `synced_at` across all entries.

If no entry has a non-null `impressions` value, set `tuitbot_social_performance` to `"none"` and omit the other three keys.

### Performance tier thresholds

Tiers are relative to the account's own historical post distribution.

| Tier | Condition |
|------|-----------|
| `"high"` | `tuitbot_best_post_impressions` >= 90th percentile of account's historical posts |
| `"medium"` | `tuitbot_best_post_impressions` >= 50th percentile |
| `"low"` | `tuitbot_best_post_impressions` > 0 but < 50th percentile |
| `"none"` | No synced analytics data, or fewer than 10 historical posts |

When the account has fewer than 10 posts with analytics data, the tier defaults to `"none"` regardless of the impression count. This avoids misleading tiers from insufficient sample sizes.

### Rust type extension

`TuitbotFrontMatter` does not need explicit fields for the summary keys. They are read/written through the existing `#[serde(flatten)] other: serde_yaml::Mapping` field, using string keys:

- `"tuitbot_social_performance"` → `serde_yaml::Value::String`
- `"tuitbot_best_post_impressions"` → `serde_yaml::Value::Number`
- `"tuitbot_best_post_url"` → `serde_yaml::Value::String`
- `"tuitbot_last_synced_at"` → `serde_yaml::Value::String`

This avoids breaking the existing `TuitbotFrontMatter` struct contract and handles forward/backward compatibility automatically.

---

## Write Semantics

### Publish step (approval poster — unchanged)

Called by `execute_loopback_for_provenance()` when a post is approved and published.

- **Operation:** Append-only. Creates a new entry with `tweet_id`, `url`, `published_at`, `type`, `status`.
- **Analytics fields:** All `null` (omitted from YAML via `skip_serializing_if`).
- **Summary fields:** Not touched. If they don't exist, they remain absent.
- **Idempotency:** If `tweet_id` already exists in the array, skip (existing behavior).

### Forge sync step (new — Session 08)

Called by the Forge sync job after fetching analytics from the X API.

- **Operation:** Match-and-update. Find the entry where `tweet_id` matches, update analytics fields in place.
- **If no match:** Skip. Forge never creates entries — that is the publish step's responsibility. This prevents orphan analytics entries for tweets posted outside Tuitbot.
- **Analytics fields:** Set `impressions`, `likes`, `retweets`, `replies`, `engagement_rate`, `performance_score`, `synced_at`.
- **Summary fields:** Recomputed from the full `tuitbot:` array after every entry update.
- **Thread entries:** See `forge-thread-contract.md` for aggregation rules.

### Implementation: `update_entry_analytics()`

A new function alongside `write_metadata_to_file()`:

```
fn update_entry_analytics(path, tweet_id, analytics) -> Result<UpdateResult>
```

Where `UpdateResult` is:
- `Updated` — entry found and analytics written
- `EntryNotFound` — no entry with that `tweet_id` (skip, don't error)
- `FileNotFound` — source file doesn't exist on disk

The function:
1. Reads the file and parses frontmatter.
2. Finds the `tuitbot:` entry matching `tweet_id`.
3. Updates analytics fields on the matched entry.
4. Recomputes summary fields from all entries.
5. Serializes and writes the file.

---

## Idempotency Rules

| Scenario | Behavior |
|----------|----------|
| Publish: `tweet_id` not in array | Append new entry (analytics = null) |
| Publish: `tweet_id` already in array | Skip (no-op, return `AlreadyPresent`) |
| Forge sync: `tweet_id` in array | Update analytics fields in place |
| Forge sync: `tweet_id` not in array | Skip (no-op, return `EntryNotFound`) |
| Forge sync: file doesn't exist | Skip (no-op, return `FileNotFound`) |
| Forge sync: multiple entries with same `tweet_id` | Update the first match only (should never happen, but defensive) |
| Summary recomputation | Always runs after any Forge sync update; deterministic from current entries |

### Ordering guarantees

- Entries in the `tuitbot:` array are ordered by `published_at` (oldest first).
- Forge sync preserves entry order; it only mutates analytics fields on an existing entry.
- New entries from the publish step are always appended to the end.

### Entry cap

The `tuitbot:` array is capped at 20 entries per note. If the publish step would exceed this:
1. Remove the oldest entry (by `published_at`).
2. Append the new entry.
3. Recompute summary fields.

This prevents frontmatter bloat on prolific notes.

---

## YAML Examples

### 1. Bare publish — no analytics yet

```yaml
---
title: "Why Static Types Catch More Than Linters"
tags: [rust, programming]
tuitbot:
  - tweet_id: "1903847562910384756"
    url: "https://x.com/i/status/1903847562910384756"
    published_at: "2026-03-22T14:30:00Z"
    type: tweet
    status: posted
---

The argument for static types goes beyond...
```

### 2. After first Forge sync

```yaml
---
title: "Why Static Types Catch More Than Linters"
tags: [rust, programming]
tuitbot_social_performance: medium
tuitbot_best_post_impressions: 4820
tuitbot_best_post_url: "https://x.com/i/status/1903847562910384756"
tuitbot_last_synced_at: "2026-03-23T02:00:00Z"
tuitbot:
  - tweet_id: "1903847562910384756"
    url: "https://x.com/i/status/1903847562910384756"
    published_at: "2026-03-22T14:30:00Z"
    type: tweet
    status: posted
    impressions: 4820
    likes: 47
    retweets: 12
    replies: 8
    engagement_rate: 1.39
    performance_score: 72.5
    synced_at: "2026-03-23T02:00:00Z"
---

The argument for static types goes beyond...
```

### 3. After second Forge sync (updated metrics)

```yaml
---
title: "Why Static Types Catch More Than Linters"
tags: [rust, programming]
tuitbot_social_performance: high
tuitbot_best_post_impressions: 15200
tuitbot_best_post_url: "https://x.com/i/status/1903847562910384756"
tuitbot_last_synced_at: "2026-03-24T02:00:00Z"
tuitbot:
  - tweet_id: "1903847562910384756"
    url: "https://x.com/i/status/1903847562910384756"
    published_at: "2026-03-22T14:30:00Z"
    type: tweet
    status: posted
    impressions: 15200
    likes: 312
    retweets: 89
    replies: 41
    engagement_rate: 2.91
    performance_score: 91.3
    synced_at: "2026-03-24T02:00:00Z"
---

The argument for static types goes beyond...
```

### 4. Note with multiple posts

```yaml
---
title: "Pricing Psychology for Developer Tools"
tags: [saas, pricing]
tuitbot_social_performance: high
tuitbot_best_post_impressions: 23100
tuitbot_best_post_url: "https://x.com/i/status/1903900000000000002"
tuitbot_last_synced_at: "2026-03-25T02:00:00Z"
tuitbot:
  - tweet_id: "1903900000000000001"
    url: "https://x.com/i/status/1903900000000000001"
    published_at: "2026-03-20T10:00:00Z"
    type: tweet
    status: posted
    impressions: 1200
    likes: 15
    retweets: 3
    replies: 2
    engagement_rate: 1.67
    performance_score: 45.2
    synced_at: "2026-03-25T02:00:00Z"
  - tweet_id: "1903900000000000002"
    url: "https://x.com/i/status/1903900000000000002"
    published_at: "2026-03-22T16:00:00Z"
    type: tweet
    status: posted
    impressions: 23100
    likes: 580
    retweets: 145
    replies: 67
    engagement_rate: 3.43
    performance_score: 94.7
    synced_at: "2026-03-25T02:00:00Z"
---

When pricing developer tools, the anchor effect...
```

---

## Backward Compatibility

| Scenario | Behavior |
|----------|----------|
| Old `LoopBackEntry` without analytics fields | Valid. Analytics fields default to `None`. Summary fields absent = `"none"` tier. |
| Old `TuitbotFrontMatter` without summary keys | Valid. Summary keys read from `other` mapping; absent = not yet synced. |
| New `LoopBackEntry` read by old code | `#[serde(flatten)]` on `TuitbotFrontMatter` captures unknown fields in `other`. Old code ignores analytics fields. |
| Note with only publish entries (no Forge sync) | Summary fields absent. Dataview queries treat missing `tuitbot_social_performance` as `"none"`. |
| Note with no `tuitbot:` key at all | Not a Tuitbot-managed note. Ignored by Forge sync. |

---

## Compatibility with Existing Writeback Path

The current `write_metadata_to_file()` function remains unchanged:
- It creates entries with the 6 existing fields only.
- New analytics fields are `None` and omitted via `skip_serializing_if`.
- Summary fields are not touched by the publish step.

The new `update_entry_analytics()` function:
- Only modifies entries that already exist (created by the publish step).
- Only writes analytics fields and summary keys.
- Preserves all other frontmatter (user-defined keys, tags, etc.) via the `other` mapping.

### Forge writeback targeting

Forge determines which note to write back to using provenance:

```sql
SELECT DISTINCT vpl.source_path, vpl.node_id
FROM vault_provenance_links vpl
WHERE vpl.account_id = ?
  AND vpl.entity_type = 'original_tweet'
  AND vpl.entity_id = ?
  AND vpl.source_role = 'primary_selection';
```

Only notes with `source_role = 'primary_selection'` receive Forge writeback. Accepted neighbors do not.

---

## Dataview Query Examples

### List high-performing notes

```dataview
TABLE tuitbot_social_performance AS "Performance", tuitbot_best_post_impressions AS "Impressions"
FROM "notes"
WHERE tuitbot_social_performance = "high"
SORT tuitbot_best_post_impressions DESC
```

### Notes synced in the last 7 days

```dataview
TABLE tuitbot_last_synced_at AS "Last Sync", tuitbot_best_post_url AS "Best Post"
FROM "notes"
WHERE tuitbot_last_synced_at
SORT tuitbot_last_synced_at DESC
```

### Notes never posted

```dataview
LIST
FROM "notes"
WHERE !tuitbot
```
