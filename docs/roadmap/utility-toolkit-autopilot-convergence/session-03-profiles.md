# Session 03: Utility Profile Architecture

**Date:** 2026-02-26
**Session:** 03 of 08
**Branch:** `feat/mcp_final`

---

## Profile Model (6 Profiles)

| Profile | Layer | State Type | DB | LLM | Mutations | Tool Count |
|---------|-------|-----------|-----|-----|-----------|------------|
| `readonly` | Workflow | `SharedReadonlyState` | No | No | No | 10 |
| `api-readonly` | Workflow | `SharedReadonlyState` | No | No | No | 20 |
| `write` | Workflow | `SharedState` | Yes | Yes | Yes | All typed |
| `admin` | Workflow | `SharedState` | Yes | Yes | Yes | All typed + universal |
| `utility-readonly` | Toolkit | `SharedReadonlyState` | No | No | No | 15 |
| `utility-write` | Toolkit | `SharedReadonlyState` | No | No | Yes | 67 |

---

## Utility Profile Design

### Principles

1. **Flat surface**: No workflow composites, no approval routing, no analytics, no context management.
2. **Toolkit-only**: Every tool calls `tuitbot_core::toolkit::*` directly — no kernel, no provider chain.
3. **No DB/LLM dependency**: Utility profiles use `SharedReadonlyState` (X client + config only).
4. **Raw mutations**: `utility-write` exposes post/reply/quote/delete/thread/engage without policy gate or audit. The calling agent owns responsibility.
5. **Superset relationship**: `utility-write` includes every tool in `utility-readonly` plus writes, engages, extended reads, and media.

### Tool Categories by Profile

**utility-readonly (15 tools):**
- Core X reads: `get_tweet_by_id`, `x_get_user_by_username`, `x_search_tweets`, `x_get_user_mentions`, `x_get_user_tweets`, `x_get_home_timeline`, `x_get_user_by_id`
- Utils: `get_config`, `validate_config`, `score_tweet`
- Health: `health_check`
- Spec-generated reads: additional read endpoints from spec pack

**utility-write (67 tools):**
- Everything in utility-readonly
- Extended reads: `x_get_followers`, `x_get_following`, `x_get_liked_tweets`, `x_get_bookmarks`, `x_get_users_by_ids`, `x_get_tweet_liking_users`
- Writes: `x_post_tweet`, `x_reply_to_tweet`, `x_quote_tweet`, `x_delete_tweet`, `x_post_thread`
- Engages: `x_like_tweet`, `x_unlike_tweet`, `x_follow_user`, `x_unfollow_user`, `x_retweet`, `x_unretweet`, `x_bookmark_tweet`, `x_unbookmark_tweet`
- Media: `x_upload_media`
- Spec-generated mutation endpoints

---

## Implementation Architecture

### Server Modules

```
crates/tuitbot-mcp/src/server/
├── mod.rs                  ← exports all 6 server types
├── write.rs                ← WriteMcpServer (workflow)
├── admin.rs                ← AdminMcpServer (workflow)
├── readonly.rs             ← ReadonlyMcpServer (workflow)
├── api_readonly.rs         ← ApiReadonlyMcpServer (workflow)
├── toolkit_response.rs     ← shared error mapping for utility servers
├── utility_readonly.rs     ← UtilityReadonlyMcpServer (toolkit)
└── utility_write.rs        ← UtilityWriteMcpServer (toolkit)
```

### Error Handling

Utility servers use `toolkit_response.rs` which maps `ToolkitError` → `CallToolResult`:
- `ToolkitError::XApi(RateLimited)` → `ErrorCode::XRateLimited` with `retry_after_ms`
- `ToolkitError::XApi(AuthExpired)` → `ErrorCode::XAuthExpired`
- `ToolkitError::XApi(Forbidden)` → `ErrorCode::XForbidden`
- `ToolkitError::XApi(Network)` → `ErrorCode::XNetworkError`
- `ToolkitError::InvalidInput` → `ErrorCode::InvalidInput`
- `ToolkitError::TweetTooLong` → `ErrorCode::TweetTooLong`
- `ToolkitError::ThreadPartialFailure` → `ErrorCode::ThreadPartialFailure`
- `ToolkitError::UnsupportedMediaType` → `ErrorCode::UnsupportedMediaType`
- `ToolkitError::MediaTooLarge` → `ErrorCode::MediaUploadError`

No audit guard, no policy decisions, no timing metadata — simpler than the workflow path.

### State Initialization

Both utility profiles reuse `init_readonly_state()` from `lib.rs`:
- Loads X API tokens (required — fails fast if missing)
- Verifies connectivity via `get_me()`
- No DB initialization, no LLM provider creation
- Lightweight startup compared to write/admin profiles

---

## Manifest Integration

### Profile Constants (manifest.rs)

```rust
const ALL_SIX: &[Profile]     // All profiles (reads + scoring + config + health)
const WRITE_UP_AND_UTIL_WRITE  // Write + Admin + UtilityWrite (mutations)
const WRITE_UP_AND_API_RO_AND_UTIL_WRITE  // + ApiReadonly (extended reads)
```

### Lane Assignment

Tools available in utility profiles use `Lane::Shared`:
- Read tools: always `Lane::Shared` (unchanged)
- Write/engage/media tools: changed from `Lane::Workflow` to `Lane::Shared` when utility profiles are included
- Workflow-only tools (analytics, approval, content, composite, etc.): remain `Lane::Workflow`

### Spec Generator Update

`spec/generator.rs` now checks if an endpoint's profiles include any utility profile. If so, the generated `ToolEntry` uses `Lane::Shared` and `requires_db: false` instead of the default workflow settings.

---

## Boundary Tests

Seven new tests in `manifest.rs` enforce profile isolation:

| Test | Assertion |
|------|-----------|
| `utility_readonly_contains_no_workflow_tools` | Zero `Lane::Workflow` tools in utility-readonly |
| `utility_write_contains_no_workflow_tools` | Zero `Lane::Workflow` tools in utility-write |
| `utility_profiles_require_no_db_or_llm` | No tool in either utility profile requires DB or LLM |
| `utility_readonly_contains_no_mutations` | Zero mutation tools in utility-readonly |
| `utility_profile_tool_counts` | utility-readonly >= 10, utility-write > utility-readonly |
| `utility_write_is_superset_of_utility_readonly_tools` | Every utility-readonly tool exists in utility-write |
| `write_utility_manifests` (ignored) | Generates `docs/generated/mcp-manifest-*.json` |
