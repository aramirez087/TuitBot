# Multi-Account Release Readiness

## Ship Recommendation: GO

The multi-account feature is ready to ship. All critical singleton seams are resolved, quality gates pass, backward compatibility is preserved, and the deferred items carry no data-leakage risk.

## What Ships

### Sessions 1-9 Deliverables

| Session | Deliverable | Status |
|---------|-------------|--------|
| 1 | Charter, implementation plan, singleton seam inventory | Complete |
| 2 | Effective config resolution (`config/merge.rs`), per-account file layout (`account_data_dir`), settings API account-scoping | Complete |
| 3 | Account-scoped credential paths, scraper session isolation, token manager per-account keying | Complete |
| 4 | WebSocket `AccountWsEvent` wrapper, client-side event filtering, `clearEvents()` on switch | Complete |
| 5 | Dashboard account management UX (create, rename, archive, switch), store invalidation on switch, page refetch listeners | Complete |
| 6 | Account management polish, credential badge display, settings section extraction | Complete |
| 7 | Settings override UX (scope badges, instance lockout, reset-to-base, dirty draft discard) | Complete |
| 8 | Per-account credential management (OAuth link/relink/unlink, scraper import/replace/remove) | Complete |
| 9 | Approval stats refetch on switch fix, QA matrix, release readiness, epic closure | Complete |

### Key Capabilities

- **Account registry**: Create, rename, archive accounts. Default account is immutable sentinel.
- **Credential isolation**: Per-account OAuth tokens and scraper sessions in isolated file paths.
- **Config isolation**: Per-account overrides stored in DB, merged at runtime via `effective_config()`.
- **Settings UX**: Account-scoped vs instance-scoped sections, override badges, reset-to-base.
- **Account switching**: Instant switch with full store invalidation, WS event flush, stats reload.
- **WebSocket scoping**: Events carry `account_id`, client filters by active account.
- **Runtime isolation**: Per-account runtime map, token manager map, content generator map.
- **In-dashboard credential linking**: OAuth PKCE and scraper session import without leaving the app.

## Quality Gates (Final)

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Clean |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | 1,935 tests pass (0 fail, 11 ignored) |
| `cargo clippy --workspace -- -D warnings` | Clean |
| `npm --prefix dashboard run check` | 0 errors, 7 warnings (all pre-existing a11y/style) |
| `npm --prefix dashboard run build` | Success |

## Test Coverage Summary

| Category | Count | Key Areas |
|----------|-------|-----------|
| Config merge unit tests | 16 | Deep merge, override validation, edge cases |
| Account path helper tests | 6 | Default vs non-default, directory creation |
| Account CRUD integration tests | 8 | Create, update, archive, list, default protection |
| Credential isolation tests | 6 | OAuth unlink, cross-account isolation, scraper session |
| WebSocket event tests | 4 | Serialization with account_id, filtering |
| Settings API tests | 8 | Account-scoped read/write, override save/reset |
| Full workspace | 1,935 | All crates, all modules |

## Charter vs Implementation Reconciliation

| Charter Item | Planned | Actual | Status |
|---|---|---|---|
| S1: Config loading | Effective config resolution | `effective_config()` in `config/merge.rs` with JSON merge-patch | **Complete** |
| S2: Token/scraper files | Per-account file layout | `account_data_dir()` helpers, per-account paths | **Complete** |
| S3: Automation loops | Thread `account_id` through all loops | `_for()` storage variants exist; runtime creates empty `Runtime` (no loops spawned from dashboard) | **Deferred** — no data leakage risk |
| S4: WebSocket events | Account-scoped events | `AccountWsEvent` wrapper, client filtering | **Complete** |
| S5: Watchtower per-account | Content source attribution | Session 6 repurposed for account management UX (higher value) | **Deferred** — content sources are config-scoped |
| S6: Settings UX | Account-scoped settings | Scope badges, instance lockout, reset-to-base | **Complete** |
| S7: Content generator | Lazy per-account init | `get_or_create_content_generator()` | **Complete** |
| S8: CLI awareness | CLI account support | Dashboard is primary UX; CLI uses default account | **Deferred** — by design |
| D5: In-dashboard credential linking | OAuth + scraper session | Full link/relink/unlink/import/replace/remove UX | **Complete** |
| Implementation sessions | 2-7 (6 sessions) | 1-9 (9 sessions, expanded scope for management UX and credential UX) | **Expanded** |

## Known Limitations

### Deferred Items (No Data Leakage Risk)

1. **S3: Automation loop wiring** — `_for()` storage variants exist but loops aren't spawned from dashboard yet. Runtime `start` creates an empty `Runtime` struct. When loop spawning is implemented, it must use the `_for()` variants. No risk: loops don't run, so no cross-account data access occurs.

2. **S5: Watchtower per-account** — Content sources are account-scoped in config but Watchtower ingestion is instance-level. Content nodes are not attributed to a specific account at ingestion time. Acceptable because Watchtower feeds into content generation which is already per-account (via `get_or_create_content_generator`).

3. **S8: CLI single-account** — CLI remains unaware of multi-account. Uses default account implicitly via missing `X-Account-Id` header. Dashboard is the primary UX.

### Accepted Open Issues

4. **Pre-switch confirmation modal** — Switching accounts auto-discards dirty drafts with a notification. No data loss risk (changes are local drafts only). Enhancement for future.

5. **Config reload after PATCH** — Running runtimes don't pick up config changes until restarted. Since runtimes are empty (no loops), this has no practical impact. Document as known behavior.

6. **Field-level override indicators** — Current badges are section-level. Matches backend merge granularity. Enhancement for future.

7. **OAuth auto-redirect** — Paste-code UX works. Auto-redirect with `local_callback` is a future enhancement.

8. **Scraper session validation** — Accepts any `auth_token`/`ct0` values. Invalid sessions fail at posting time with clear error messages.

## Rollback Guidance

The multi-account feature is fully additive. Rolling back is safe:

1. **Database**: The `accounts` table and `account_id` columns remain but are unused. Default values ensure all data maps to the default account. No destructive migration exists.

2. **Files**: `accounts/{uuid}/` directories remain on disk but are ignored by pre-multi-account code. Root-level `tokens.json` and `scraper_session.json` continue to work for the default account.

3. **Config**: `config.toml` is unchanged. `config_overrides` JSON in the `accounts` table is ignored by pre-multi-account code.

4. **Frontend**: Reverting dashboard code removes the switcher and account management UX. All pages fall back to default account behavior (no `X-Account-Id` header).

5. **Procedure**: `git revert` the multi-account commits. Run `cargo build` and `npm --prefix dashboard run build`. No data migration needed.

## Contract Documents

| Document | Path |
|----------|------|
| Charter | `docs/roadmap/dashboard-multi-account/charter.md` |
| Implementation Plan | `docs/roadmap/dashboard-multi-account/implementation-plan.md` |
| Settings Scope Matrix | `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md` |
| Credential Isolation Contract | `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md` |
| Runtime Isolation Plan | `docs/roadmap/dashboard-multi-account/runtime-isolation-plan.md` |
| Frontend Switching Flow | `docs/roadmap/dashboard-multi-account/frontend-switching-flow.md` |
| Account Management Flow | `docs/roadmap/dashboard-multi-account/account-management-flow.md` |
| Settings Override UX | `docs/roadmap/dashboard-multi-account/settings-override-ux.md` |
| X Access Account Flow | `docs/roadmap/dashboard-multi-account/x-access-account-flow.md` |
| QA Matrix | `docs/roadmap/dashboard-multi-account/qa-matrix.md` |
| Session Handoffs | `docs/roadmap/dashboard-multi-account/session-{01..09}-handoff.md` |
