# X Enterprise API Parity — Implementation Charter

**Date:** 2026-02-26
**Status:** Decision-Final (no unresolved alternatives)
**Owner:** Session-driven epic execution
**Branch:** `feat/mcp_x_api_coverage`

---

## 1. Mission

Close the DM, Ads/Campaign, and Enterprise Admin/Compliance API coverage gaps in the Tuitbot MCP server. Deliver 31 new typed tools across 4 endpoint families, extend universal request safety to `ads-api.x.com`, and maintain all existing profile isolation guarantees.

---

## 2. Architecture Decisions (Locked)

### Decision 1: Profile Model — No Changes

**Decision:** Keep the existing six-profile model unchanged.

| Profile | Role | Unchanged |
|---------|------|-----------|
| Readonly | Minimal safe surface | Yes |
| ApiReadonly | X API reads + meta | Gains DM reads |
| Write | Standard operating profile | Gains DM reads + writes |
| Admin | Full access superset | Gains DM + Ads + Compliance + Stream |
| UtilityReadonly | Flat toolkit reads | Unchanged |
| UtilityWrite | Flat toolkit reads + writes | Gains DM reads + writes |

**Rationale:** The existing profile hierarchy (Readonly < ApiReadonly < Write < Admin, with Utility profiles as flat toolkit variants) already captures the right permission boundaries. DM tools fit into existing Read/Write tiers. Ads and Compliance are enterprise-only features that belong exclusively in Admin.

### Decision 2: New Endpoint Families as Typed Layer 2 Spec Entries

**Decision:** All 31 new endpoints are added as `EndpointDef` entries in `crates/tuitbot-mcp/src/spec/endpoints.rs`, following the exact same pattern as Lists, Mutes, Blocks, and Spaces.

**Rationale:** The spec-pack generator (`crates/tuitbot-mcp/src/spec/generator.rs`) already handles:
- Lane assignment (Shared for reads + utility mutations, Workflow for non-utility mutations)
- Tool schema generation (JSON Schema from `ParamDef` arrays)
- Profile filtering
- Manifest merging with curated L1 tools

No generator changes needed — only new `EndpointDef` data.

### Decision 3: Three New ToolCategory Variants

**Decision:** Add three new `ToolCategory` variants to `crates/tuitbot-mcp/src/tools/manifest.rs`:

```rust
pub enum ToolCategory {
    // ... existing 18 variants ...
    DirectMessage,  // DM reads and writes
    Ads,            // Ads API campaign management
    Compliance,     // GDPR compliance and stream rules
}
```

**Rationale:** Categories are used for manifest grouping, boundary tests, and agent context. Mixing DMs into `Read`/`Write` would lose semantic meaning. Ads and Compliance are distinct enterprise domains with their own access patterns.

### Decision 4: Extend Host Allowlist for `ads-api.x.com`

**Decision:** Add `ads-api.x.com` to `ALLOWED_HOSTS` in `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs`.

```rust
const ALLOWED_HOSTS: &[&str] = &[
    "api.x.com",
    "upload.x.com",
    "upload.twitter.com",
    "ads-api.x.com",  // Enterprise Ads API
];
```

**Safety controls retained:**
- Path validation (no traversal, no query in path, no control chars)
- SSRF protection (no IP literals, HTTPS only)
- Header blocklist (authorization, host, cookie, etc.)
- Admin-only access gate (universal request tools remain Admin profile only)

**Additional control:** The typed Ads tools themselves are Admin-only in the spec definitions, so even without universal request tools, Ads endpoints are gated to Admin profile.

### Decision 5: All Mutations Through Policy + Mutation Audit

**Decision:** Every new mutation tool routes through the existing policy gate and mutation audit pipeline. No exceptions for DM sends, Ad creation, or compliance job creation.

**Implementation:** This happens automatically for Workflow-lane mutation tools — the generator already sets `lane: Workflow, requires_db: true` for mutations in non-utility profiles. The existing `check_policy` + `record_mutation` pattern in the server handlers applies.

For utility-profile mutations (DM sends in UtilityWrite), these bypass policy gate (matching existing behavior for `x_post_tweet` etc. in utility profiles).

### Decision 6: Mutation Denylist Expansion

**Decision:** Add all 13 new mutation tool names to the `mutation_denylist()` in `crates/tuitbot-mcp/src/tools/boundary_tests.rs`. Update all tool count assertions.

**New mutation tools:**
- DM (3): `x_v2_dm_send_in_conversation`, `x_v2_dm_send_to_participant`, `x_v2_dm_create_group`
- Ads (7): `x_ads_campaign_create`, `x_ads_campaign_update`, `x_ads_campaign_delete`, `x_ads_line_item_create`, `x_ads_promoted_tweet_create`, `x_ads_targeting_create`, `x_ads_targeting_delete`
- Compliance (1): `x_v2_compliance_job_create`
- Stream Rules (2): `x_v2_stream_rules_add`, `x_v2_stream_rules_delete`

### Decision 7: Ads API Version Strategy

**Decision:** Use Ads API v12 paths. The spec pack `api_version` field will be `"ads-v12"` for Ads endpoints (distinct from `"v2"` for standard endpoints).

**Rationale:** X Ads API versioning is independent from the v2 public API. Pinning to v12 ensures deterministic behavior. When X releases v13+, a version bump in the spec pack is a one-line change per endpoint.

### Decision 8: No Streaming Support

**Decision:** Do NOT implement the filtered stream connection endpoint (`GET /2/tweets/search/stream`). Only implement stream rule management (add/delete/list rules).

**Rationale:** The filtered stream is a long-lived SSE connection that does not fit the MCP request/response model. Stream rule management (CRUD operations) IS standard request/response and fits perfectly.

---

## 3. Endpoint Family Definitions

### 3.1 Direct Messages (8 tools)

| Tool Name | Method | Path | Category | Mutation | Scopes | Profiles |
|-----------|--------|------|----------|----------|--------|----------|
| `x_v2_dm_conversations` | GET | `/2/dm_conversations` | DirectMessage | No | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_conversation_by_id` | GET | `/2/dm_conversations/{id}` | DirectMessage | No | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events_by_conversation` | GET | `/2/dm_conversations/{id}/dm_events` | DirectMessage | No | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events_by_participant` | GET | `/2/dm_conversations/with/{participant_id}/dm_events` | DirectMessage | No | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events` | GET | `/2/dm_events` | DirectMessage | No | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_send_in_conversation` | POST | `/2/dm_conversations/{id}/messages` | DirectMessage | Yes | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |
| `x_v2_dm_send_to_participant` | POST | `/2/dm_conversations/with/{participant_id}/messages` | DirectMessage | Yes | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |
| `x_v2_dm_create_group` | POST | `/2/dm_conversations` | DirectMessage | Yes | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |

**Parameters per tool:**

| Tool | Required Params | Optional Params |
|------|----------------|-----------------|
| `x_v2_dm_conversations` | — | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` |
| `x_v2_dm_conversation_by_id` | `id` | `dm_event_fields`, `expansions` |
| `x_v2_dm_events_by_conversation` | `id` | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` |
| `x_v2_dm_events_by_participant` | `participant_id` | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` |
| `x_v2_dm_events` | — | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` |
| `x_v2_dm_send_in_conversation` | `id`, `text` | `attachments` (media IDs array) |
| `x_v2_dm_send_to_participant` | `participant_id`, `text` | `attachments` |
| `x_v2_dm_create_group` | `participant_ids` (array), `text` | `attachments` |

### 3.2 Ads / Campaign (16 tools)

| Tool Name | Method | Path | Category | Mutation | Profiles |
|-----------|--------|------|----------|----------|----------|
| `x_ads_accounts` | GET | `/12/accounts` | Ads | No | Admin |
| `x_ads_account_by_id` | GET | `/12/accounts/{account_id}` | Ads | No | Admin |
| `x_ads_campaigns` | GET | `/12/accounts/{account_id}/campaigns` | Ads | No | Admin |
| `x_ads_campaign_by_id` | GET | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Ads | No | Admin |
| `x_ads_campaign_create` | POST | `/12/accounts/{account_id}/campaigns` | Ads | Yes | Admin |
| `x_ads_campaign_update` | PUT | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Ads | Yes | Admin |
| `x_ads_campaign_delete` | DELETE | `/12/accounts/{account_id}/campaigns/{campaign_id}` | Ads | Yes | Admin |
| `x_ads_line_items` | GET | `/12/accounts/{account_id}/line_items` | Ads | No | Admin |
| `x_ads_line_item_create` | POST | `/12/accounts/{account_id}/line_items` | Ads | Yes | Admin |
| `x_ads_promoted_tweets` | GET | `/12/accounts/{account_id}/promoted_tweets` | Ads | No | Admin |
| `x_ads_promoted_tweet_create` | POST | `/12/accounts/{account_id}/promoted_tweets` | Ads | Yes | Admin |
| `x_ads_targeting_criteria` | GET | `/12/accounts/{account_id}/targeting_criteria` | Ads | No | Admin |
| `x_ads_targeting_create` | POST | `/12/accounts/{account_id}/targeting_criteria` | Ads | Yes | Admin |
| `x_ads_targeting_delete` | DELETE | `/12/accounts/{account_id}/targeting_criteria/{id}` | Ads | Yes | Admin |
| `x_ads_analytics` | GET | `/12/stats/accounts/{account_id}` | Ads | No | Admin |
| `x_ads_funding_instruments` | GET | `/12/accounts/{account_id}/funding_instruments` | Ads | No | Admin |

**Ads tools are Admin-only** because:
1. Ads API access requires a separate developer account approval
2. Ads mutations can incur financial costs (ad spend)
3. Admin profile already implies elevated trust and access

**Host:** All Ads tools use `ads-api.x.com` as the host parameter in the EndpointDef.

### 3.3 Compliance (4 tools)

| Tool Name | Method | Path | Category | Mutation | Scopes | Profiles |
|-----------|--------|------|----------|----------|--------|----------|
| `x_v2_compliance_jobs` | GET | `/2/compliance/jobs` | Compliance | No | `compliance.write` | Admin |
| `x_v2_compliance_job_by_id` | GET | `/2/compliance/jobs/{id}` | Compliance | No | `compliance.write` | Admin |
| `x_v2_compliance_job_create` | POST | `/2/compliance/jobs` | Compliance | Yes | `compliance.write` | Admin |
| `x_v2_usage_tweets` | GET | `/2/usage/tweets` | Compliance | No | `usage.read` | Admin |

### 3.4 Stream Rules (3 tools)

| Tool Name | Method | Path | Category | Mutation | Scopes | Profiles |
|-----------|--------|------|----------|----------|--------|----------|
| `x_v2_stream_rules_list` | GET | `/2/tweets/search/stream/rules` | Compliance | No | `tweet.read` | Admin |
| `x_v2_stream_rules_add` | POST | `/2/tweets/search/stream/rules` | Compliance | Yes | `tweet.read` | Admin |
| `x_v2_stream_rules_delete` | POST | `/2/tweets/search/stream/rules` | Compliance | Yes | `tweet.read` | Admin |

**Note:** Stream rule delete uses POST with a `delete` body payload (X API design quirk). The tool will accept a `rule_ids` parameter and construct the delete body.

---

## 4. Acceptance Metrics

### 4.1 Manifest Tool Counts

| Profile | Pre-Implementation | Post-Implementation | Delta |
|---------|-------------------|---------------------|-------|
| Readonly | 14 | 14 | +0 |
| ApiReadonly | 40 | 45 | +5 (DM reads) |
| Write | 104 | 112 | +8 (DM reads + writes) |
| Admin | 108 | 139 | +31 (DM + Ads + Compliance + Stream) |
| UtilityReadonly | ~12 | ~12 | +0 |
| UtilityWrite | ~50 | ~58 | +8 (DM reads + writes) |

### 4.2 Coverage Report Deltas

| Metric | Pre | Post |
|--------|-----|------|
| Total unique tools | 108 | 139 |
| Spec-generated tools (L2) | 36 | 67 |
| Endpoint groups in spec | 7 | 11 |
| OAuth scopes covered | 17 | 21 |
| ToolCategory variants | 18 | 21 |
| Hosts in allowlist | 3 | 4 |

### 4.3 Test Coverage Requirements

| Test Area | Minimum New Tests | Files |
|-----------|------------------|-------|
| Boundary tests (profile counts) | Update 8 existing count assertions | `boundary_tests.rs` |
| Boundary tests (mutation denylist) | Add 13 entries | `boundary_tests.rs` |
| Boundary tests (category guards) | Add DM/Ads/Compliance to mutation categories | `boundary_tests.rs` |
| Host allowlist validation | 2 new tests (ads-api.x.com allowed, random host blocked) | `x_request/mod.rs::tests` |
| Manifest snapshot | Regenerate snapshot artifact | `manifest.rs::tests` |
| Spec generator | Verify 67 tools generated (was 36) | `spec/generator.rs::tests` |
| New category roundtrip | 3 serde tests for new categories | `manifest.rs::tests` |

### 4.4 Documentation Requirements

| Document | Update Required |
|----------|----------------|
| `docs/mcp-reference.md` | Update tool counts, add DM/Ads/Compliance sections |
| `docs/configuration.md` | Document Ads API access requirements |
| `README.md` | Update feature list and tool counts |
| `docs/generated/mcp-manifest-*.json` | Regenerate all profile manifests |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerate snapshot |

---

## 5. Session-by-Session Execution Map

### Session 02: DM Tools (8 new tools)

**Scope:** Add typed Direct Message tools to the spec pack.

**File targets:**
| File | Action |
|------|--------|
| `crates/tuitbot-mcp/src/tools/manifest.rs:58-79` | Add `DirectMessage` to `ToolCategory` enum |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Add 8 `EndpointDef` entries for DM endpoints |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` (top) | Add `PARAM_DM_EVENT_FIELDS` and `PARAM_PARTICIPANT_ID` common params |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Add 3 DM mutations to denylist; update profile counts: ApiRO 40→45, Write 104→112, Admin 108→116 |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerate |

**Verification:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Entry points:**
- Read `crates/tuitbot-mcp/src/spec/endpoints.rs` to see existing patterns
- Read `crates/tuitbot-mcp/src/tools/manifest.rs:58-79` for ToolCategory enum
- Read `crates/tuitbot-mcp/src/tools/boundary_tests.rs` for count assertions

---

### Session 03: Ads/Campaign Tools (16 new tools)

**Scope:** Add typed Ads API tools and extend host allowlist.

**File targets:**
| File | Action |
|------|--------|
| `crates/tuitbot-mcp/src/tools/manifest.rs:58-79` | Add `Ads` to `ToolCategory` enum |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Add 16 `EndpointDef` entries for Ads endpoints |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` (top) | Add `ADMIN_ONLY` profile shorthand, `PARAM_ACCOUNT_ID`, `PARAM_CAMPAIGN_ID` common params |
| `crates/tuitbot-mcp/src/spec/params.rs` | Add `host` field to `EndpointDef` (currently all endpoints default to `api.x.com`; Ads need `ads-api.x.com`) |
| `crates/tuitbot-mcp/src/spec/generator.rs` | Pass `host` from EndpointDef through to generated tool schemas |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs:29` | Add `ads-api.x.com` to `ALLOWED_HOSTS` |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs::tests` | Add test for ads-api.x.com host validation |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Add 7 Ads mutations to denylist; update Admin count 116→132 |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerate |

**Verification:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Key risk:** The `EndpointDef` struct currently has no `host` field — all endpoints implicitly target `api.x.com`. Session 03 must add an optional `host` field to `EndpointDef` in `params.rs` and thread it through the generator.

**Entry points:**
- Read `crates/tuitbot-mcp/src/spec/params.rs` for EndpointDef struct
- Read `crates/tuitbot-mcp/src/spec/generator.rs` for tool generation logic
- Read `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs:26-40` for host allowlist

---

### Session 04: Compliance + Stream Rule Tools (7 new tools)

**Scope:** Add Compliance and Stream Rule management tools.

**File targets:**
| File | Action |
|------|--------|
| `crates/tuitbot-mcp/src/tools/manifest.rs:58-79` | Add `Compliance` to `ToolCategory` enum |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Add 7 `EndpointDef` entries (4 compliance + 3 stream rules) |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Add 3 mutations to denylist (compliance_job_create, stream_rules_add, stream_rules_delete); update Admin count 132→139 |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerate |

**Verification:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Entry points:**
- Read `crates/tuitbot-mcp/src/spec/endpoints.rs` for existing patterns
- Read `crates/tuitbot-mcp/src/tools/boundary_tests.rs` for count assertions

---

### Session 05: Documentation + Manifest Regeneration

**Scope:** Update all documentation and regenerate machine artifacts.

**File targets:**
| File | Action |
|------|--------|
| `docs/mcp-reference.md` | Update tool counts, add DM/Ads/Compliance sections, update profile table |
| `docs/configuration.md` | Document Ads API requirements, DM scopes |
| `README.md` | Update feature list, tool counts |
| `docs/generated/mcp-manifest-write.json` | Regenerate |
| `docs/generated/mcp-manifest-admin.json` | Regenerate |
| `docs/generated/mcp-manifest-readonly.json` | Regenerate |
| `docs/generated/mcp-manifest-api-readonly.json` | Regenerate |
| `docs/generated/mcp-manifest-utility-readonly.json` | Regenerate (if exists) |
| `docs/generated/mcp-manifest-utility-write.json` | Regenerate (if exists) |

**Verification:**
```bash
bash scripts/generate-mcp-manifests.sh  # if script exists
cargo test -p tuitbot-mcp manifest -- --ignored  # regenerate snapshot
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Entry points:**
- Read `docs/mcp-reference.md` for current documentation structure
- Read `scripts/generate-mcp-manifests.sh` (if exists) for manifest generation workflow

---

### Session 06: Integration Verification + Final Audit

**Scope:** End-to-end verification, coverage report, final sign-off.

**Tasks:**
1. Run full test suite with all new tools
2. Verify manifest snapshot matches runtime
3. Verify profile isolation (no mutations leak to read-only profiles)
4. Verify host allowlist works for ads-api.x.com
5. Verify DM scopes are declared on new DM tools
6. Generate final coverage report comparing pre vs post
7. Update `docs/roadmap/x-enterprise-api-parity/` with final status

**Verification:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Exit criteria:**
- All 139 tools in Admin manifest
- All profile count assertions pass
- All mutation denylist assertions pass
- All documentation reflects actual tool counts
- Coverage gap audit shows zero remaining gaps

---

## 6. Risk Register

| Risk | Severity | Mitigation |
|------|----------|-----------|
| X Ads API may require separate developer account approval | Medium | Document in `docs/configuration.md`; Ads tools gracefully return `x_forbidden` if account lacks access |
| `EndpointDef` needs a `host` field for Ads API | Low | Session 03 adds optional `host: Option<&'static str>` with `None` defaulting to `api.x.com` |
| Stream rule delete uses POST (not DELETE) | Low | Documented in charter; `x_v2_stream_rules_delete` endpoint definition uses POST method |
| Ads API version may change (v12 → v13+) | Low | `api_version: "ads-v12"` is a one-line change per endpoint when needed |
| New ToolCategory variants require serde compatibility | Low | Existing serde derives with `rename_all = "snake_case"` handle this automatically |
| Boundary test count updates across 5 sessions | Medium | Each session updates counts incrementally; session 06 does final verification |

---

## 7. Non-Goals

The following are explicitly out of scope for this initiative:

1. **Real-time streaming connections** — The filtered stream SSE endpoint is not implementable in MCP's request/response model. Only stream rule management is in scope.
2. **Premium/Academic API tiers** — No special handling for premium search, academic research, or full-archive endpoints beyond what the universal request tools already support.
3. **New profile types** — No new profiles are created. Enterprise tools are gated to existing profiles.
4. **Workflow-layer DM/Ads orchestration** — No new composite tools for DM or Ads workflows. This charter covers typed API access only; orchestration composites are a separate initiative.
5. **Financial guardrails for Ads API** — No spend limits or budget caps in the MCP layer. These must be managed in the X Ads API dashboard. Policy gate blocks tool-level abuse; financial controls are out of scope.
