# Session 05 — Enterprise Admin/Compliance Endpoint Matrix

**Date:** 2026-02-26
**Spec Version:** 1.3.0
**Tools Added:** 7 (4 reads + 3 mutations)

---

## Compliance Endpoints (4)

| Tool Name | Method | Path | Mutation | Profile | Scopes | Host |
|-----------|--------|------|----------|---------|--------|------|
| `x_v2_compliance_jobs` | GET | `/2/compliance/jobs` | No | Admin | `compliance.write` | api.x.com |
| `x_v2_compliance_job_by_id` | GET | `/2/compliance/jobs/{id}` | No | Admin | `compliance.write` | api.x.com |
| `x_v2_compliance_job_create` | POST | `/2/compliance/jobs` | Yes | Admin | `compliance.write` | api.x.com |
| `x_v2_usage_tweets` | GET | `/2/usage/tweets` | No | Admin | `usage.read` | api.x.com |

### Parameters

| Tool | Required | Optional |
|------|----------|----------|
| `x_v2_compliance_jobs` | `type` (tweets/users) | `status` (created, in_progress, complete, expired, failed) |
| `x_v2_compliance_job_by_id` | `id` | — |
| `x_v2_compliance_job_create` | `type` (tweets/users) | `name`, `resumable` |
| `x_v2_usage_tweets` | — | `days` (default: 7) |

---

## Stream Rule Endpoints (3)

| Tool Name | Method | Path | Mutation | Profile | Scopes | Host |
|-----------|--------|------|----------|---------|--------|------|
| `x_v2_stream_rules_list` | GET | `/2/tweets/search/stream/rules` | No | Admin | `tweet.read` | api.x.com |
| `x_v2_stream_rules_add` | POST | `/2/tweets/search/stream/rules` | Yes | Admin | `tweet.read` | api.x.com |
| `x_v2_stream_rules_delete` | POST | `/2/tweets/search/stream/rules` | Yes | Admin | `tweet.read` | api.x.com |

### Parameters

| Tool | Required | Optional |
|------|----------|----------|
| `x_v2_stream_rules_list` | — | — |
| `x_v2_stream_rules_add` | `value` (filter expression) | `tag` (label) |
| `x_v2_stream_rules_delete` | `rule_ids` (comma-separated) | — |

### Stream Rule Notes

- `x_v2_stream_rules_delete` uses HTTP POST (not DELETE) per X API design — the rule IDs are sent in the request body with a `delete` payload.
- Stream rules manage the filtered stream (`/2/tweets/search/stream`) but this session does not implement the long-lived SSE connection endpoint, which is incompatible with the MCP request/response model.
- Max 25 rules per add request.

---

## Safety Controls

| Control | Coverage |
|---------|----------|
| **Profile isolation** | All 7 tools Admin-only; absent from Write, ApiReadonly, Readonly |
| **Elevated access** | All 3 mutations require elevated access (derived from admin-only profile) |
| **DB audit** | All 3 mutations require DB for mutation audit logging |
| **Policy gate** | Mutations route through Workflow lane, triggering policy evaluation |
| **Error handling** | Reads: standard X read errors. Mutations: full policy + mutation error set |
| **Policy category** | `EnterpriseAdmin` variant in policy engine for compliance mutations |
| **OAuth scopes** | `compliance.write` for compliance jobs, `usage.read` for usage, `tweet.read` for stream rules |

---

## Rollback Guidance

| Mutation | Rollback Strategy |
|----------|-------------------|
| `x_v2_compliance_job_create` | Jobs cannot be cancelled once created. Compliance jobs are idempotent — creating a duplicate job for the same data set is safe. Monitor job status via `x_v2_compliance_jobs` or `x_v2_compliance_job_by_id`. |
| `x_v2_stream_rules_add` | Use `x_v2_stream_rules_delete` to remove unwanted rules. List current rules with `x_v2_stream_rules_list` to identify IDs. |
| `x_v2_stream_rules_delete` | Deleted rules must be re-created via `x_v2_stream_rules_add`. Rule IDs change on re-creation. |

---

## Profile Impact

| Profile | Before | After | Delta |
|---------|--------|-------|-------|
| Readonly | 14 | 14 | +0 |
| ApiReadonly | 45 | 45 | +0 |
| Write | 112 | 112 | +0 |
| Admin | 132 | 139 | +7 |
