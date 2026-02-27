# Session 02 — Enterprise Security Foundation

**Date:** 2026-02-26
**Branch:** `feat/mcp_x_api_coverage`
**Status:** Complete

---

## Mission

Harden the universal request layer (`x_get`, `x_post`, `x_put`, `x_delete`) so that
enterprise API domains (DM, Ads, Compliance) can be added in future sessions without
weakening security or auditability.

---

## Changes Delivered

### 1. Host Allowlist Extension

**File:** `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs`

- Added `ads-api.x.com` to `ALLOWED_HOSTS` (required for Ads/Campaign API endpoints).
- Existing SSRF protections (IP-literal blocking, path traversal rejection, header blocklist) apply uniformly to the new host.

### 2. Policy-Gated Mutations via Unified Gateway

**Files:**
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/audited.rs` (new)
- `crates/tuitbot-mcp/src/server/admin.rs` (modified)

Created `x_post_audited`, `x_put_audited`, `x_delete_audited` — mutation variants that
run through the unified mutation gateway (`run_gateway`) before executing the HTTP request.

This means every universal mutation request now:
1. Gets policy evaluation (allow/deny/require-approval/dry-run)
2. Gets idempotency deduplication
3. Gets a full audit trail recorded to `mutation_audit` table
4. Includes `correlation_id` in error responses for traceability

The admin MCP server handlers in `admin.rs` were updated to call these audited variants
instead of the raw HTTP functions.

### 3. Request Family Classification

**File:** `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/family.rs` (new)

Introduced `RequestFamily` enum with five variants:
- `PublicApi` — Standard v2 endpoints (tweets, users, lists)
- `DirectMessage` — DM endpoints (`/2/dm_conversations`, `/2/dm_events`)
- `Ads` — Ads/Campaign API on `ads-api.x.com`
- `EnterpriseAdmin` — Compliance/Usage endpoints (`/2/compliance`, `/2/usage`)
- `MediaUpload` — Upload endpoints on `upload.x.com` / `upload.twitter.com`

Classification is derived from host + path and included in audit `params_json` for every
mutation, enabling per-family reporting and future per-family policy rules.

### 4. Manifest Metadata Updates

**File:** `crates/tuitbot-mcp/src/tools/manifest.rs`

- Split `X_REQUEST_ERR` into `X_REQUEST_READ_ERR` (for `x_get`) and `X_REQUEST_MUTATION_ERR`
  (for `x_post`, `x_put`, `x_delete`).
- `X_REQUEST_MUTATION_ERR` adds five policy-related error codes:
  `PolicyDeniedBlocked`, `PolicyDeniedRateLimited`, `PolicyDeniedHardRule`,
  `PolicyDeniedUserRule`, `PolicyError`.
- Mutation tools now declare `requires_db: true` (required for audit recording).
- Regenerated `roadmap/artifacts/session-06-tool-manifest.json`.

### 5. Core Policy Types

**File:** `crates/tuitbot-core/src/mcp_policy/types.rs`

- Added `UniversalRequest` variant to `ToolCategory` enum.
- Updated `tool_category()` to map `x_post`, `x_put`, `x_delete` to `UniversalRequest`.
- Added tests for the new category mapping, display, and serde roundtrip.

### 6. Module Splitting (File Size Compliance)

Extracted from `mod.rs` (was 782 lines → now 474 lines) into focused submodules:
- `audited.rs` — Policy-gated mutation execution (198 lines)
- `family.rs` — Request family enum and classifier (62 lines)

---

## Test Coverage

### New Tests in `x_request/tests.rs`

| Test | What It Verifies |
|------|------------------|
| `ads_api_host_accepted` | `ads-api.x.com` passes allowlist |
| `ads_api_host_case_insensitive` | Case-insensitive host matching |
| `classify_public_api_default_host` | Default host → PublicApi |
| `classify_public_api_explicit_host` | Explicit api.x.com → PublicApi |
| `classify_dm_conversations` | /2/dm_conversations → DirectMessage |
| `classify_dm_events` | /2/dm_events → DirectMessage |
| `classify_dm_conversations_subpath` | Nested DM path → DirectMessage |
| `classify_dm_case_insensitive` | Case-insensitive path matching |
| `classify_ads_api` | ads-api.x.com → Ads |
| `classify_ads_api_case_insensitive` | Case-insensitive host → Ads |
| `classify_ads_api_any_path` | Any path on ads-api.x.com → Ads |
| `classify_enterprise_compliance` | /2/compliance → EnterpriseAdmin |
| `classify_enterprise_usage` | /2/usage → EnterpriseAdmin |
| `classify_enterprise_subpath` | Nested compliance path → EnterpriseAdmin |
| `classify_media_upload` | upload.x.com → MediaUpload |
| `classify_media_upload_twitter` | upload.twitter.com → MediaUpload |
| `classify_unrecognized_path` | Unknown path → PublicApi fallback |
| `request_family_display` | Display impl for all variants |
| `request_family_serializes_to_snake_case` | Serde serialization |
| `ads_host_still_validates_path` | Path validation on ads host |
| `ads_host_query_in_path_blocked` | Query-in-path rejected on ads host |
| `ads_host_control_chars_blocked` | Control chars rejected on ads host |

### New Tests in `mcp_policy/types.rs`

| Test | What It Verifies |
|------|------------------|
| `tool_category_universal_request_tools` | x_post/x_put/x_delete → UniversalRequest |
| `tool_category_standard_tools` | Existing tool mappings unchanged |
| `tool_category_unknown_defaults_to_write` | Unknown tools default to Write |
| `tool_category_display` | Display impl for UniversalRequest |
| `tool_category_serde_roundtrip` | JSON serialize/deserialize roundtrip |

---

## CI Status

```
cargo fmt --all && cargo fmt --all --check    ✅
RUSTFLAGS="-D warnings" cargo test --workspace ✅ (456 passed, 11 ignored)
cargo clippy --workspace -- -D warnings        ✅
```
