# Session 08 Handoff: Deployment-Aware Privacy & Local-First Ghostwriter

## What Changed

Desktop + local vault now has a defensible local-first claim with privacy labels propagated through the API, frontend, and Obsidian plugin. Other deployment modes are explicit and honest about their privacy envelope. The system never over-claims local handling it cannot guarantee.

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-core/src/config/types.rs` | Added `is_local_first()`, `privacy_envelope()` to `DeploymentMode`; added `privacy_envelope` and `ghostwriter_local_only` fields to `DeploymentCapabilities` with `#[serde(default)]` |
| `crates/tuitbot-core/src/config/tests/deployment.rs` | 12 new privacy tests: `is_local_first`, `privacy_envelope`, `capabilities_privacy_fields`, backward compat, JSON roundtrip |
| `crates/tuitbot-server/src/state.rs` | Added `is_local_first()` convenience method; info log when local_fs source runs in non-Desktop mode |
| `crates/tuitbot-server/src/routes/vault/mod.rs` | Added `deployment_mode`/`privacy_envelope` to `VaultSourcesResponse`; Cloud guard on path; 2 new integration tests |
| `crates/tuitbot-server/src/routes/vault/selections.rs` | Added `privacy_envelope` to `GetSelectionResponse`; 1 new serialization test |
| `crates/tuitbot-server/src/routes/sources.rs` | Added `deployment_mode` to `SourceStatusResponse` |
| `dashboard/src-tauri/src/lib.rs` | Added `get_privacy_envelope` Tauri command |
| `dashboard/src/lib/api/types.ts` | Added `privacy_envelope`, `ghostwriter_local_only` to `DeploymentCapabilities`; `privacy_envelope` to `VaultSelectionResponse`; new `VaultSourcesResponse` interface |
| `dashboard/src/lib/api/client.ts` | Updated `vault.sources()` to use `VaultSourcesResponse` type |
| `dashboard/src/lib/stores/runtime.ts` | Added `privacyEnvelope` and `isLocalFirst` derived stores; updated `DESKTOP_DEFAULTS` |
| `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` | Privacy banner with mode-specific copy; `notice-success` CSS class |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Privacy badge in vault panel header (Local-first / Self-hosted / Cloud) |
| `plugins/obsidian-tuitbot/src/main.ts` | `isLocalTransport()` method and network transport notice |
| `dashboard/tests/helpers/mockApi.ts` | Updated mock capabilities to include new fields |

### Files Created

| File | Purpose |
|---|---|
| `docs/roadmap/obsidian-ghostwriter-edge/local-first-implementation.md` | Decision log (8 decisions) |
| `docs/roadmap/obsidian-ghostwriter-edge/session-08-handoff.md` | This file |

## Decisions Made

See `local-first-implementation.md` for full decision log (8 decisions).

Key decisions:
1. **`is_local_first()` returns true only for Desktop** — Self-host has network boundary, Cloud is provider infrastructure
2. **`privacy_envelope()` returns static labels** — `"local_first"`, `"user_controlled"`, `"provider_controlled"`
3. **Additive API changes with serde defaults** — backward compatible for existing clients
4. **Cloud guard on local path in vault_sources** — defense in depth even though validation already blocks local_fs in Cloud
5. **Positive copy for Self-host** — "Self-hosted — you control the server" rather than "Not local-first"
6. **Non-blocking transport notice in Obsidian plugin** — informs but doesn't prevent sends to remote servers
7. **Tauri `get_privacy_envelope` is a fast sync fallback** — available before API server is ready

## Test Coverage

- **Rust:** 12 new privacy tests + 2 new integration tests (567 → 581 total pass)
- **Frontend:** 686 tests pass (mock helpers updated for new fields)
- **CI:** All gates green (fmt, clippy, svelte-check, vitest, obsidian-tuitbot build)

## What's Next (Session 9+)

- **End-to-end validation:** Verify privacy banners render correctly in each deployment mode via manual testing or E2E tests
- **Release judgment:** Determine if the Ghostwriter Edge feature set is complete enough for initial release
- **Privacy documentation for users:** Add a user-facing explanation of what "local-first" means in the TuitBot context
- **Obsidian plugin settings tab:** Add a settings UI for the plugin so users can change `serverUrl` and see a transport indicator
- **Publish audit trail:** Show provenance on posted tweets in the timeline view (from Session 7 handoff)
