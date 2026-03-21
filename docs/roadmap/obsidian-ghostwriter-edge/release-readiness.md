# Release Readiness: Ghostwriter Edge

## Recommendation

**CONDITIONAL GO**

The Ghostwriter Edge feature set is complete against the epic charter with all 16 requirements passing validation. All CI gates are green. The system is ready for release with two conditions:

1. **Manual smoke test** of the Obsidian plugin → TuitBot Desktop flow before public announcement (automated E2E was explicitly out of scope per the charter's non-goals).
2. **User-facing privacy documentation** should ship alongside or shortly after release to explain what "local-first" means in the TuitBot context.

## Feature Completeness vs Charter

| Charter Capability | Status | Session |
|-------------------|--------|---------|
| Block-level send from Obsidian | Complete | 4 |
| Selection ingress API with TTL + rate limiting | Complete | 4 |
| Hook-first thread drafting (5 differentiated hooks) | Complete | 5-6 |
| Provenance tracking with seed_id support | Complete | 3 |
| Provenance propagation through content lifecycle | Complete | 7 |
| Heading-anchor deep-links | Complete | 3 |
| Privacy-by-deployment (Desktop/SelfHost/Cloud) | Complete | 8 |
| Cloud guard on vault path + selected_text | Complete | 8 |
| Privacy banners and badges | Complete | 8 |
| Obsidian transport awareness notice | Complete | 8 |
| Tauri privacy fallback command | Complete | 8 |
| Snippet truncation (120 chars universal) | Complete | 2 |
| No raw body_text in API responses | Complete | 2 |
| Account-scoped queries | Complete | 2 |
| 30-minute selection TTL | Complete | 4 |

**16/16 charter requirements implemented and validated.**

## Residual Risks

| Risk | Severity | Mitigation | Blocking? |
|------|----------|------------|-----------|
| No runtime E2E test for Obsidian → TuitBot flow | Medium | Manual smoke test before release; unit/integration tests cover each boundary | No (charter non-goal) |
| `general` label in hookStyles.ts has no backend variant | Low | Graceful fallback auto-formats unknown keys; no runtime error possible | No |
| Obsidian plugin has no settings UI | Low | Users must edit `serverUrl` in plugin source; documented in plugin contract | No |
| No publish audit trail in timeline view | Low | Provenance is stored and queryable; UI display deferred to follow-up | No |
| Cloud deployment mode untested at runtime | Medium | All Cloud guards validated in integration tests; needs real Cloud deployment for full validation | No (Cloud mode is future) |

## Known Limitations (Non-Blocking)

- **No Obsidian community plugin listing**: The plugin is a reference implementation, not a community-published plugin. Users must manually install.
- **No plugin settings tab**: `serverUrl` defaults to `http://127.0.0.1:3001` with no UI to change it. Documented in `obsidian-plugin-contract.md`.
- **No analytics by hook style**: Hook style is captured in provenance (`assist:hook:{style}`) but no dashboard analytics view aggregates conversion rates by style.
- **No E2E Playwright tests**: The charter explicitly excluded building a full E2E test harness. Coverage relies on unit/integration tests at each boundary.
- **Selection cleanup is passive**: Expired selections are filtered at query time (`WHERE expires_at > datetime('now')`). No active cleanup cron exists. This is adequate for typical usage volumes.

## Rollback Plan

The Ghostwriter Edge feature set is additive — it introduces new API routes, UI panels, and capabilities without modifying existing flows. Rollback is straightforward:

1. **API routes**: All new routes (`/api/vault/send-selection`, `/api/assist/hooks`) are isolated. Removing them does not affect existing compose, approval, or scheduling flows.
2. **Provenance**: The `provenance_links` table and storage module were added fresh. Removing them orphans no existing data.
3. **Privacy fields**: `DeploymentCapabilities` uses `#[serde(default)]` for new fields. Reverting the struct preserves backward compatibility — old clients already ignore unknown fields.
4. **Frontend panels**: `FromVaultPanel`, `HookPicker`, `VaultHighlights`, and `VaultSelectionReview` are conditionally rendered. Removing them leaves the composer functional.
5. **Obsidian plugin**: Standalone package with no coupling to dashboard runtime. Can be unpublished independently.

**Recommended rollback procedure**: Revert to the commit before Session 2 (`git revert` the session 2-8 commits). All existing tests will continue to pass since no pre-existing behavior was modified.

## Follow-Up Work

These items are explicitly separated from the release decision. None are release blockers.

| Item | Priority | Description |
|------|----------|-------------|
| Obsidian plugin settings tab | P2 | UI for changing `serverUrl` and viewing transport indicator |
| Publish audit trail in timeline | P2 | Show provenance citations on posted tweets in the timeline view |
| Analytics by hook style | P3 | Dashboard view showing conversion rates by hook style (question, hot take, etc.) |
| User-facing privacy docs | P1 | Explain what "local-first" means in the TuitBot context for marketing and support |
| E2E smoke test automation | P3 | Playwright or similar for the Obsidian → TuitBot → publish flow |
| Selection cleanup cron | P3 | Active cleanup of expired vault selections (passive filtering is sufficient for now) |
| Hook style A/B testing | P3 | Experiment with different hook style distributions to optimize engagement |
