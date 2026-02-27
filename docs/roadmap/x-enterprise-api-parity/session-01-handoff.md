# Session 01 Handoff — X Enterprise API Parity

**Date:** 2026-02-26
**Branch:** `feat/mcp_x_api_coverage`
**Status:** Complete

---

## Summary of Changes

Session 01 produced three planning artifacts that establish the full implementation charter for closing DM, Ads/Campaign, and Enterprise Admin/Compliance API coverage gaps in the MCP server:

1. **`coverage-gap-audit.md`** — Detailed inventory of all 108 existing tools across 6 profiles, identification of 31 missing enterprise endpoints across 4 families (DM, Ads, Compliance, Stream Rules), scope coverage matrix, and file-path evidence for every gap.

2. **`charter.md`** — Decision-final implementation charter with 8 locked architecture decisions, endpoint family definitions for all 31 new tools (including parameters, scopes, mutation classification, and profile membership), acceptance metrics with pre/post tool counts, and a session-by-session execution map for Sessions 02-06.

3. **`session-01-handoff.md`** — This document.

No Rust code was changed in this session.

---

## Decisions Made

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Keep existing 6-profile model | DM fits existing Read/Write tiers; Ads/Compliance fit Admin |
| 2 | All 31 new endpoints as Layer 2 spec entries | Generator handles lane/schema/profile automatically |
| 3 | Three new `ToolCategory` variants: `DirectMessage`, `Ads`, `Compliance` | Semantic grouping for manifests and agent context |
| 4 | Add `ads-api.x.com` to host allowlist | Required for typed Ads tools and admin universal requests |
| 5 | All mutations through policy + mutation audit | Existing pipeline handles this automatically for Workflow-lane tools |
| 6 | 13 new entries in mutation denylist | Safety boundary tests enforce no mutations leak to read-only |
| 7 | Ads API pinned to v12 paths | Independent versioning from v2 public API |
| 8 | No streaming support (only rule management) | SSE connections incompatible with MCP request/response model |

---

## Open Risks

| Risk | Severity | Status |
|------|----------|--------|
| `EndpointDef` struct needs optional `host` field for Ads API (currently all default to api.x.com) | Medium | Deferred to Session 03 — requires changes to `params.rs` + `generator.rs` |
| Boundary test count updates must be incremental across sessions | Medium | Each session documents exact expected counts |
| Manifest snapshot regeneration may conflict if other branches modify tools | Low | Session 06 does final verification |

---

## Next-Session Inputs (Session 02)

### Mission

Implement the 8 Direct Message typed tools as Layer 2 spec entries.

### Files to Read First

| File | Why |
|------|-----|
| `crates/tuitbot-mcp/src/tools/manifest.rs:58-79` | Current `ToolCategory` enum — add `DirectMessage` |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Existing spec pack — add 8 DM EndpointDefs following same patterns |
| `crates/tuitbot-mcp/src/spec/endpoints.rs:12-32` | Profile shorthands — reuse `WRITE_UP_AND_API_RO_AND_UTIL_WRITE` for DM reads, `WRITE_UP_AND_UTIL_WRITE` for DM writes |
| `crates/tuitbot-mcp/src/spec/endpoints.rs:36-58` | Error code sets — reuse `X_READ_ERR` for reads, `X_WRITE_ERR` for writes |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs:16-58` | Mutation denylist — add 3 DM mutation names |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs:242-284` | Profile count assertions — update: ApiRO 40→45, Write 104→112, Admin 108→116 |
| `roadmap/artifacts/session-06-tool-manifest.json` | Manifest snapshot — must be regenerated |
| `docs/roadmap/x-enterprise-api-parity/charter.md` | Charter Section 3.1 — DM tool definitions with parameters |

### Commands to Run After Changes

```bash
# Full CI checklist
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings

# Regenerate manifest snapshot (if test fails)
cargo test -p tuitbot-mcp manifest_snapshot -- --ignored
```

### Exact Changes to Make

1. **`crates/tuitbot-mcp/src/tools/manifest.rs:58-79`**
   - Add `DirectMessage` variant to `ToolCategory` enum (after `Moderation`)

2. **`crates/tuitbot-mcp/src/spec/endpoints.rs`**
   - Add `PARAM_DM_EVENT_FIELDS` common param constant
   - Add `PARAM_PARTICIPANT_ID` common param constant
   - Add 8 `EndpointDef` entries for DM tools (5 reads, 3 mutations) at the end of `SPEC_ENDPOINTS`
   - Group comment: `// -- Direct Messages (8) --`

3. **`crates/tuitbot-mcp/src/tools/boundary_tests.rs`**
   - Add to `mutation_denylist()`: `"x_v2_dm_send_in_conversation"`, `"x_v2_dm_send_to_participant"`, `"x_v2_dm_create_group"`
   - Update `api_readonly_profile_tool_count`: 40 → 45
   - Update `write_profile_tool_count`: 104 → 112
   - Update `admin_profile_tool_count`: 108 → 116

4. **`roadmap/artifacts/session-06-tool-manifest.json`**
   - Regenerate via `cargo test -p tuitbot-mcp manifest_snapshot -- --ignored`

### Expected Post-Session 02 State

| Profile | Tool Count |
|---------|-----------|
| Readonly | 14 (unchanged) |
| ApiReadonly | 45 (+5 DM reads) |
| Write | 112 (+5 DM reads, +3 DM writes) |
| Admin | 116 (+5 DM reads, +3 DM writes) |
| UtilityReadonly | ~12 (unchanged) |
| UtilityWrite | ~58 (+5 DM reads, +3 DM writes) |
