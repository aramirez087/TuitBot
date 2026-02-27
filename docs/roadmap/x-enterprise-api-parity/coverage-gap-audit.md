# X Enterprise API Parity — Coverage Gap Audit

**Date:** 2026-02-26
**Session:** 01 (Initial Audit)
**Branch:** `feat/mcp_x_api_coverage`

---

## 1. Current Inventory

### 1.1 Tool Counts by Profile

| Profile | Manifest Tools | Server Handler Fns | Source |
|---------|---------------|-------------------|--------|
| Readonly | 14 | 10 | `server/readonly.rs` |
| ApiReadonly | 40 | 20 | `server/api_readonly.rs` |
| Write | 104 | 68 | `server/write.rs` |
| Admin | 108 | 72 | `server/admin.rs` |
| UtilityReadonly | ~12 | — | `server/utility_readonly.rs` |
| UtilityWrite | ~50 | — | `server/utility_write.rs` |

**Source:** `crates/tuitbot-mcp/src/tools/boundary_tests.rs:242-284`

### 1.2 Layer 1 — Curated Tools (72 total, in manifest.rs)

Location: `crates/tuitbot-mcp/src/tools/manifest.rs:347-1298`

| Category | Tools | Mutation | Profiles |
|----------|-------|----------|----------|
| Analytics (7) | `get_stats`, `get_follower_trend`, `get_action_log`, `get_action_counts`, `get_recent_mutations`, `get_mutation_detail`, `get_x_usage` | No | Write+Admin |
| Policy (2) | `get_rate_limits`, `get_policy_status` | No | Write+Admin |
| Replies (2) | `get_recent_replies`, `get_reply_count_today` | No | Write+Admin |
| Discovery (3) | `list_target_accounts`, `list_unreplied_tweets`, `get_discovery_feed` | No | Write+Admin |
| Scoring (1) | `score_tweet` | No | All 6 |
| Approval (5) | `list_pending_approvals`, `get_pending_count`, `approve_item`, `reject_item`, `approve_all` | 3 Yes | Write+Admin |
| Content (4) | `generate_reply`, `generate_tweet`, `generate_thread`, `suggest_topics` | No | Write+Admin |
| Config (2) | `get_config`, `validate_config` | No | All 6 / 5 |
| Meta (2) | `get_capabilities`, `get_mode` | No | ApiReadonly+Write+Admin |
| Health (1) | `health_check` | No | All 6 |
| Compose (1) | `compose_tweet` | Yes | Write+Admin |
| X Read Core (7) | `get_tweet_by_id`, `x_get_user_by_username`, `x_search_tweets`, `x_get_user_mentions`, `x_get_user_tweets`, `x_get_home_timeline`, `x_get_user_by_id` | No | All 6 |
| X Read Extended (6) | `x_get_followers`, `x_get_following`, `x_get_liked_tweets`, `x_get_bookmarks`, `x_get_users_by_ids`, `x_get_tweet_liking_users` | No | ApiRO+Write+Admin+UtilWrite |
| X Read (ApiRO only) (1) | `x_get_me` | No | ApiReadonly |
| X Write (5) | `x_post_tweet`, `x_reply_to_tweet`, `x_quote_tweet`, `x_delete_tweet`, `x_post_thread` | Yes | Write+Admin+UtilWrite |
| X Engage (8) | `x_like_tweet`, `x_unlike_tweet`, `x_follow_user`, `x_unfollow_user`, `x_retweet`, `x_unretweet`, `x_bookmark_tweet`, `x_unbookmark_tweet` | Yes | Write+Admin+UtilWrite |
| X Media (1) | `x_upload_media` | Yes | Write+Admin+UtilWrite |
| Dry Run (2) | `x_post_tweet_dry_run`, `x_post_thread_dry_run` | No | Write+Admin |
| Context (3) | `get_author_context`, `recommend_engagement_action`, `topic_performance_snapshot` | No | Write+Admin |
| Telemetry (2) | `get_mcp_tool_metrics`, `get_mcp_error_breakdown` | No | Write+Admin |
| Composite (4) | `find_reply_opportunities`, `draft_replies_for_candidates`, `propose_and_queue_replies`, `generate_thread_plan` | 1 Yes | Write+Admin |
| Universal Request (4) | `x_get`, `x_post`, `x_put`, `x_delete` | 3 Yes | Admin only |

### 1.3 Layer 2 — Spec-Generated Tools (36 total, in endpoints.rs)

Location: `crates/tuitbot-mcp/src/spec/endpoints.rs:148-891`

| Group | Tools | Mutation | Read/Mutation Split |
|-------|-------|----------|-------------------|
| Batch Lookups (3) | `x_v2_tweets_lookup`, `x_v2_users_lookup_by_usernames`, `x_v2_tweets_retweeted_by` | No | 3 read / 0 mutation |
| Tweet Metadata (4) | `x_v2_tweets_quote_tweets`, `x_v2_tweets_counts_recent`, `x_v2_tweets_hide_reply`, `x_v2_tweets_unhide_reply` | 2 Yes | 2 read / 2 mutation |
| Pin Management (2) | `x_v2_users_pin_tweet`, `x_v2_users_unpin_tweet` | Yes | 0 read / 2 mutation |
| Lists (15) | `x_v2_lists_get`, `x_v2_lists_owned`, `x_v2_lists_create`, `x_v2_lists_update`, `x_v2_lists_delete`, `x_v2_lists_tweets`, `x_v2_lists_members`, `x_v2_lists_members_add`, `x_v2_lists_members_remove`, `x_v2_lists_memberships`, `x_v2_lists_followers`, `x_v2_lists_follow`, `x_v2_lists_unfollow`, `x_v2_lists_pinned`, `x_v2_lists_pin` | 7 Yes | 8 read / 7 mutation |
| Mutes (3) | `x_v2_mutes_list`, `x_v2_mutes_create`, `x_v2_mutes_delete` | 2 Yes | 1 read / 2 mutation |
| Blocks (3) | `x_v2_blocks_list`, `x_v2_blocks_create`, `x_v2_blocks_delete` | 2 Yes | 1 read / 2 mutation |
| Spaces (6) | `x_v2_spaces_get`, `x_v2_spaces_lookup`, `x_v2_spaces_by_creator`, `x_v2_spaces_search`, `x_v2_spaces_buyers`, `x_v2_spaces_tweets` | No | 6 read / 0 mutation |

### 1.4 Host Allowlist

Location: `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs:29`

```rust
const ALLOWED_HOSTS: &[&str] = &["api.x.com", "upload.x.com", "upload.twitter.com"];
```

---

## 2. Enterprise Coverage Gaps

### 2.1 Direct Messages (DM) — ZERO COVERAGE

**X API v2 DM Endpoints (11 endpoints):**

| Endpoint | Method | Path | Read/Mutation | OAuth Scopes |
|----------|--------|------|--------------|-------------|
| Lookup DM conversations | GET | `/2/dm_conversations` | Read | `dm.read`, `users.read` |
| Get DM conversation by ID | GET | `/2/dm_conversations/{id}` | Read | `dm.read`, `users.read` |
| Get DM events by conversation | GET | `/2/dm_conversations/{id}/dm_events` | Read | `dm.read`, `users.read` |
| Get DM events for participant | GET | `/2/dm_conversations/with/{participant_id}/dm_events` | Read | `dm.read`, `users.read` |
| Get all DM events | GET | `/2/dm_events` | Read | `dm.read`, `users.read` |
| Send DM in conversation | POST | `/2/dm_conversations/{id}/messages` | Mutation | `dm.write`, `dm.read`, `users.read` |
| Send DM to participant | POST | `/2/dm_conversations/with/{participant_id}/messages` | Mutation | `dm.write`, `dm.read`, `users.read` |
| Create group DM conversation | POST | `/2/dm_conversations` | Mutation | `dm.write`, `dm.read`, `users.read` |

**Impact:** No typed DM support at all. Agents cannot read or send direct messages. Currently reachable only via admin-profile universal request tools (`x_get`/`x_post`).

**Scopes needed:** `dm.read`, `dm.write` (not currently in any curated tool scope requirements).

### 2.2 Ads / Campaign API — ZERO COVERAGE

**X Ads API v12+ Endpoints (core subset, 16 endpoints):**

| Endpoint | Method | Path | Read/Mutation | Notes |
|----------|--------|------|--------------|-------|
| Get campaigns | GET | `/12/accounts/{account_id}/campaigns` | Read | List campaigns |
| Get campaign by ID | GET | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Read | Single campaign |
| Create campaign | POST | `/12/accounts/{account_id}/campaigns` | Mutation | New campaign |
| Update campaign | PUT | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Mutation | Modify campaign |
| Delete campaign | DELETE | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Mutation | Remove campaign |
| Get line items | GET | `/12/accounts/{account_id}/line_items` | Read | Ad groups |
| Create line item | POST | `/12/accounts/{account_id}/line_items` | Mutation | New ad group |
| Get promoted tweets | GET | `/12/accounts/{account_id}/promoted_tweets` | Read | Promoted content |
| Create promoted tweet | POST | `/12/accounts/{account_id}/promoted_tweets` | Mutation | Promote tweet |
| Get targeting criteria | GET | `/12/accounts/{account_id}/targeting_criteria` | Read | Audience targeting |
| Get ad analytics | GET | `/12/stats/accounts/{account_id}` | Read | Performance data |
| Get funding instruments | GET | `/12/accounts/{account_id}/funding_instruments` | Read | Payment methods |
| Get account media | GET | `/12/accounts/{account_id}/account_media` | Read | Ad media library |
| Get ad accounts | GET | `/12/accounts` | Read | List ad accounts |
| Get account by ID | GET | `/12/accounts/{account_id}` | Read | Single account |
| Get active entities | GET | `/12/stats/accounts/{account_id}/active_entities` | Read | Active entities |

**Blockers:**
1. Host `ads-api.x.com` is NOT in the `ALLOWED_HOSTS` allowlist (`x_request/mod.rs:29`)
2. No `ToolCategory::Ads` variant exists in `manifest.rs:58-79`
3. No `ads` group defined in the spec pack
4. Ads API requires an approved Ads API developer account (separate from standard v2 access)

**Impact:** Enterprise users with Ads API access cannot manage campaigns, view ad analytics, or create promoted content through MCP tools. Only reachable via universal request tools IF `ads-api.x.com` were added to the allowlist (currently blocked).

### 2.3 Enterprise Admin / Compliance — ZERO COVERAGE

**X API v2 Compliance Endpoints (6 endpoints):**

| Endpoint | Method | Path | Read/Mutation | OAuth Scopes |
|----------|--------|------|--------------|-------------|
| Create compliance job | POST | `/2/compliance/jobs` | Mutation | `compliance.write` |
| Get compliance job | GET | `/2/compliance/jobs/{id}` | Read | `compliance.write` |
| List compliance jobs | GET | `/2/compliance/jobs` | Read | `compliance.write` |
| Get usage tweets | GET | `/2/usage/tweets` | Read | `usage.read` |

**X API v2 Enterprise Stream Endpoints (4 endpoints):**

| Endpoint | Method | Path | Read/Mutation | Notes |
|----------|--------|------|--------------|-------|
| Add stream rules | POST | `/2/tweets/search/stream/rules` | Mutation | Filtered stream rules |
| Delete stream rules | POST | `/2/tweets/search/stream/rules` (with delete body) | Mutation | Remove rules |
| Get stream rules | GET | `/2/tweets/search/stream/rules` | Read | List active rules |
| Get filtered stream | GET | `/2/tweets/search/stream` | Read | Real-time stream (SSE) |

**Impact:** No GDPR compliance tooling, no usage monitoring, no filtered stream management. All enterprise compliance workflows require manual API calls.

**Note:** Filtered stream is a long-lived SSE connection — not suitable for standard request/response tools. Stream rule management IS suitable for typed tools.

### 2.4 Missing Community/Notes Endpoints

| Endpoint | Method | Path | Read/Mutation | Notes |
|----------|--------|------|--------------|-------|
| Get Community notes | GET | `/2/tweets/{id}/notes` | Read | Community Notes on tweets |

**Impact:** Minor. Community Notes is a newer feature with limited API surface.

---

## 3. Scope Coverage Matrix

### Currently Declared Scopes (in curated + spec tools)

| Scope | Used By | Status |
|-------|---------|--------|
| `tweet.read` | Core reads, search, timeline, write tools | Covered |
| `tweet.write` | Post, reply, quote, delete, retweet, media | Covered |
| `tweet.moderate.write` | Hide/unhide reply | Covered |
| `users.read` | All user lookups, follows, mentions | Covered |
| `follows.read` | Get followers/following | Covered |
| `follows.write` | Follow/unfollow | Covered |
| `like.read` | Get liked tweets, liking users | Covered |
| `like.write` | Like/unlike | Covered |
| `bookmark.read` | Get bookmarks | Covered |
| `bookmark.write` | Bookmark/unbookmark | Covered |
| `list.read` | List reads | Covered |
| `list.write` | List mutations | Covered |
| `mute.read` | Mute list | Covered |
| `mute.write` | Mute/unmute | Covered |
| `block.read` | Block list | Covered |
| `block.write` | Block/unblock | Covered |
| `space.read` | Space reads | Covered |

### Enterprise Scopes NOT Currently Used

| Scope | Needed For | Gap Area |
|-------|-----------|----------|
| `dm.read` | Read DM conversations and events | DM |
| `dm.write` | Send DMs, create group conversations | DM |
| `compliance.write` | Create/read compliance jobs | Compliance |
| `usage.read` | API usage statistics | Admin |

---

## 4. Universal Request Safety Gap

### Current Safety Constraints (`x_request/mod.rs:26-40`)

| Constraint | Status | File:Line |
|-----------|--------|-----------|
| Host allowlist | 3 hosts only | `mod.rs:29` |
| Path validation (no traversal) | Enforced | `mod.rs:50-69` |
| SSRF protection (no IP literals) | Enforced | `mod.rs:80-86` |
| Header blocklist | 6 headers blocked | `mod.rs:32-40` |
| Admin-only access | Enforced | `manifest.rs:1250-1297` |

### Gap: `ads-api.x.com` Not in Allowlist

Adding Ads API support requires extending `ALLOWED_HOSTS` to include `ads-api.x.com`. This is a one-line change but has security implications:

- All admin-profile universal requests could then target the Ads API
- Ads API mutations can incur real financial costs (ad spend)
- Recommendation: Gate behind an explicit `ads_api_enabled` config flag in addition to admin profile requirement

---

## 5. Profile Gap Summary

| Gap Area | Typed Tools Needed | New Read Tools | New Mutation Tools | Target Profiles |
|----------|-------------------|---------------|-------------------|----------------|
| DM | 8 | 5 | 3 | Read: ApiRO+Write+Admin+UtilWrite; Write: Write+Admin+UtilWrite |
| Ads/Campaign | 16 | 9 | 7 | Admin only |
| Compliance | 4 | 3 | 1 | Admin only |
| Stream Rules | 3 | 1 | 2 | Admin only |
| **Total** | **31** | **18** | **13** | — |

### Post-Implementation Target Counts

| Profile | Current | +DM Read | +DM Write | +Ads | +Compliance | +Stream | Target |
|---------|---------|----------|-----------|------|-------------|---------|--------|
| Readonly | 14 | — | — | — | — | — | 14 |
| ApiReadonly | 40 | +5 | — | — | — | — | 45 |
| Write | 104 | +5 | +3 | — | — | — | 112 |
| Admin | 108 | +5 | +3 | +16 | +4 | +3 | 139 |
| UtilityReadonly | ~12 | — | — | — | — | — | ~12 |
| UtilityWrite | ~50 | +5 | +3 | — | — | — | ~58 |

---

## 6. File Paths Requiring Changes (Sessions 02-06)

| File | Change Type | Affected By |
|------|------------|------------|
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Add 31 new EndpointDefs | DM, Ads, Compliance, Stream |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Add `ToolCategory::Ads`, `ToolCategory::Compliance`, `ToolCategory::DirectMessage` | Ads, Compliance, DM |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs` | Add `ads-api.x.com` to ALLOWED_HOSTS | Ads |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Update tool counts, mutation denylist, profile counts | All |
| `crates/tuitbot-mcp/src/contract/error_code.rs` | Add DM/Ads/Compliance error codes if needed | DM, Ads, Compliance |
| `docs/mcp-reference.md` | Update tool counts, profile descriptions | All |
| `docs/configuration.md` | Document `ads_api_enabled` flag | Ads |
| `README.md` | Update feature list | All |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerate snapshot | All |
| `docs/generated/mcp-manifest-*.json` | Regenerate all 4-6 manifests | All |
