# Session 05 Handoff — Strict 4-Profile Model

## What Changed

Wired generated and universal tools into a strict 4-profile model so that
capability is enforced by registration (structural enforcement), not soft
policy checks. Renamed the `Full`/`Workflow` profile to `Write`, added a new
`Admin` profile as its strict superset, and moved universal request tools
(`x_get`/`x_post`/`x_put`/`x_delete`) to admin-only.

### Profile Model

```
readonly  ⊂  api-readonly     (strict subset on reads)
write     ⊂  admin            (strict subset — admin adds 4 universal tools)
api-readonly ⊥ write           (independent branches)
```

| Profile | CLI Flag | Curated | Generated | Total | Changes |
|---------|----------|---------|-----------|-------|---------|
| `readonly` | `--profile readonly` | 10 | 4 | 14 | No change |
| `api-readonly` | `--profile api-readonly` | 20 | 20 | 40 | No change |
| `write` | (default) | 64 | 36 | 100 | Renamed from `full`/`workflow`; universal tools removed |
| `admin` | `--profile admin` | 68 | 36 | 104 | New — superset of write + universal request tools |

### Enforcement Mechanism

The `rmcp` framework's `#[tool_router]` macro requires all `#[tool]` methods
in a single `impl` block with no conditional registration. This means
profile enforcement must be structural — separate server structs per profile:

```
ReadonlyMcpServer      → 10 curated handlers
ApiReadonlyMcpServer   → 20 curated handlers
WriteMcpServer         → 64 curated handlers (no universal tools)
AdminMcpServer         → 68 curated handlers (all tools including universal)
```

Tools not registered on a server simply don't exist for that profile.

### Admin-Only Tools

The 4 universal request tools allow arbitrary X API endpoint access:

| Tool | Method | Risk |
|------|--------|------|
| `x_get` | GET | Read — low |
| `x_post` | POST | Mutation — high |
| `x_put` | PUT | Mutation — high |
| `x_delete` | DELETE | Mutation — high |

These bypass the typed tool layer. Restricting them to `admin` ensures
standard agents operate through structured, policy-gated tools only.

## Files Changed

| File | Change |
|------|--------|
| `crates/tuitbot-mcp/src/state.rs` | Renamed `Profile::Full` → `Profile::Write`, added `Profile::Admin`. Legacy `"full"` maps to `Write` in `FromStr`. |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | Renamed `Profile::Workflow` → `Profile::Write`, added `Profile::Admin`. New shorthands: `ALL_FOUR`, `WRITE_UP`, `WRITE_UP_AND_API_RO`, `ADMIN_ONLY`. Universal tools moved to `ADMIN_ONLY`. |
| `crates/tuitbot-mcp/src/spec/endpoints.rs` | Updated profile shorthands: `ALL_THREE`→`ALL_FOUR`, `WF_AND_API_RO`→`WRITE_UP_AND_API_RO`, `WF`→`WRITE_UP`. |
| `crates/tuitbot-mcp/src/server/admin.rs` | **New** — `AdminMcpServer` (renamed from `TuitbotMcpServer` in `workflow.rs`). All 68 curated handlers. |
| `crates/tuitbot-mcp/src/server/write.rs` | **New** — `WriteMcpServer`. 64 curated handlers (universal request tools and `kv_to_tuples` removed). |
| `crates/tuitbot-mcp/src/server/workflow.rs` | **Deleted** — replaced by `admin.rs` + `write.rs`. |
| `crates/tuitbot-mcp/src/server/mod.rs` | Updated exports: `WriteMcpServer`, `AdminMcpServer`. |
| `crates/tuitbot-mcp/src/lib.rs` | 4-profile dispatch. `init_write_state()` shared for write/admin. Separate `run_write_server()` and `run_admin_server()`. |
| `crates/tuitbot-cli/src/commands/mod.rs` | Default profile `"full"` → `"write"` in CLI args. |
| `crates/tuitbot-cli/src/commands/mcp.rs` | Doc comment updated to "four runtime profiles". |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | New tests: `write_server_does_not_register_universal_tools`, `admin_server_registers_universal_tools`, `write_is_subset_of_admin`, `write_profile_excludes_admin_only_tools`, `admin_profile_includes_admin_only_tools`. Updated all counts. |
| `scripts/generate-mcp-manifests.sh` | `PROFILES=(write admin readonly api-readonly)` |
| `scripts/check-mcp-manifests.sh` | `PROFILES=(write admin readonly api-readonly)` |
| `docs/mcp-reference.md` | Full rewrite for 4-profile model: tool counts, profile tables, CLI examples, admin section, capability matrix. |

## Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D-024 | Separate `WriteMcpServer` and `AdminMcpServer` structs | `rmcp` requires all tools in one `impl` block. No way to conditionally register tools at runtime. Structural enforcement is the only option. |
| D-025 | Universal request tools admin-only | `x_get`/`x_post`/`x_put`/`x_delete` bypass typed tool layer. Agents should use structured tools for standard operations. Admin is explicit opt-in. |
| D-026 | `profile` field removed from `AppState` | Profile is a startup routing decision, not runtime state. The server struct identity determines the profile. Avoids dead-code warning. |
| D-027 | Legacy `"full"` maps to `Write` in `FromStr` | Backward-compatible for existing configs and scripts. |
| D-028 | Default profile is `write`, not `admin` | Principle of least privilege. Users must explicitly opt in to universal tools. |

## Known Limitations / Tech Debt

1. **Server file duplication**: `write.rs` (~1000 lines) and `admin.rs` (~1100 lines) share ~95% of their code. This is a framework limitation (`rmcp` macros). A future refactoring could use a code-generation step to produce both from a single source.
2. **Server files exceed 500-line limit**: Pre-existing (inherited from `workflow.rs`). Cannot be split due to `rmcp`'s single-impl-block requirement.
3. **Manifest artifacts not regenerated**: The `docs/generated/` JSON manifests need regeneration after this session's profile renames. Run `bash scripts/generate-mcp-manifests.sh` after build.

## Quality Gate Results

```
cargo fmt --all --check          ✅ pass
cargo clippy --workspace         ✅ pass (0 warnings)
cargo test --workspace           ✅ 1,242 tests passed
  - tuitbot-cli:  118 passed
  - tuitbot-core: 718 passed
  - tuitbot-mcp:  374 passed (+9 new boundary tests)
  - tuitbot-server: 32 passed
```

## Next Session Inputs

Session 06 should focus on:

- **Generated tool handlers**: The 36 spec-pack tools have manifest entries but no
  runtime handlers yet. Each generated tool should dispatch through the universal
  request layer (`x_get`/`x_post`/`x_put`/`x_delete` from Session 03), interpolating
  path params and building query/body from the tool's typed input schema.
- **Manifest artifact regeneration**: Run `bash scripts/generate-mcp-manifests.sh`
  to produce `write.json` and `admin.json` (replacing old `full.json`).
- **Snapshot test update**: The manifest snapshot test path was updated to
  `session-06-tool-manifest.json` — this needs to be generated.
- **Integration testing**: End-to-end test that a generated tool (e.g., `x_v2_tweets_lookup`)
  dispatches correctly through the universal request layer to a mock X API.
