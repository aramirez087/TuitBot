# Session 04 — Ads/Campaign API Endpoint Matrix

Spec version: **1.2.0** · API version: **ads-v12** · Host: `ads-api.x.com`

## Read Endpoints (9)

| Tool Name | Method | Path | Scopes | Group | Profile |
|-----------|--------|------|--------|-------|---------|
| `x_ads_accounts` | GET | `/12/accounts` | ads.read | ads | Admin |
| `x_ads_account_by_id` | GET | `/12/accounts/:account_id` | ads.read | ads | Admin |
| `x_ads_analytics` | GET | `/12/stats/accounts/:account_id` | ads.read | ads | Admin |
| `x_ads_campaign_by_id` | GET | `/12/accounts/:account_id/campaigns/:campaign_id` | ads.read | ads | Admin |
| `x_ads_campaigns` | GET | `/12/accounts/:account_id/campaigns` | ads.read | ads | Admin |
| `x_ads_funding_instruments` | GET | `/12/accounts/:account_id/funding_instruments` | ads.read | ads | Admin |
| `x_ads_line_items` | GET | `/12/accounts/:account_id/line_items` | ads.read | ads | Admin |
| `x_ads_promoted_tweets` | GET | `/12/accounts/:account_id/promoted_tweets` | ads.read | ads | Admin |
| `x_ads_targeting_criteria` | GET | `/12/accounts/:account_id/targeting_criteria` | ads.read | ads | Admin |

## Mutation Endpoints (7)

| Tool Name | Method | Path | Scopes | Group | Profile | Elevated |
|-----------|--------|------|--------|-------|---------|----------|
| `x_ads_campaign_create` | POST | `/12/accounts/:account_id/campaigns` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_campaign_update` | PUT | `/12/accounts/:account_id/campaigns/:campaign_id` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_campaign_delete` | DELETE | `/12/accounts/:account_id/campaigns/:campaign_id` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_line_item_create` | POST | `/12/accounts/:account_id/line_items` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_promoted_tweet_create` | POST | `/12/accounts/:account_id/promoted_tweets` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_targeting_create` | POST | `/12/accounts/:account_id/targeting_criteria` | ads.read, ads.write | ads | Admin | Yes |
| `x_ads_targeting_delete` | DELETE | `/12/accounts/:account_id/targeting_criteria/:targeting_id` | ads.read, ads.write | ads | Admin | Yes |

## Safety Controls

| Control | Status |
|---------|--------|
| Host restriction (`ads-api.x.com`) | Enforced via `host` field + SSRF allowlist |
| Admin-only profile | All 16 tools in `Profile::Admin` only |
| Elevated access for mutations | 7 mutations require `requires_elevated_access` |
| DB audit trail for mutations | 7 mutations require `requires_db` |
| Mutation denylist coverage | All 7 mutations in boundary test denylist |
| Naming convention | `x_ads_*` prefix (distinct from `x_v2_*`) |
| API version tag | `ads-v12` (distinct from v2 public API) |

## Parameter Summary

| Parameter | Type | Required | Used By |
|-----------|------|----------|---------|
| `account_id` | String | Yes | All except `x_ads_accounts` |
| `campaign_id` | String | Yes | `*_by_id`, `*_update`, `*_delete` |
| `targeting_id` | String | Yes | `x_ads_targeting_delete` |
| `with_deleted` | Boolean | No | Read endpoints |
| `cursor` | String | No | List endpoints |
| `count` | Integer | No | List endpoints |
| `name` | String | Yes | `campaign_create` |
| `funding_instrument_id` | String | Yes | `campaign_create` |
| `start_time` | String | Yes | `campaign_create` |
| `daily_budget_amount_local_micro` | Integer | Yes | `campaign_create` |
| `entity_status` | String | No | `campaign_create/update` |
| `end_time` | String | No | `campaign_create/update` |
| `status` | String | No | `campaign_update` |
| `objective` | String | Yes | `line_item_create` |
| `bid_amount_local_micro` | Integer | Yes | `line_item_create` |
| `placements` | String | Yes | `line_item_create` |
| `product_type` | String | Yes | `line_item_create` |
| `line_item_id` | String | Yes | `promoted_tweet_create`, `targeting_create` |
| `tweet_ids` | StringArray | Yes | `promoted_tweet_create` |
| `targeting_type` | String | Yes | `targeting_create` |
| `targeting_value` | String | Yes | `targeting_create` |
| `start_time` (analytics) | String | Yes | `x_ads_analytics` |
| `end_time` (analytics) | String | Yes | `x_ads_analytics` |
| `granularity` | String | Yes | `x_ads_analytics` |
| `metric_groups` | StringArray | Yes | `x_ads_analytics` |
| `entity` | String | Yes | `x_ads_analytics` |
| `entity_ids` | StringArray | Yes | `x_ads_analytics` |
