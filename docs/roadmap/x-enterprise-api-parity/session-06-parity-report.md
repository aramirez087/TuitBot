# Session 06 — Manifest/Runtime Parity Report

**Date:** 2026-02-26
**Branch:** `feat/mcp_x_api_coverage`
**Status:** All parity checks pass

---

## Parity Verification Results

### Structural Parity (per-profile)

Every tool declared in the manifest has a backing implementation: either a curated `#[tool]` handler in the server module or a generated spec endpoint dispatched through universal request tools.

| Profile | Manifest Tools | Curated Handlers | Spec Endpoints | Orphan Handlers | Missing Backing |
|---------|---------------|------------------|----------------|-----------------|-----------------|
| Readonly | 14 | 10 | 4 | 0 | 0 |
| ApiReadonly | 45 | 20 | 25 | 0 | 0 |
| Write | 112 | 68 | 44 | 0 | 0 |
| Admin | 139 | 72 | 67 | 0 | 0 |

### Cross-Profile Consistency

| Check | Status |
|-------|--------|
| Every spec endpoint in manifest for declared profiles | PASS |
| Every manifest tool has backing implementation | PASS |
| No tool is both curated and spec-generated | PASS |
| Readonly is subset of ApiReadonly | PASS |
| Write is subset of Admin | PASS |

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Total tools | 140 |
| Curated (L1) | 73 |
| Generated (L2) | 67 |
| Mutation tools | 51 |
| Read-only tools | 89 |
| Tools with any test | 76 (54.3%) |

### Test Coverage by Type

| Test Type | Count |
|-----------|-------|
| Kernel conformance (L1 curated) | 27 |
| Spec conformance (L2 generated) | 31 |
| Contract envelope | 18 |
| Live (sandbox) | 9 |
| Untested | 64 |

---

## Profile Deltas (Pre vs Post Initiative)

| Profile | Pre | Post | Delta | New Reads | New Mutations |
|---------|-----|------|-------|-----------|---------------|
| Readonly | 14 | 14 | +0 | 0 | 0 |
| ApiReadonly | 40 | 45 | +5 | +5 DM reads | 0 |
| Write | 104 | 112 | +8 | +5 DM reads | +3 DM mutations |
| Admin | 108 | 139 | +31 | +18 reads | +13 mutations |

### Admin Delta Breakdown

| Family | Reads | Mutations | Total |
|--------|-------|-----------|-------|
| Direct Messages | 5 | 3 | 8 |
| Ads/Campaign | 9 | 7 | 16 |
| Compliance | 3 | 1 | 4 |
| Stream Rules | 1 | 2 | 3 |
| **Total** | **18** | **13** | **31** |

---

## New Tool Families — Conformance Test Coverage

### Direct Messages (8 tools) — dm.rs

| Test | Status |
|------|--------|
| All 8 tools exist in manifest | PASS |
| All 8 tools exist in SPEC_ENDPOINTS | PASS |
| DirectMessage category assignment | PASS |
| Read/mutation classification | PASS |
| Profile assignments (reads: ApiRO+Write+Admin+UtilWrite) | PASS |
| Profile assignments (mutations: Write+Admin+UtilWrite) | PASS |
| Shared lane for reads | PASS |
| Shared lane for mutations (utility bypass) | PASS |
| dm.read scope on reads | PASS |
| dm.write scope on mutations | PASS |
| Default host (api.x.com) | PASS |
| v2 API version | PASS |
| Valid schemas generated | PASS |
| No elevated access required | PASS |
| Family count = 8 | PASS |

### Ads/Campaign (16 tools) — ads.rs

| Test | Status |
|------|--------|
| All 16 tools exist in manifest | PASS |
| All 16 tools exist in SPEC_ENDPOINTS | PASS |
| Ads category assignment | PASS |
| Read/mutation classification (9 reads, 7 mutations) | PASS |
| All Admin-only | PASS |
| Not in Write profile | PASS |
| Not in readonly profiles | PASS |
| Reads use Shared lane | PASS |
| Mutations use Workflow lane | PASS |
| Mutations require DB | PASS |
| All require elevated access | PASS |
| Target ads-api.x.com host | PASS |
| ads-v12 API version | PASS |
| ads.read scope on reads | PASS |
| ads.write scope on mutations | PASS |
| Valid schemas generated | PASS |
| Family count = 16 | PASS |

### Enterprise Admin (7 tools) — enterprise_admin.rs

| Test | Status |
|------|--------|
| All 7 tools exist in manifest | PASS |
| All 7 tools exist in SPEC_ENDPOINTS | PASS |
| Compliance category assignment | PASS |
| Read/mutation classification (4 reads, 3 mutations) | PASS |
| All Admin-only | PASS |
| Not in Write profile | PASS |
| Not in readonly profiles | PASS |
| Reads use Shared lane | PASS |
| Mutations use Workflow lane | PASS |
| Mutations require DB | PASS |
| All require elevated access | PASS |
| Default host (api.x.com) | PASS |
| v2 API version | PASS |
| compliance.write scope on compliance jobs | PASS |
| usage.read scope on usage tweets | PASS |
| tweet.read scope on stream rules | PASS |
| Stream rules delete uses POST | PASS |
| Valid schemas generated | PASS |
| Family count = 7 | PASS |

---

## Safety Invariants Verified

1. **No mutations in readonly profiles** — 0 mutations in Readonly (14 tools), 0 in ApiReadonly (45 tools)
2. **Mutation denylist synchronized** — All 51 mutation tools in denylist, denylist matches manifest exactly
3. **Profile hierarchy preserved** — Readonly < ApiReadonly < Write < Admin (strict superset chain)
4. **Enterprise tools isolated** — All 23 Ads+Compliance tools Admin-only, none leak to Write/ApiReadonly/Readonly
5. **Elevated access on admin-only tools** — All 27 admin-only tools require elevated access
6. **Policy gating on mutations** — All non-utility mutations route through Workflow lane with DB audit
7. **Host allowlist** — Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com accepted
