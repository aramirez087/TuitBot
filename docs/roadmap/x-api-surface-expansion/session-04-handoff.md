# Session 04 Handoff — Spec Pack and Tool Generation

## What Changed

Introduced a versioned X API spec pack and a generator pipeline that produces
typed MCP tool entries from declarative endpoint definitions. This eliminates
hand-writing boilerplate for each new X API endpoint and makes coverage
expansion pipeline-driven.

### Architecture

```
spec/mod.rs          ← module root, version constants
spec/params.rs       ← EndpointDef, ParamDef, HttpMethod, ParamType
spec/endpoints.rs    ← 36 static endpoint definitions (the spec pack)
spec/generator.rs    ← EndpointDef → ToolEntry + JSON Schema
spec/tests.rs        ← 18 tests for determinism, naming, completeness

manifest.rs          ← all_tools() now merges curated + generated
                       ProfileManifest now has version triplet
```

**Data flow:**

```
SPEC_ENDPOINTS (static array of EndpointDef)
       │
       ▼
generate_spec_tools() → Vec<ToolEntry>
       │
       ▼
all_tools() = all_curated_tools() ∪ generate_spec_tools()
       │
       ▼
generate_manifest() / generate_profile_manifest(profile)
       │
       ▼
JSON artifacts in docs/generated/
```

### Versioning Contract

Three independent version strings exposed in every `ProfileManifest`:

| Field | Source | Current | Meaning |
|-------|--------|---------|---------|
| `tuitbot_mcp_version` | `CARGO_PKG_VERSION` | `0.1.8` | Crate release version |
| `mcp_schema_version` | `spec::MCP_SCHEMA_VERSION` | `1.1` | Manifest format version (bumped from 1.0) |
| `x_api_spec_version` | `spec::X_API_SPEC_VERSION` | `1.0.0` | Spec pack content version (semver) |

**When to bump each:**

- `tuitbot_mcp_version`: automatic on release via release-plz
- `mcp_schema_version`: bump when `ProfileManifest` or `ToolManifest` struct changes
- `x_api_spec_version`: bump when endpoints are added, removed, or modified in `endpoints.rs`

### Naming Convention

All generated tools follow `x_v2_<group>_<operation>`:

| Group | Operations |
|-------|-----------|
| `tweets` | `lookup`, `retweeted_by`, `quote_tweets`, `counts_recent`, `hide_reply`, `unhide_reply` |
| `users` | `lookup_by_usernames`, `pin_tweet`, `unpin_tweet` |
| `lists` | `get`, `owned`, `create`, `update`, `delete`, `tweets`, `members`, `members_add`, `members_remove`, `memberships`, `followers`, `follow`, `unfollow`, `pinned`, `pin` |
| `mutes` | `list`, `create`, `delete` |
| `blocks` | `list`, `create`, `delete` |
| `spaces` | `get`, `lookup`, `by_creator`, `search`, `buyers`, `tweets` |

### New Tool Categories

Two new `ToolCategory` variants added to the manifest enum:

- **`List`** — List CRUD, membership, following, tweets (15 tools)
- **`Moderation`** — Mutes, blocks, hide/unhide replies (7 tools)

### Tool Count Changes

| Profile | Before | After | Delta |
|---------|--------|-------|-------|
| full (Workflow) | 68 | 104 | +36 |
| api-readonly | 20 | 40 | +20 |
| readonly | 10 | 14 | +4 |

### Determinism Guarantees

1. `generate_spec_tools()` output is sorted alphabetically by tool name
2. `generate_tool_schemas()` output is sorted alphabetically by tool name
3. Two consecutive calls produce byte-for-byte identical JSON (tested)
4. `all_tools()` output is sorted alphabetically (curated + generated merged)
5. Snapshot test (`manifest_snapshot`) catches any drift

## Files Changed

| File | Change |
|------|--------|
| `crates/tuitbot-mcp/src/spec/mod.rs` | New — module root, version constants, re-exports |
| `crates/tuitbot-mcp/src/spec/params.rs` | New — EndpointDef, ParamDef, HttpMethod, ParamType types |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | New — 36 endpoint definitions (the spec pack) |
| `crates/tuitbot-mcp/src/spec/generator.rs` | New — generator: EndpointDef → ToolEntry + ToolSchema |
| `crates/tuitbot-mcp/src/spec/tests.rs` | New — 18 tests for spec pack and generator |
| `crates/tuitbot-mcp/src/lib.rs` | Added `pub mod spec` |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Added `List`/`Moderation` categories, version triplet in ProfileManifest, `all_tools()` merges curated+generated |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Updated tool counts (14/40/104), mutation denylist (+16 entries), version triplet test |
| `scripts/check-mcp-manifests.sh` | Updated field name: `tuitbot_version` → `tuitbot_mcp_version` |
| `roadmap/artifacts/session-05-tool-manifest.json` | Regenerated with 104 tools |
| `docs/generated/mcp-manifest-full.json` | Regenerated (104 tools, version triplet) |
| `docs/generated/mcp-manifest-api-readonly.json` | Regenerated (40 tools, version triplet) |
| `docs/generated/mcp-manifest-readonly.json` | Regenerated (14 tools, version triplet) |

## Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D-018 | Spec as pure Rust `static` arrays, not YAML/JSON | Compile-time validation, no runtime parsing, trivial integration. IDE support for refactoring. |
| D-019 | Generator is a function, not a build-time codegen step | Simplicity — `generate_spec_tools()` returns `Vec<ToolEntry>` directly. No proc macros, no build.rs complexity. Code-generation can be added later if perf matters. |
| D-020 | `x_v2_` prefix for all generated tools | Clear namespace separation from curated Layer 1 tools. Agents can identify generated tools by prefix. |
| D-021 | Mutation tools get `requires_db: true` | All mutations need policy engine + idempotency, both of which require DB. Enforced by generator. |
| D-022 | Read tools use `Lane::Shared`, mutation tools use `Lane::Workflow` | Reads can be used across profiles; mutations are workflow-only. Matches existing convention. |
| D-023 | Version bumped to `mcp_schema_version: "1.1"` | `ProfileManifest` added `x_api_spec_version` field and renamed `tuitbot_version` → `tuitbot_mcp_version`. This is a backward-incompatible schema change. |

## How to Extend for New Endpoints

1. **Add endpoint definition** to `crates/tuitbot-mcp/src/spec/endpoints.rs`:
   ```rust
   EndpointDef {
       tool_name: "x_v2_<group>_<operation>",
       description: "Human-readable description",
       method: HttpMethod::Get,  // or Post/Put/Delete
       path: "/2/endpoint/{id}/path",
       category: ToolCategory::Read,
       profiles: WF_AND_API_RO,
       scopes: &["scope.read"],
       params: &[PARAM_ID, PARAM_MAX_RESULTS],
       error_codes: X_READ_ERR,
       api_version: "v2",
       group: "group_name",
   },
   ```

2. **Bump `X_API_SPEC_VERSION`** in `spec/mod.rs`

3. **Update boundary tests** in `boundary_tests.rs`:
   - Adjust profile tool counts
   - Add mutation tools to `mutation_denylist()` if applicable

4. **Regenerate artifacts**:
   ```bash
   cargo test -p tuitbot-mcp manifest_snapshot  # regenerates snapshot
   bash scripts/generate-mcp-manifests.sh       # regenerates profile manifests
   ```

5. **Run CI checklist**:
   ```bash
   cargo fmt --all && cargo fmt --all --check
   RUSTFLAGS="-D warnings" cargo test --workspace
   cargo clippy --workspace -- -D warnings
   ```

## Quality Gate Results

```
cargo fmt --all --check          ✅ pass
cargo clippy --workspace         ✅ pass (0 warnings)
cargo test --workspace           ✅ 1,233 tests passed
  - tuitbot-cli:  118 passed
  - tuitbot-core: 718 passed
  - tuitbot-mcp:  365 passed (including 18 new spec tests)
  - tuitbot-server: 32 passed
scripts/generate-mcp-manifests.sh ✅ 3 manifests generated
```

## Next Session Inputs

Session 05 should wire the generated tools into strict profile enforcement:

- **Handler registration**: Generated tools need actual request handlers dispatching
  to the universal `x_get`/`x_post`/`x_put`/`x_delete` layer from Session 03.
  The spec pack provides `method`, `path`, and `params` — the handler should
  interpolate path params, build query/body from tool input, and dispatch.
- **Profile enforcement**: Ensure generated read tools are callable in their
  declared profiles (readonly, api-readonly) and mutations are workflow-only.
- **Capability metadata**: Add `requires_scope` and `requires_user_auth` fields
  from the spec pack's `scopes` data to the manifest entries.
