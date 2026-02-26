# Session 04: Added Tools Matrix

## New Tools (10)

| # | Tool Name | Category | X API Endpoint | Method | Profile |
|---|-----------|----------|----------------|--------|---------|
| 1 | `x_unlike_tweet` | Engage | `/users/{id}/likes/{tweet_id}` | DELETE | API + Workflow |
| 2 | `x_get_followers` | Read | `/users/{id}/followers` | GET | API + Workflow |
| 3 | `x_get_following` | Read | `/users/{id}/following` | GET | API + Workflow |
| 4 | `x_get_user_by_id` | Read | `/users/{id}` | GET | API + Workflow |
| 5 | `x_get_liked_tweets` | Read | `/users/{id}/liked_tweets` | GET | API + Workflow |
| 6 | `x_get_bookmarks` | Read | `/users/{id}/bookmarks` | GET | API + Workflow |
| 7 | `x_bookmark_tweet` | Engage | `/users/{id}/bookmarks` | POST | API + Workflow |
| 8 | `x_unbookmark_tweet` | Engage | `/users/{id}/bookmarks/{tweet_id}` | DELETE | API + Workflow |
| 9 | `x_get_users_by_ids` | Read | `/users?ids=1,2,3` | GET | API + Workflow |
| 10 | `x_get_tweet_liking_users` | Read | `/tweets/{id}/liking_users` | GET | API + Workflow |

## New Types

| Type | Purpose |
|------|---------|
| `UsersResponse` | Response for endpoints returning `Vec<User>` (followers, following, batch lookup, liking users) |
| `UsersMeta` | Pagination metadata for user list responses |
| `BookmarkTweetRequest` | Request body for bookmark POST endpoint |

## OAuth Scopes Added

- `bookmark.read` - Required for reading bookmarks
- `bookmark.write` - Required for creating/deleting bookmarks

## Feature-Scope Mapping Added

- **Bookmarks**: `bookmark.read`, `bookmark.write`, `users.read`

## Cost Estimation Rules Added

| Endpoint Pattern | Method | Cost |
|-----------------|--------|------|
| `/bookmarks` | GET | $0.005 |
| `/liked_tweets` | GET | $0.005 |
| `/liking_users` | GET | $0.005 |
| `/users` (batch) | GET | $0.010 |
| `/bookmarks` | POST | $0.010 |
| `/bookmarks` | DELETE | $0.010 |

## Capabilities Map

- Read tools: 5 -> 12 entries
- Mutation tools: 6 -> 9 entries
- Total direct tools in `get_capabilities`: 11 -> 21
