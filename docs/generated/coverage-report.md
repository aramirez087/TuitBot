# MCP Endpoint Coverage Report

**Generated:** 2026-02-28T14:10:11.642405+00:00

**MCP Schema:** 1.2 | **X API Spec:** 1.3.0

## Summary

| Metric | Count |
|--------|-------|
| Total tools | 140 |
| Curated (L1) | 73 |
| Generated (L2) | 67 |
| Mutation tools | 51 |
| Read-only tools | 89 |
| Requires X client | 106 |
| Requires LLM | 5 |
| Requires DB | 47 |
| Requires user auth | 99 |
| Requires elevated access | 27 |

## Test Coverage

**76/140 tools have at least one test (54.3%)**

| Test Type | Count |
|-----------|-------|
| Kernel conformance | 27 |
| Spec conformance | 31 |
| Contract envelope | 18 |
| Live (sandbox) | 9 |
| Untested | 64 |

## By Category

| Category | Total | Curated | Generated | Mutations | Tested |
|----------|-------|---------|-----------|-----------|--------|
| ads | 16 | 0 | 16 | 7 | 16 |
| analytics | 9 | 9 | 0 | 0 | 7 |
| approval | 5 | 5 | 0 | 3 | 2 |
| compliance | 7 | 0 | 7 | 3 | 7 |
| composite | 4 | 4 | 0 | 1 | 0 |
| config | 2 | 2 | 0 | 0 | 2 |
| content | 4 | 4 | 0 | 0 | 0 |
| context | 3 | 3 | 0 | 0 | 1 |
| direct_message | 8 | 0 | 8 | 3 | 8 |
| discovery | 3 | 3 | 0 | 0 | 2 |
| engage | 10 | 8 | 2 | 10 | 8 |
| health | 1 | 1 | 0 | 0 | 0 |
| list | 15 | 0 | 15 | 8 | 0 |
| media | 1 | 1 | 0 | 1 | 0 |
| meta | 2 | 2 | 0 | 0 | 0 |
| moderation | 8 | 0 | 8 | 6 | 0 |
| policy | 2 | 2 | 0 | 0 | 1 |
| read | 26 | 15 | 11 | 0 | 14 |
| scoring | 1 | 1 | 0 | 0 | 1 |
| telemetry | 2 | 2 | 0 | 0 | 2 |
| write | 11 | 11 | 0 | 9 | 5 |

## By Profile

| Profile | Total | Pre-Initiative | Delta | Mutations | Read-Only |
|---------|-------|----------------|-------|-----------|----------|
| readonly | 14 | 14 | +0 | 0 | 14 |
| api_readonly | 45 | 40 | +5 | 0 | 45 |
| write | 112 | 104 | +8 | 38 | 74 |
| admin | 139 | 108 | +31 | 51 | 88 |

## Tier-Gated Areas

Tools restricted to specific profiles:

- **admin only**: 27 tools
- **all tiers**: 14 tools
- **api_readonly+**: 31 tools
- **write+**: 68 tools

## Credential-Gated Areas

99 tools require specific credentials:

- get_tweet_by_id: [user_auth, scoped]
- x_ads_account_by_id: [user_auth, elevated_access, scoped]
- x_ads_accounts: [user_auth, elevated_access, scoped]
- x_ads_analytics: [user_auth, elevated_access, scoped]
- x_ads_campaign_by_id: [user_auth, elevated_access, scoped]
- x_ads_campaign_create: [user_auth, elevated_access, scoped]
- x_ads_campaign_delete: [user_auth, elevated_access, scoped]
- x_ads_campaign_update: [user_auth, elevated_access, scoped]
- x_ads_campaigns: [user_auth, elevated_access, scoped]
- x_ads_funding_instruments: [user_auth, elevated_access, scoped]
- x_ads_line_item_create: [user_auth, elevated_access, scoped]
- x_ads_line_items: [user_auth, elevated_access, scoped]
- x_ads_promoted_tweet_create: [user_auth, elevated_access, scoped]
- x_ads_promoted_tweets: [user_auth, elevated_access, scoped]
- x_ads_targeting_create: [user_auth, elevated_access, scoped]
- x_ads_targeting_criteria: [user_auth, elevated_access, scoped]
- x_ads_targeting_delete: [user_auth, elevated_access, scoped]
- x_bookmark_tweet: [user_auth, scoped]
- x_delete: [user_auth, elevated_access]
- x_delete_tweet: [user_auth, scoped]
- ... and 79 more

## Coverage Gaps (Untested Tools)

64 tools lack any test coverage:

- approve_item (approval)
- compose_tweet (write)
- draft_replies_for_candidates (composite)
- find_reply_opportunities (composite)
- generate_reply (content)
- generate_thread (content)
- generate_thread_plan (composite)
- generate_tweet (content)
- get_author_context (context)
- get_capabilities (meta)
- get_discovery_feed (discovery)
- get_mode (meta)
- get_policy_status (policy)
- get_stats (analytics)
- get_x_usage (analytics)
- health_check (health)
- list_pending_approvals (approval)
- propose_and_queue_replies (composite)
- recommend_engagement_action (context)
- reject_item (approval)
- suggest_topics (content)
- x_delete (write)
- x_get (read)
- x_post (write)
- x_post_thread_dry_run (write)
- x_post_tweet_dry_run (write)
- x_put (write)
- x_upload_media (media)
- x_v2_blocks_create (moderation)
- x_v2_blocks_delete (moderation)
- x_v2_blocks_list (moderation)
- x_v2_lists_create (list)
- x_v2_lists_delete (list)
- x_v2_lists_follow (list)
- x_v2_lists_followers (list)
- x_v2_lists_get (list)
- x_v2_lists_members (list)
- x_v2_lists_members_add (list)
- x_v2_lists_members_remove (list)
- x_v2_lists_memberships (list)
- x_v2_lists_owned (list)
- x_v2_lists_pin (list)
- x_v2_lists_pinned (list)
- x_v2_lists_tweets (list)
- x_v2_lists_unfollow (list)
- x_v2_lists_update (list)
- x_v2_mutes_create (moderation)
- x_v2_mutes_delete (moderation)
- x_v2_mutes_list (moderation)
- x_v2_spaces_buyers (read)
- x_v2_spaces_by_creator (read)
- x_v2_spaces_get (read)
- x_v2_spaces_lookup (read)
- x_v2_spaces_search (read)
- x_v2_spaces_tweets (read)
- x_v2_tweets_counts_recent (read)
- x_v2_tweets_hide_reply (moderation)
- x_v2_tweets_lookup (read)
- x_v2_tweets_quote_tweets (read)
- x_v2_tweets_retweeted_by (read)
- x_v2_tweets_unhide_reply (moderation)
- x_v2_users_lookup_by_usernames (read)
- x_v2_users_pin_tweet (engage)
- x_v2_users_unpin_tweet (engage)
