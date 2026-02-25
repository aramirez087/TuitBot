# Session 01 — X API v2 Coverage Matrix

> Status per X API v2 category: `implemented`, `partial`, `missing`.
> Source of truth: `XApiClient` trait (`crates/tuitbot-core/src/x_api/mod.rs`) and `XApiHttpClient` impl (`crates/tuitbot-core/src/x_api/client.rs`).

## Coverage Summary

| Category | Status | Trait Methods | HTTP Endpoints | Notes |
|----------|--------|---------------|----------------|-------|
| **Auth / Me** | **implemented** | `get_me()` | `GET /users/me` | Full profile + `public_metrics`. Token renewal via `access_token_lock()`. |
| **Tweet Read** | **implemented** | `get_tweet()` | `GET /tweets/{id}` | Full tweet data with `public_metrics`, `author_id`, `created_at`, `conversation_id`. |
| **Tweet Create** | **implemented** | `post_tweet()`, `post_tweet_with_media()` | `POST /tweets` | Plain text + media variants. Policy-gated. |
| **Tweet Reply** | **implemented** | `reply_to_tweet()`, `reply_to_tweet_with_media()` | `POST /tweets` (with `reply.in_reply_to_tweet_id`) | Plain text + media variants. Policy-gated. |
| **Tweet Quote** | **implemented** | `quote_tweet()` | `POST /tweets` (with `quote_tweet_id`) | Text-only. Media not forwarded to quote tweets. Default trait returns error — `XApiHttpClient` overrides. |
| **Tweet Delete** | **implemented** | `delete_tweet()` | `DELETE /tweets/{id}` | Always policy-gated (requires approval). Default trait returns error — `XApiHttpClient` overrides. |
| **Thread** | **implemented** | Loop: `post_tweet()` → `reply_to_tweet()` | `POST /tweets` (chained) | MCP tool validates all tweets before posting. Partial failure returns posted IDs + failed index. |
| **Search** | **partial** | `search_tweets()` | `GET /tweets/search/recent` | Recent search only (7-day window). **Missing:** `GET /tweets/search/all` (full-archive, requires Academic/Pro tier). |
| **Timelines** | **implemented** | `get_home_timeline()`, `get_user_tweets()`, `get_mentions()` | `GET /users/{id}/timelines/reverse_chronological`, `GET /users/{id}/tweets`, `GET /users/{id}/mentions` | All support `pagination_token`. Home timeline requires `authenticated_user_id`. |
| **Users Lookup** | **partial** | `get_me()`, `get_user_by_username()` | `GET /users/me`, `GET /users/by/username/{username}` | **Missing:** `GET /users/{id}` (by ID), `GET /users` (batch lookup by IDs), `GET /users/by` (batch by usernames). |
| **Follows Actions** | **implemented** | `follow_user()`, `unfollow_user()` | `POST /users/{id}/following`, `DELETE /users/{id}/following/{target_id}` | Policy-gated. Require `authenticated_user_id`. |
| **Follows Graph** | **partial** | — | — | Follow/unfollow actions work. **Missing:** `GET /users/{id}/followers`, `GET /users/{id}/following` (list endpoints). |
| **Likes** | **partial** | `like_tweet()` | `POST /users/{id}/likes` | Policy-gated. **Missing:** `DELETE /users/{id}/likes/{tweet_id}` (unlike), `GET /users/{id}/liked_tweets`. |
| **Retweets** | **implemented** | `retweet()`, `unretweet()` | `POST /users/{id}/retweets`, `DELETE /users/{id}/retweets/{tweet_id}` | Full CRUD. Policy-gated. |
| **Bookmarks** | **missing** | — | — | No endpoints. **Missing:** `POST /users/{id}/bookmarks`, `DELETE /users/{id}/bookmarks/{tweet_id}`, `GET /users/{id}/bookmarks`. |
| **Media Upload** | **implemented** | `upload_media()` | v1.1 `POST /media/upload` | Simple + chunked upload. Supports image (.jpg/.png/.webp), GIF, video (.mp4). `infer_media_type()` from extension. |
| **Usage / Rate Visibility** | **implemented** | (header parsing) | All endpoints | Rate limit headers parsed on every response. DB-tracked cost estimation. Daily/endpoint breakdowns via `storage::x_api_usage`. |
| **Pagination** | **implemented** | All list endpoints | `pagination_token` + `since_id` | All paginated endpoints (search, mentions, user_tweets, home_timeline) accept `pagination_token`. Response includes `next_token`. |

## Detailed Gap Analysis

### `partial` Categories

#### Search
- **Implemented:** `GET /tweets/search/recent` — 7-day window, up to 100 results per page.
- **Missing:** `GET /tweets/search/all` — Full-archive search. Requires Academic Research or Pro tier access. Useful for historical analysis but niche use case.

#### Users Lookup
- **Implemented:** `get_me()` (self), `get_user_by_username()` (single user by handle).
- **Missing:**
  - `GET /users/{id}` — Look up user by numeric ID. Required for resolving `author_id` fields from tweet data.
  - `GET /users` — Batch lookup by up to 100 user IDs.
  - `GET /users/by` — Batch lookup by up to 100 usernames.

#### Follows Graph
- **Implemented:** `follow_user()`, `unfollow_user()` — mutation actions.
- **Missing:**
  - `GET /users/{id}/followers` — Paginated list of followers. Needed for audience analysis.
  - `GET /users/{id}/following` — Paginated list of accounts followed. Needed for relationship mapping.

#### Likes
- **Implemented:** `like_tweet()` — like action.
- **Missing:**
  - `DELETE /users/{id}/likes/{tweet_id}` — Unlike a tweet. No `unlike_tweet()` in trait or impl.
  - `GET /users/{id}/liked_tweets` — List tweets liked by a user.

### `missing` Categories

#### Bookmarks
No implementation at all. X API v2 supports:
- `POST /users/{id}/bookmarks` — Bookmark a tweet.
- `DELETE /users/{id}/bookmarks/{tweet_id}` — Remove bookmark.
- `GET /users/{id}/bookmarks` — List bookmarked tweets.

## Trait Default vs Override Status

Several `XApiClient` trait methods have default implementations that return errors. The `XApiHttpClient` concrete implementation overrides all of them:

| Trait Method | Default Returns Error? | XApiHttpClient Overrides? | Risk |
|---|---|---|---|
| `search_tweets()` | No (required) | Yes | None |
| `get_mentions()` | No (required) | Yes | None |
| `post_tweet()` | No (required) | Yes | None |
| `reply_to_tweet()` | No (required) | Yes | None |
| `get_tweet()` | No (required) | Yes | None |
| `get_me()` | No (required) | Yes | None |
| `get_user_tweets()` | No (required) | Yes | None |
| `get_user_by_username()` | No (required) | Yes | None |
| `upload_media()` | **Yes** — "not implemented" | **Yes** | Low — mock tests may miss |
| `post_tweet_with_media()` | **Yes** — delegates to `post_tweet()` (ignores media) | **Yes** | Low — fallback is lossy |
| `reply_to_tweet_with_media()` | **Yes** — delegates to `reply_to_tweet()` (ignores media) | **Yes** | Low — fallback is lossy |
| `quote_tweet()` | **Yes** — "not implemented" | **Yes** | **Medium** — verify test coverage |
| `like_tweet()` | **Yes** — "not implemented" | **Yes** | Low |
| `follow_user()` | **Yes** — "not implemented" | **Yes** | Low |
| `unfollow_user()` | **Yes** — "not implemented" | **Yes** | Low |
| `retweet()` | **Yes** — "not implemented" | **Yes** | Low |
| `unretweet()` | **Yes** — "not implemented" | **Yes** | Low |
| `delete_tweet()` | **Yes** — "not implemented" | **Yes** | Low |
| `get_home_timeline()` | **Yes** — "not implemented" | **Yes** | Low |
