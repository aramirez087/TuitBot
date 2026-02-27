# Session 05: Autopilot Loops → Toolkit Convergence

**Date:** 2026-02-26
**Session:** 05 of 08
**Branch:** `feat/mcp_final`

---

## Objective

Refactor all autopilot adapter impls to route X API calls through the toolkit
layer, enforcing AD-06 (autopilot MUST NOT call `XApiClient` directly).

## Architecture Before → After

### Before (direct X API calls)

```
Autopilot Loop → Port Trait → Adapter → Arc<XApiHttpClient>.method()
```

### After (toolkit-routed)

```
Autopilot Loop → Port Trait → Adapter → toolkit::read/write/media → &dyn XApiClient
```

The loops themselves are unchanged — they still depend on the same port traits
defined in `loop_helpers.rs`. Only the adapter implementations changed.

## What Changed

### 1. `adapters.rs` — Six X API adapter structs refactored

All six adapters changed from `Arc<XApiHttpClient>` (concrete) to
`Arc<dyn XApiClient>` (trait object), and all delegate through toolkit functions:

| Adapter | Toolkit calls |
|---------|--------------|
| `XApiSearchAdapter` | `toolkit::read::search_tweets` |
| `XApiMentionsAdapter` | `toolkit::read::get_mentions` |
| `XApiTargetAdapter` | `toolkit::read::get_user_tweets`, `toolkit::read::get_user_by_username` |
| `XApiProfileAdapter` | `toolkit::read::get_me`, `toolkit::read::get_tweet` |
| `XApiPostExecutorAdapter` | `toolkit::write::reply_to_tweet`, `toolkit::write::post_tweet` |
| `XApiThreadPosterAdapter` | `toolkit::write::post_tweet`, `toolkit::write::reply_to_tweet` |

### 2. `adapters.rs` — Error mapping updated

Three error mappers replaced:

| Before | After |
|--------|-------|
| `xapi_to_loop_error(XApiError)` | `toolkit_to_loop_error(ToolkitError)` |
| `xapi_to_content_error(XApiError)` | `toolkit_to_content_error(ToolkitError)` |
| `xapi_to_analytics_error(XApiError)` | `toolkit_to_analytics_error(ToolkitError)` |

The new mappers pattern-match through `ToolkitError::XApi(XApiError)` to
extract rate limiting, auth expiry, and network errors — preserving the
same `LoopError` / `ContentLoopError` / `AnalyticsError` discriminants.

Non-XApi toolkit errors (e.g., `InvalidInput`, `TweetTooLong`) map to
`LoopError::Other` / `ContentLoopError::PostFailed`.

### 3. `approval_poster.rs` — Routed through toolkit

- Signature: `Arc<XApiHttpClient>` → `Arc<dyn XApiClient>`
- `post_reply()` → `toolkit::write::reply_to_tweet`
- `post_tweet()` → `toolkit::write::post_tweet`
- `upload_media()` → `toolkit::media::infer_media_type` + `toolkit::media::upload_media`
  (replaced manual extension matching with toolkit's `infer_media_type`)

### 4. CLI `deps.rs` — Adapter construction updated

Single cast point: `Arc<XApiHttpClient>` → `Arc<dyn XApiClient>`, shared
across all adapter constructors. The concrete `x_client` field stays for
token refresh (AD-06 infrastructure exception).

### 5. CLI `run.rs` — Approval poster cast added

The `run_approval_poster` call now passes `Arc<dyn XApiClient>` via explicit cast.

### 6. Tests — 12 new adapter tests

New `#[cfg(test)]` module in `adapters.rs` with a `MockXApiClient` that
verifies all six adapters correctly route through toolkit:

- `search_adapter_routes_through_toolkit`
- `mentions_adapter_routes_through_toolkit`
- `target_adapter_fetch_routes_through_toolkit`
- `target_adapter_lookup_routes_through_toolkit`
- `profile_adapter_routes_through_toolkit`
- `engagement_adapter_routes_through_toolkit`
- `post_executor_reply_routes_through_toolkit`
- `post_executor_tweet_routes_through_toolkit`
- `thread_poster_post_routes_through_toolkit`
- `thread_poster_reply_routes_through_toolkit`
- `toolkit_error_maps_to_loop_error` (rate limit passthrough)
- `empty_id_triggers_toolkit_validation` (toolkit validation propagates)

## What Did NOT Change

- **Loop files** (`discovery_loop.rs`, `mentions_loop.rs`, `content_loop.rs`,
  `thread_loop.rs`) — these only use port traits, no direct X API calls
- **Port traits** in `loop_helpers.rs` — unchanged interface
- **Token refresh loop** in `mod.rs` — allowed infrastructure exception per AD-06
- **LLM adapters**, **scoring adapter**, **safety adapters**, **storage adapters**,
  **posting queue adapters** — not X API consumers, no changes needed

## AD-06 Compliance Audit

| X API call site | Routed through toolkit? | Notes |
|----------------|------------------------|-------|
| `XApiSearchAdapter.search_tweets` | Yes | `toolkit::read::search_tweets` |
| `XApiMentionsAdapter.get_mentions` | Yes | `toolkit::read::get_mentions` |
| `XApiTargetAdapter.get_user_tweets` | Yes | `toolkit::read::get_user_tweets` |
| `XApiTargetAdapter.get_user_by_username` | Yes | `toolkit::read::get_user_by_username` |
| `XApiProfileAdapter.get_me` | Yes | `toolkit::read::get_me` |
| `XApiProfileAdapter.get_tweet` | Yes | `toolkit::read::get_tweet` |
| `XApiPostExecutorAdapter.execute_reply` | Yes | `toolkit::write::reply_to_tweet` |
| `XApiPostExecutorAdapter.execute_tweet` | Yes | `toolkit::write::post_tweet` |
| `XApiThreadPosterAdapter.post_tweet` | Yes | `toolkit::write::post_tweet` |
| `XApiThreadPosterAdapter.reply_to_tweet` | Yes | `toolkit::write::reply_to_tweet` |
| `approval_poster::post_reply` | Yes | `toolkit::write::reply_to_tweet` |
| `approval_poster::post_tweet` | Yes | `toolkit::write::post_tweet` |
| `approval_poster::upload_media` | Yes | `toolkit::media::upload_media` |
| `run_token_refresh_loop` | **No** | Infrastructure exception (AD-06) |
