# Session 10 Handoff — Admin, Ads, DM Boundaries & Positioning

## What Changed

Before this session, the documentation had:
1. Ambiguous "admin" profile description — no clear definition of what admin includes vs excludes
2. No explicit statement about Ads API or DM API support status
3. Stale tool counts and profile names in README and operations docs (referencing "Full profile" with 64 tools from pre-Session 5)
4. A speculative comment in manifest.rs about "future ads/admin endpoints"
5. No unified API coverage boundary documentation

This session resolves all five.

### Deliverables

1. **Admin Profile Scope documentation** (`docs/mcp-reference.md`) — New "Admin Profile Scope" section defining precisely what admin includes (all Write tools + 4 universal request tools, constrained to approved hosts, policy-gated) and excludes (X platform admin, Ads API, DM API, policy bypass).

2. **API Coverage Boundaries** (`docs/mcp-reference.md`) — New section documenting explicit support status for every X API surface area. Five surfaces documented as "Not Supported" (Ads, DMs, Premium/Enterprise-only, v1.1 legacy, account administration) with reasons. Supported surface summary table covering all 109 tools across 9 categories.

3. **README alignment** (`README.md`) — Updated MCP section from stale 3-profile/64-tool model to current 4-profile model (Write 104, Admin 108, API read-only 40, Read-only 14). Updated compliance table to explicitly state Ads and DMs are not implemented.

4. **Operations doc update** (`docs/operations.md`) — Updated profile names and tool counts in verification runbook (Full→Write/Admin, added admin profile to smoke tests).

5. **Configuration doc update** (`docs/configuration.md`) — Added admin profile note to MCP policy section clarifying that universal request mutations are subject to the same policy engine.

6. **Manifest comment fix** (`crates/tuitbot-mcp/src/tools/manifest.rs`) — Changed speculative "future ads/admin endpoints" comment to precise "universal request tools for ad-hoc X API v2 access".

## Final Support Matrix

### X API Surface Coverage

| Surface | Status | Typed Tools | Via Universal Request (admin) |
|---------|--------|-------------|-------------------------------|
| **Tweet reads** | Supported | 26 tools | Yes |
| **Tweet writes** | Supported | 11 tools | Yes |
| **Engagements** | Supported | 10 tools | Yes |
| **User reads** | Supported | 6 tools | Yes |
| **Lists** | Supported | 15 tools | Yes |
| **Moderation** | Supported | 8 tools | Yes |
| **Spaces** | Supported | 6 tools | Yes |
| **Media upload** | Supported | 1 tool | Yes |
| **X Ads API** | Not supported | None | No (host not in allowlist) |
| **X DM API** | Not supported | None | Theoretically reachable but against X rules |
| **X Premium/Enterprise** | Partial | Standard-tier only | Yes (if credentials authorize) |
| **X API v1.1** | Not targeted | None | Technically reachable but no handling |
| **Platform admin** | Not supported | None | No (not API operations) |

### Profile Distribution

| Profile | Tools | Mutations | Admin-only | Use Case |
|---------|-------|-----------|------------|----------|
| readonly | 14 | 0 | No | Config, scoring, health |
| api_readonly | 40 | 0 | No | X API reads + utility |
| write | 104 | 35 | No | Full growth co-pilot |
| admin | 108 | 38 | Yes (4) | Write + universal request |

### Total: 109 tools (73 curated L1 + 36 generated L2)

## Explicit Non-Goals

The following are explicitly documented as not supported and not planned:

1. **X Ads API** — Requires separate Ads account. The `ads-api.x.com` host is not in the universal request tools' allowlist. No typed Ads tools exist. Building Ads support would require a separate auth flow, separate rate limit handling, and a distinct tool set — this is a different product.

2. **X DM API** — DM automation violates X's Automation Rules. No DM endpoints are implemented as typed tools. While the universal request tools could theoretically reach DM endpoints via `api.x.com/2/dm_*`, this boundary is enforced by policy documentation rather than code — a deliberate choice since the DM endpoints live on the same host as public API endpoints.

3. **Platform administration** — Account suspension, content moderation at scale, app permission management. These are X platform operations, not developer API operations.

4. **X API v1.1** — TuitBot targets v2 exclusively. No v1.1-specific pagination, error mapping, or abstraction exists.

5. **Full-archive search / Compliance streams** — Require Premium or Enterprise X API plans. TuitBot doesn't provide typed tools for these, though the universal request tools can reach them if your credentials authorize access.

## Risk Communication Notes

### Positioning Language

All documentation now uses **"maximum coverage of the X API v2 public surface"** rather than "everything" or "complete X API access." This is:
- Ambitious: 109 tools covering all standard-tier v2 endpoint categories
- Truthful: Explicitly excludes Ads, DMs, Enterprise-only, v1.1, and platform-admin
- Differentiated: Still vastly more than thin X MCP wrappers (which typically offer 10-20 tools)

### Admin Profile Communication

The admin profile is now documented with:
- Precise scope: "superset of Write + 4 universal request tools"
- Clear negatives: does NOT grant platform admin, does NOT bypass policy, does NOT grant Ads/DM access
- Use-case guidance: when to use admin vs when to use write

### DM Boundary Enforcement

DM endpoints (`/2/dm_conversations`, `/2/dm_events`) are on the `api.x.com` host, which is in the universal request allowlist. This means the admin profile's `x_get`/`x_post` tools could technically reach DM endpoints. The boundary is enforced by:
1. Documentation: clearly stated as "Not supported" in API Coverage Boundaries
2. Compliance section: "DM automation: Not implemented and not planned"
3. All mutations are logged to `mutation_audit_log` — DM operations would be auditable
4. Policy engine can block specific endpoints via `blocked_tools`

A future session could add an explicit path-prefix denylist to the universal request handler if stronger enforcement is desired.

## Files Changed

| File | Change |
|------|--------|
| `docs/mcp-reference.md` | Admin Profile Scope section, API Coverage Boundaries section, updated tool counts (100→104, 104→108), positioning language ("maximum public API coverage") |
| `README.md` | MCP section: 3→4 profiles, 64→109 tools, Full→Write/Admin. Compliance: added Ads and DMs with explicit status |
| `docs/operations.md` | Profile runbook: Full→Write/Admin, added admin profile to smoke tests, updated tool counts |
| `docs/configuration.md` | Admin profile note in MCP policy section |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Comment: "future ads/admin endpoints" → "ad-hoc X API v2 access" |
| `docs/roadmap/x-api-surface-expansion/session-10-handoff.md` | This document |

## Test Results

```
cargo fmt --all && cargo fmt --all --check          # clean
cargo clippy --workspace -- -D warnings             # clean
RUSTFLAGS="-D warnings" cargo test --workspace -- --test-threads=1
  tuitbot-cli:  118 passed, 0 failed
  tuitbot-core: 730 passed, 0 failed
  tuitbot-mcp:  408 passed, 0 failed, 10 ignored (live tests)
  tuitbot-server: 31 passed, 0 failed
  Total: 1288 passed, 0 failed, 10 ignored
```

Note: `env_var_override_approval_mode` test is flaky when run multi-threaded (env var race condition). Passes reliably with `--test-threads=1`. This is a pre-existing condition, not introduced by this session.

## What's NOT Changed

- No tools added or removed (still 109 total)
- No profile behavior changes — admin/write/readonly/api-readonly work exactly as before
- No policy engine changes
- No safety system changes
- No universal request handler changes (allowlist, blocklist, validation unchanged)
- Session 08 idempotency/audit system untouched
- Session 09 conformance harness and coverage report untouched

## Recommended Follow-Up

### Priority 1 (High)
1. **DM path-prefix denylist** — Add an explicit `BLOCKED_PATHS` constant to the universal request handler that rejects `/2/dm_*` paths. Currently this is a documentation-level boundary; code enforcement would be stronger.
2. **Documentation site build** — Run `mkdocs build --strict` in CI to catch broken cross-references.

### Priority 2 (Medium)
3. **Ads host denylist documentation** — Consider adding `ads-api.x.com` to `ALLOWED_HOSTS` with a comment explaining why it's absent, to make the exclusion visible to future contributors.
4. **Enterprise endpoint inventory** — Document which specific endpoints require Premium/Enterprise access so users know what the universal request tools can/cannot reach on their plan.

### Priority 3 (Low)
5. **CHANGELOG update** — Add a changelog entry for the documentation clarification when the next release PR is created.
6. **Profile count automation** — Wire manifest tool counts into documentation generation so counts never drift again.
