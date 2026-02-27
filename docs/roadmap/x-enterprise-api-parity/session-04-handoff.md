# Session 04 Handoff — X Enterprise API Parity

**Date:** 2026-02-26
**Branch:** `feat/mcp_x_api_coverage`
**Status:** Complete

---

## Summary of Changes

Session 04 added typed Ads/Campaign API tool coverage (16 tools: 9 reads + 7 mutations), restricted to Admin profile with dedicated host routing:

1. **`host` field on EndpointDef** — New `Option<&'static str>` field enables per-endpoint host overrides. `None` defaults to `api.x.com`; Ads tools use `Some("ads-api.x.com")`.
2. **`Ads` category** — New `ToolCategory::Ads` variant for manifest grouping.
3. **16 Ads EndpointDef entries** — Full campaign lifecycle: accounts, campaigns, line items, promoted tweets, targeting criteria, analytics, and funding instruments.
4. **Elevated access for Admin-only mutations** — Generator derives `requires_elevated_access` from admin-only profile membership. All 7 Ads mutations are elevated.
5. **Boundary test updates** — 7 Ads mutations in denylist; Admin count 116→132; fixed lane test to check mutations only.
6. **Spec version bump** — `X_API_SPEC_VERSION` 1.1.0 → 1.2.0.
7. **Manifest regeneration** — Snapshot artifact + 4 profile manifests regenerated with Ads tool entries.
8. **Spec test hardening** — Naming convention relaxed for `x_ads_*`, API version test expanded for `ads-v12`, 3 new Ads-specific invariant tests.

---

## Decisions Made

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | All 16 Ads tools Admin-only | Ads API is enterprise functionality; no exposure to Write/ApiRO/Readonly profiles |
| 2 | Host field on EndpointDef (not hardcoded in request layer) | Declarative — each endpoint carries its target host, enabling future multi-host APIs |
| 3 | `requires_elevated_access` derived from admin-only profile membership | Avoids manual flag per endpoint; any tool exclusively in Admin profile is automatically elevated |
| 4 | `x_ads_*` naming prefix (not `x_v2_*`) | Ads API uses independent versioning (`ads-v12`), distinct naming prevents confusion |
| 5 | Synthetic `ads.read`/`ads.write` scopes | X Ads API uses account-level permissions, but spec requires at least one scope per endpoint for validation |
| 6 | Ads reads use `Lane::Shared`, mutations use `Lane::Workflow` | Consistent with generator logic: admin-only mutations get Workflow + DB; reads get Shared |
| 7 | Fixed `write_admin_only_tools_have_workflow_lane` boundary test | Added `t.mutation &&` guard — Ads reads are admin-only but correctly use Shared lane |

---

## Open Risks

| Risk | Severity | Status |
|------|----------|--------|
| Ads API OAuth scopes are synthetic | Low | X Ads API uses account-level access grants, not standard OAuth scopes. Current synthetic scopes work for internal validation but may need mapping if OAuth flow integrates Ads |
| Pre-existing env var race in tuitbot-core tests | Low | `config::tests::env_var_override_approval_mode` flakes due to parallel test env mutation; unrelated to session changes |
| Manifest snapshot must be regenerated after each tool change | Low | Process documented and followed |

---

## Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-mcp/src/spec/params.rs` | Added `host: Option<&'static str>` to `EndpointDef` |
| `crates/tuitbot-mcp/src/spec/generator.rs` | Added `host` to `ToolSchema`; derived `requires_elevated_access` from admin-only membership |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Added `host: None` to 44 existing entries; added 5 Ads params, `ADMIN_ONLY` const, 16 Ads `EndpointDef` entries |
| `crates/tuitbot-mcp/src/spec/tests.rs` | Updated count 44→60; relaxed naming/version tests; added 3 Ads invariant tests |
| `crates/tuitbot-mcp/src/spec/mod.rs` | Bumped `X_API_SPEC_VERSION` to 1.2.0 |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Added `Ads` variant to `ToolCategory` |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Added 7 Ads mutations to denylist; updated Admin count; fixed lane test name and condition |
| `crates/tuitbot-mcp/src/tools/conformance_tests/coverage.rs` | Added `Ads` match arm to `category_str` |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerated snapshot |
| `docs/generated/mcp-manifest-admin.json` | Regenerated (132 tools) |
| `docs/generated/mcp-manifest-write.json` | Regenerated (112 tools, unchanged count) |
| `docs/generated/mcp-manifest-api-readonly.json` | Regenerated (45 tools, unchanged count) |
| `docs/generated/mcp-manifest-readonly.json` | Regenerated (14 tools, unchanged count) |
| `docs/generated/mcp-manifest-utility-readonly.json` | Regenerated |
| `docs/generated/mcp-manifest-utility-write.json` | Regenerated |

## Files Created

| File | Purpose |
|------|---------|
| `docs/roadmap/x-enterprise-api-parity/session-04-ads-endpoint-matrix.md` | Ads endpoint reference with parameters, profiles, safety controls |
| `docs/roadmap/x-enterprise-api-parity/session-04-handoff.md` | This handoff document |

---

## Post-Session 04 State

| Profile | Tool Count | Delta |
|---------|-----------|-------|
| Readonly | 14 | +0 |
| ApiReadonly | 45 | +0 |
| Write | 112 | +0 |
| Admin | 132 | +16 (Ads tools) |

| Metric | Value |
|--------|-------|
| Total spec endpoints | 60 (was 44) |
| ToolCategory variants | 20 (was 19) |
| Spec version | 1.2.0 (was 1.1.0) |
| API versions supported | v2, ads-v12 |
| Ads reads | 9 |
| Ads mutations | 7 (all elevated, all require DB) |

---

## Next-Session Inputs (Session 05)

### Mission

Per charter, Session 05 covers **Media & Polls API** tools. This session should add Upload media, poll creation, and poll voting endpoints to the spec pack.

### Files to Read First

| File | Why |
|------|-----|
| `docs/roadmap/x-enterprise-api-parity/charter.md` | Charter Section 3.3 — Media & Polls tool definitions |
| `crates/tuitbot-mcp/src/spec/params.rs` | `EndpointDef` struct — may need new `ParamType` variants for file uploads |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Add new EndpointDef entries |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | May need new `ToolCategory` variant |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Add mutations to denylist; update profile counts |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs` | `upload.x.com` already in ALLOWED_HOSTS |

### Considerations

- Media upload uses `upload.x.com` host (already allowlisted)
- Upload API may require multipart form data — `ParamType` may need extension
- Poll creation is part of tweet compose (body params), not a separate endpoint
- Consider whether media tools belong in existing `Write` category or new `Media` category
