# Session 04 Handoff: Close High-Value Endpoint Gaps

## What Was Done

Added 10 new X API v2 tools to the MCP server, closing the P0/P1 gaps identified in Session 01's priority backlog. All tools are registered in both the API profile and Workflow profile.

### New Read Tools (7)
1. **x_get_followers** - Get paginated follower list for any user
2. **x_get_following** - Get paginated following list for any user
3. **x_get_user_by_id** - Look up user by numeric ID (complements existing username lookup)
4. **x_get_liked_tweets** - Get tweets a user has liked
5. **x_get_bookmarks** - Get authenticated user's bookmarks
6. **x_get_users_by_ids** - Batch user lookup (1-100 IDs, validated)
7. **x_get_tweet_liking_users** - Get users who liked a specific tweet

### New Engage Tools (3)
8. **x_unlike_tweet** - Unlike a previously liked tweet
9. **x_bookmark_tweet** - Bookmark a tweet
10. **x_unbookmark_tweet** - Remove a bookmark

## Files Modified (21)

### Core Crate (5 files)
- `core/x_api/types.rs` - Added `UsersResponse`, `UsersMeta`, `BookmarkTweetRequest`, `bookmarked` alias
- `core/x_api/mod.rs` - 10 new trait methods on `XApiClient`
- `core/x_api/client.rs` - 10 HTTP implementations + 10 wiremock tests
- `core/x_api/scopes.rs` - Added `bookmark.read`/`bookmark.write` scopes + Bookmarks feature mapping
- `core/storage/x_api_usage.rs` - Cost estimation rules for bookmarks, liked_tweets, liking_users

### MCP Crate (13 files)
- `mcp/provider/mod.rs` - 7 new `SocialReadProvider` trait methods
- `mcp/provider/x_api.rs` - 7 adapter implementations
- `mcp/kernel/read.rs` - 7 kernel read functions (+ `get_users_by_ids` with 1-100 ID validation)
- `mcp/kernel/engage.rs` - 3 kernel engage functions
- `mcp/kernel/tests.rs` - 14 new tests + mock updates
- `mcp/requests.rs` - 10 new request structs
- `mcp/server/api.rs` - 10 new tool registrations (Read: 7->14, Engage: 5->8)
- `mcp/server/workflow.rs` - 10 new tool registrations
- `mcp/tools/x_actions/mod.rs` - Updated re-exports
- `mcp/tools/x_actions/read.rs` - 7 delegation functions
- `mcp/tools/x_actions/engage.rs` - 3 delegation functions
- `mcp/tools/x_actions/tests/mod.rs` - Mock client updates
- `mcp/tools/capabilities.rs` - 10 new entries (total: 21)

### CLI Crate (1 file)
- `cli/commands/test.rs` - Updated `valid_tokens()` fixture with new bookmark scopes

### Artifacts (3 files)
- `roadmap/artifacts/session-04-added-tools-matrix.md`
- `roadmap/artifacts/session-04-endpoint-test-report.md`
- `roadmap/artifacts/session-04-handoff.md`

## Key Design Decisions

1. **`UsersResponse` is a distinct type** - Followers/following/batch endpoints return `Vec<User>`, not `Vec<Tweet>`, so reusing `SearchResponse` would be incorrect.
2. **`get_liked_tweets` and `get_bookmarks` return `SearchResponse`** - These endpoints return tweets, matching the existing pattern.
3. **`GetBookmarksRequest` has no `user_id` field** - The server injects the authenticated user ID, matching `x_get_user_mentions` pattern.
4. **`get_users_by_ids` validates 1-100 IDs** - Returns `invalid_input` error for empty or oversized batches.
5. **Bookmark scopes added to `REQUIRED_SCOPES`** - This is a breaking change for existing auth tokens; users will need to re-authenticate to grant bookmark scopes.
6. **Workflow engage tools use policy gating** - `unlike_tweet`, `bookmark_tweet`, `unbookmark_tweet` all go through the policy gate + mutation recording pattern.

## Tool Count After Session 04

- **API profile**: 24 -> 34 tools
- **Workflow profile**: 54 -> 64 tools

## What's Next

Potential Session 05 topics from the backlog:
- Lists API support (create, manage, get list tweets)
- Spaces API support
- DM read support (if scopes allow)
- Composite tools leveraging new endpoints (e.g., "analyze my audience" using followers + liked tweets)
