# Session 06 Handoff — Capability Discovery and Auth Metadata

## What Changed

Before this session, the `get_capabilities` tool returned tier info, boolean flags, rate limits, and a hardcoded tool availability map — but had no visibility into OAuth scopes, no per-tool auth requirements, and no actionable remediation guidance. Agents would discover missing permissions only when calls failed.

This session adds truthful capability discovery: scope analysis, endpoint group availability, per-tool auth metadata, and actionable guidance so users and agents can see exactly what their credentials can do before calls fail.

### Changes Summary

1. **`ToolEntry` extended** with `requires_scopes`, `requires_user_auth`, `requires_elevated_access` fields. MCP schema version bumped from 1.1 to 1.2.
2. **All curated X API tools** annotated with correct OAuth scopes and auth requirements (30+ tools converted from `tool()` to `x_tool()` helper).
3. **Generator updated** to populate scope/auth metadata from `EndpointDef` for all 36 generated spec tools.
4. **`AppState` extended** with `granted_scopes: Vec<String>` field, populated from tokens at startup.
5. **`get_capabilities` response expanded** with four new sections: `auth`, `scope_analysis`, `endpoint_groups`, `guidance`.
6. **12 new tests** covering scope analysis, endpoint groups, auth info, guidance messages, and per-tool scope metadata.

## Capability Model

### Response Structure

```json
{
  "success": true,
  "data": {
    "tier": "Basic",
    "tier_detected_at": "2025-01-15T10:30:00Z",
    "can_post_tweets": true,
    "can_reply": true,
    "can_search": true,
    "can_discover": true,
    "approval_mode": false,
    "llm_available": true,
    "auth": {
      "mode": "oauth2_user_context",
      "x_client_available": true,
      "authenticated_user_id": "123456",
      "token_scopes_available": true
    },
    "scope_analysis": {
      "granted": ["tweet.read", "tweet.write", "users.read", "..."],
      "required": ["tweet.read", "tweet.write", "users.read", "..."],
      "missing": [],
      "extra": ["mute.read"],
      "degraded_features": [],
      "all_required_present": true
    },
    "endpoint_groups": [
      {
        "group": "tweets",
        "total_endpoints": 7,
        "available_endpoints": 7,
        "required_scopes": ["tweet.read", "tweet.write", "tweet.moderate.write", "users.read"],
        "missing_scopes": [],
        "fully_available": true
      },
      {
        "group": "lists",
        "total_endpoints": 15,
        "available_endpoints": 0,
        "required_scopes": ["list.read", "list.write", "tweet.read", "users.read"],
        "missing_scopes": ["list.read", "list.write"],
        "fully_available": false
      }
    ],
    "rate_limits": [...],
    "recommended_max_actions": { "replies": 5, "tweets": 6, "threads": 1 },
    "direct_tools": {
      "x_client_available": true,
      "authenticated_user_id": "123456",
      "tools": [
        {
          "name": "x_post_tweet",
          "available": true,
          "requires_scopes": ["tweet.read", "tweet.write", "users.read"]
        }
      ]
    },
    "provider": { "backend": "x_api", "mutations_available": true, "..." : "..." },
    "guidance": ["All systems operational. No issues detected."]
  }
}
```

### Auth Metadata on Tool Manifest

Every tool now declares:

| Field | Type | Description |
|-------|------|-------------|
| `requires_scopes` | `Vec<String>` | OAuth scopes needed (empty for non-X tools) |
| `requires_user_auth` | `bool` | Needs OAuth user-context token |
| `requires_elevated_access` | `bool` | Needs admin profile (universal request tools only) |

These fields are populated:
- **Curated tools** (Layer 1): manually annotated with correct scopes in `manifest.rs`
- **Generated tools** (Layer 2): derived from `EndpointDef.scopes` in `generator.rs`

### Guidance Engine

The `guidance` array provides actionable remediation messages:

| Condition | Guidance |
|-----------|----------|
| No X client | "Run `tuitbot auth` to authenticate with OAuth 2.0." |
| No user ID | "Token may be valid but get_me() failed. Check network." |
| No scopes available | "Re-authenticate with `tuitbot auth` to capture granted scopes." |
| Missing required scopes | "Missing required scopes: X, Y. Re-authenticate with `tuitbot auth`." |
| Degraded features | "Like/unlike is degraded — missing: like.write." |
| Free tier | "Free tier detected. Search, mentions, and discovery are unavailable." |
| Unknown tier | "API tier not yet detected. Run a search to trigger detection." |
| No LLM | "LLM provider not configured. Content generation tools will not work." |
| All clear | "All systems operational. No issues detected." |

## Files Changed

| File | Change |
|------|--------|
| `crates/tuitbot-mcp/src/state.rs` | +`granted_scopes` field on `AppState` |
| `crates/tuitbot-mcp/src/lib.rs` | Capture scopes from tokens at startup, pass to `AppState` |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | +`requires_scopes`, `requires_user_auth`, `requires_elevated_access` on `ToolEntry`; +`x_tool()` helper; 30+ curated X tools annotated |
| `crates/tuitbot-mcp/src/spec/generator.rs` | Populate scope/auth metadata from `EndpointDef` |
| `crates/tuitbot-mcp/src/spec/mod.rs` | `MCP_SCHEMA_VERSION` bumped to 1.2 |
| `crates/tuitbot-mcp/src/tools/workflow/capabilities.rs` | Complete rewrite: +auth info, +scope analysis, +endpoint groups, +guidance, +per-tool scope metadata; 12 new tests |
| `crates/tuitbot-mcp/src/server/write.rs` | Pass `granted_scopes` to `get_capabilities` |
| `crates/tuitbot-mcp/src/server/admin.rs` | Pass `granted_scopes` to `get_capabilities` |
| `crates/tuitbot-mcp/src/tools/benchmark/baseline.rs` | Updated call site |
| `crates/tuitbot-mcp/src/tools/benchmark/expanded.rs` | Updated call site |
| `crates/tuitbot-mcp/src/tools/eval_harness/mocks.rs` | +`granted_scopes: vec![]` |
| `crates/tuitbot-mcp/src/tools/eval_session09/helpers.rs` | +`granted_scopes: vec![]` |
| `crates/tuitbot-mcp/src/tools/workflow/composite/tests.rs` | +`granted_scopes: vec![]` |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/tests/mod.rs` | +`granted_scopes: vec![]` |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerated with new schema |
| `docs/roadmap/x-api-surface-expansion/session-06-handoff.md` | This document |

## Known Unknowns

### Where X Does Not Expose Clear Tier Signals

1. **Tier detection is heuristic.** We probe the search endpoint and infer tier from the HTTP status (403 = Free, 200/429 = Basic/Pro). X does not provide a direct "what tier am I?" endpoint. The `tier_detected_at` timestamp helps agents decide if re-detection is needed.

2. **No runtime scope verification.** We report scopes from the token's `scope` field at auth time. If X revokes a scope server-side without revoking the token, we won't detect it until a 403 response. The guidance suggests re-authenticating if unexpected 403s occur.

3. **Elevated access (ads, analytics) is not detectable.** X's elevated access tiers (for academic research, ads API, etc.) are not surfaceable through standard OAuth endpoints. The `requires_elevated_access` flag on manifest entries refers to our admin profile, not X's elevation.

4. **Rate limit headers are not persisted.** X returns `x-rate-limit-remaining` headers on every response, but we only track our internal rate limits (replies/tweets/threads per day). X's per-endpoint rate limits are not reflected in the capability response.

### Scope Coverage Gaps

The `REQUIRED_SCOPES` constant only includes the 10 scopes Tuitbot requests by default. Generated tools that need additional scopes (e.g., `list.read`, `list.write`, `mute.read`, `mute.write`, `block.read`, `block.write`, `space.read`, `tweet.moderate.write`) are correctly annotated in the manifest and endpoint groups, but these scopes won't be granted unless the auth flow requests them. The endpoint group availability section makes this visible.

## Caller UX Guidance

### For Agents

1. **Call `get_capabilities` first** in every session to understand what's available.
2. **Check `scope_analysis.all_required_present`** — if false, core features are degraded.
3. **Check `endpoint_groups`** before using generated tools — if a group is not `fully_available`, its tools will fail.
4. **Read `guidance`** for actionable next steps when something is wrong.
5. **Check `direct_tools[name].available`** before calling any specific tool.

### For Users

1. If `guidance` mentions "Run `tuitbot auth`", do that first.
2. If `scope_analysis.missing` lists scopes, the token was granted with restricted permissions.
3. If `tier` is "Free", upgrade the X API subscription for search and discovery.
4. If `endpoint_groups` shows missing scopes for lists/mutes/blocks, these are extended scopes not requested by default. A future session could add scope negotiation to the auth flow.

## Verification

```bash
cargo fmt --all && cargo fmt --all --check     # clean
RUSTFLAGS="-D warnings" cargo test --workspace  # all pass
cargo clippy --workspace -- -D warnings         # clean
```

All 12 new capability tests pass:
- `scope_analysis_present_with_full_scopes`
- `scope_analysis_absent_without_scopes`
- `scope_analysis_reports_missing_scopes`
- `endpoint_groups_present`
- `endpoint_groups_show_missing_with_partial_scopes`
- `auth_info_present`
- `guidance_reports_no_x_client`
- `guidance_all_operational`
- `guidance_reports_free_tier`
- `guidance_reports_missing_scopes`
- `direct_tools_include_scope_metadata`

## What's NOT Changed

- `core/x_api/scopes.rs` — No changes to the scope registry
- `core/x_api/tier.rs` — No changes to tier detection
- `core/x_api/auth.rs` — No changes to token management
- `spec/endpoints.rs` — No changes to endpoint definitions
- Server readonly/api-readonly profiles — They have their own `get_capabilities` implementation

## Next Steps

- **Session 07** could add scope negotiation to the auth flow — let users request extended scopes (list.*, mute.*, block.*, space.*) during `tuitbot auth`.
- **Runtime scope verification**: when a tool gets a 403, compare against `requires_scopes` to give a specific "missing scope X" error instead of generic "forbidden".
- **Rate limit header tracking**: persist X's `x-rate-limit-*` headers from responses and surface them in capability responses.
- **Tier re-detection**: add a manual `tuitbot tier check` command or periodic re-detection.
