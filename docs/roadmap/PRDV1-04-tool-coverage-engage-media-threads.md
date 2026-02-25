# PRDV1-04: Tool Coverage (Engage + Media + Threads)

## Goal

Ship the missing MCP/X capabilities required by PRD v1 so TuitBot is feature-
complete for read/write/engage/media/thread workflows.

## Already built (do not redo)

- Read tools: tweet, user lookup, search, mentions, user tweets.
- Write tools: post, reply, quote.
- Engage tool: like.
- Media upload exists in server flow and core X API media module.
- URL-aware length utility exists (`tweet_weighted_len`).

## Missing v1 capabilities

1. Read
   - `get_home_timeline` (if tier allows)
   - unified usage/rate tool for MCP (`get_x_usage`)
2. Write
   - `delete_tweet` (always policy-gated)
   - `post_thread` first-class mutation
3. Engage
   - `retweet`
   - `unretweet`
4. Media
   - direct MCP `upload_media`
   - attach media IDs in post/reply/quote/thread paths
5. Pagination consistency
   - use `next_token` style paging where X supports it

## Primary code touchpoints

- `crates/tuitbot-core/src/x_api/mod.rs`
- `crates/tuitbot-core/src/x_api/client.rs`
- `crates/tuitbot-core/src/x_api/types.rs`
- `crates/tuitbot-mcp/src/requests.rs`
- `crates/tuitbot-mcp/src/server.rs`
- `crates/tuitbot-mcp/src/tools/x_actions.rs`
- `crates/tuitbot-core/src/content/length.rs`
- `docs/mcp-reference.md`

## Implementation tasks

1. Extend `XApiClient` trait.
   - Add retweet/unretweet/delete/home timeline methods.
   - Add thread helper primitives where needed.
2. Implement HTTP client methods + error mappings.
   - Map 429/403/401 consistently with typed errors.
3. Add MCP request structs and tool handlers.
   - Register tools in `server.rs`.
   - Add tool docs and examples.
4. Implement MCP media upload tool.
   - Input: local path + media type.
   - Output: media ID + metadata.
5. Add media-aware mutations.
   - `x_post_tweet`, `x_reply_to_tweet`, `x_quote_tweet`, `x_post_thread` accept optional media IDs.
6. Add URL-aware length validation on all mutation entrypoints.
   - Reject with typed validation errors pre-flight.
7. Unify pagination interfaces.
   - Add `pagination_token` request parameters where applicable.
   - Return `next_token` consistently.

## Acceptance criteria

- New tools are callable via MCP with docs + tests.
- Delete is always policy-gated and routed/blocked by default template.
- Thread posting is first-class (single tool call with ordered tweets).
- Pagination output and inputs are consistent across read tools.
- All mutation tools enforce URL-aware tweet length validation.

## Verification commands

```bash
cargo test -p tuitbot-core x_api
cargo test -p tuitbot-mcp x_actions
cargo test -p tuitbot-mcp server
```

## Out of scope

- Agency multi-account quotas (PRDV1-09).
- QA hard/soft flag system (PRDV1-06).
