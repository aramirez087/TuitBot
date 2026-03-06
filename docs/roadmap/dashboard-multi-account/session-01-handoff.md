# Session 01 Handoff

## What Was Done

Audited the full multi-account foundation across 12+ repository anchors and produced:

1. **`charter.md`** — Problem statement, complete inventory of existing multi-account infrastructure, 8 singleton seams (S1-S8) with severity ratings, 6 design decisions (D1-D6) with rationale, UX goals, and risk matrix.
2. **`implementation-plan.md`** — 6-session sequence (Sessions 2-7) with specific file targets, test requirements, and regression risks per session.

## Key Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Config split: `server`, `storage`, `logging`, `llm`, `connectors`, `circuit_breaker` are instance-scoped; `business`, `scoring`, `limits`, `intervals`, `mode`, `schedule`, `targets`, `content_sources`, `x_api`, `approval_mode` are account-scoped | Keeps single-user product (one LLM key, one server) while allowing distinct personas |
| D2 | `effective_config()` in `tuitbot-core::config`, not server | Respects server-boundary rule from architecture.md |
| D3 | Default account uses root files; new accounts get `accounts/{uuid}/` subdirectory | Zero-migration backward compatibility |
| D4 | `account_id` on every WsEvent variant, client-side filtering | Simpler than per-account broadcast channels |
| D5 | In-dashboard credential linking (OAuth PKCE or scraper session import) | Primary UX, not CLI-only |
| D6 | Default account sentinel is permanent, always admin, backward-compatible fallback | No breaking change for existing users |

## Singleton Seams Identified

| Seam | Severity | Targeted Session |
|------|----------|-----------------|
| S1: Config loading (single global config.toml) | CRITICAL | Session 2 |
| S2: Token & scraper session files (hardcoded paths) | CRITICAL | Session 2 |
| S3: Automation loops ignore account_id | CRITICAL | Session 3 |
| S4: WebSocket events unscoped | HIGH | Session 4 |
| S5: Watchtower is global | MEDIUM | Session 6 |
| S6: Settings UX is instance-level | MEDIUM | Session 5 |
| S7: Content generator init (default only) | LOW | Session 2 (lazy-init) |
| S8: CLI has no account awareness | LOW | Deferred (fallback UX) |

## Open Issues for Session 2

1. **`effective_config` merge semantics:** Should array fields (e.g., `content_sources.sources`) be replaced entirely or merged by key? Recommendation: replace entirely — arrays are hard to merge meaningfully.

2. **Account creation directory:** When `POST /api/accounts` is called, should the `accounts/{id}/` directory be created immediately or lazily on first credential save? Recommendation: lazily, to avoid empty directories.

3. **Scraper session endpoint backward compat:** The existing `GET/POST/DELETE /api/settings/scraper-session` endpoints have no `X-Account-Id` awareness. Should the old endpoints be preserved as aliases for default account? Recommendation: yes — the `AccountContext` extractor defaults to default account when header is missing, so existing behavior is preserved automatically.

## Exact Inputs for Session 2

### Files to Modify

- `crates/tuitbot-core/src/config/mod.rs` — Add `effective_config()` public function
- `crates/tuitbot-core/src/storage/accounts.rs` — Add `account_data_dir()` helper
- `crates/tuitbot-server/src/routes/scraper_session.rs` — Add `AccountContext` extractor, resolve per-account paths
- `crates/tuitbot-server/src/routes/accounts.rs` — Set `token_path` on creation, create account directory
- `crates/tuitbot-server/src/routes/settings.rs` — `get_settings`/`patch_settings` merge account overrides when non-default

### Files to Create

- `crates/tuitbot-core/src/config/merge.rs` — JSON merge-patch logic (or inline in `mod.rs` if small)

### Tests to Run

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### Key Contracts to Respect

- `effective_config` lives in `tuitbot-core`, not `tuitbot-server` (architecture boundary)
- Default account file paths remain at root level (backward compat, D3)
- `config_overrides` JSON is validated against Config schema before save (D2)
- `AccountContext` extractor already defaults to `DEFAULT_ACCOUNT_ID` when header is missing (D6)
