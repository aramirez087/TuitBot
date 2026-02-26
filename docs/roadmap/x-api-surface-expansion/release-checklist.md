# X API Surface Expansion — Release Checklist

**Date:** 2026-02-26
**Session:** 11 (Final)
**Status:** All items verified

## Build & Lint

- [x] `cargo fmt --all` — clean
- [x] `cargo fmt --all --check` — no formatting drift
- [x] `cargo clippy --workspace -- -D warnings` — clean, zero warnings
- [x] `RUSTFLAGS="-D warnings" cargo test --workspace -- --test-threads=1` — 1288 passed, 0 failed, 10 ignored

## Conformance Harness

- [x] Phase 1 — Kernel conformance: 34 passed, 0 failed, 10 ignored (live tests)
- [x] Phase 2 — Contract envelope: 32 passed, 0 failed
- [x] Phase 3 — Golden fixtures: 9 passed, 0 failed
- [x] Phase 4 — Boundary/profile isolation: 40 passed, 0 failed
- [x] Phase 5 — Eval scenarios: 5 passed, 0 failed
- [x] Coverage report generated: `docs/generated/coverage-report.{json,md}`

## Manifests

- [x] `scripts/generate-mcp-manifests.sh` — regenerated all 4 manifests
- [x] `scripts/check-mcp-manifests.sh` — all manifests in sync, zero drift
- [x] Stale `mcp-manifest-full.json` removed (replaced by `mcp-manifest-write.json`)
- [x] Manifest file listing:
  - `mcp-manifest-write.json` — 104 tools, 35 mutations
  - `mcp-manifest-admin.json` — 108 tools, 38 mutations
  - `mcp-manifest-readonly.json` — 14 tools, 0 mutations
  - `mcp-manifest-api-readonly.json` — 40 tools, 0 mutations

## Profile Safety Verification

- [x] **Readonly (14 tools)**: Zero mutations. Structural enforcement via `ReadonlyMcpServer` impl block. Boundary tests pass.
- [x] **Api-readonly (40 tools)**: Zero mutations. Structural enforcement via `ApiReadonlyMcpServer` impl block. Boundary tests pass.
- [x] **Write (104 tools)**: 35 mutations, all policy-gated via `check_policy()`. No universal request tools registered. Boundary test `write_server_does_not_register_universal_tools` passes.
- [x] **Admin (108 tools)**: 38 mutations. 4 universal request tools (x_get, x_post, x_put, x_delete) are admin-only. Boundary test `admin_server_registers_universal_tools` passes.

## Safety Infrastructure

- [x] Host allowlist: `api.x.com`, `upload.x.com`, `upload.twitter.com` — hard-coded, enforced at client level
- [x] SSRF guard: path validation (no `..`, no `?#`, no control chars), IP literal rejection (IPv4, IPv6, bracket syntax)
- [x] Header blocklist: 7 restricted headers (authorization, host, cookie, set-cookie, transfer-encoding, proxy-authorization, proxy-connection)
- [x] Idempotency: 30s in-memory + 5min DB-backed dedup via `mutation_audit` table
- [x] Rollback guidance: all typed mutation tools include reversibility metadata and undo parameters

## Documentation Consistency

- [x] README.md — 4-profile model, correct tool counts (14/40/104/108)
- [x] docs/mcp-reference.md — admin scope, API coverage boundaries, accurate policy enforcement description
- [x] docs/operations.md — correct profile names and tool counts in runbook
- [x] docs/configuration.md — accurate admin profile policy description
- [x] docs/cli-reference.md — updated profile names and tool counts
- [x] docs/contributing.md — updated manifest file list (4 files, correct names)

## Known Caveats (Non-Blocking)

- [ ] **Test coverage at 41.3%** (45/109 tools) — 64 tools lack dedicated tests. All 36 generated L2 tools, 4 composite, 4 content, 2 dry-run, 1 media. Tracked as post-launch backlog.
- [ ] **Universal request mutations bypass policy engine** — x_post, x_put, x_delete do not call `check_policy()` or log to `mutation_audit`. Documented accurately; constrained by host allowlist + SSRF guard + admin-only profile.
- [ ] **DM boundary is documentation-level** — `/2/dm_*` paths not blocked in code. DM endpoints on `api.x.com` host. Tracked for post-launch path-prefix denylist.
- [ ] **Flaky test** — `env_var_override_approval_mode` fails intermittently under parallel execution (env var race). Pre-existing, passes with `--test-threads=1`.
