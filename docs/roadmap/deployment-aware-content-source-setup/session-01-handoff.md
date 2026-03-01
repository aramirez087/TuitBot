# Session 01 Handoff -- Charter and Source Strategy

## Completed Work

1. **Full audit of 12 repository anchor files.** Every file listed in the session
   instructions was read and analyzed:
   - `docs/architecture.md` -- three-layer model, deployment modes, content source pipeline
   - `docs/configuration.md` -- config sections, deployment mode docs, content source TOML examples
   - `docs/lan-mode.md` -- dual auth model, passphrase management, LAN access
   - `crates/tuitbot-core/src/config/types.rs` -- `DeploymentMode`, `DeploymentCapabilities`, `ContentSourceEntry`
   - `crates/tuitbot-core/src/source/google_drive.rs` -- service-account JWT auth, custom RSA, Drive API v3 polling
   - `crates/tuitbot-core/src/automation/watchtower/mod.rs` -- local watcher + remote polling, ingest pipeline
   - `crates/tuitbot-server/src/routes/settings.rs` -- config status, init, get, patch, validate, factory reset
   - `dashboard/src/lib/stores/runtime.ts` -- capabilities loading, desktop fallback
   - `dashboard/src/lib/stores/onboarding.ts` -- hardcoded `source_type: 'local_fs'` default
   - `dashboard/src/lib/components/onboarding/SourcesStep.svelte` -- source type select, vault path input, Drive fields
   - `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` -- post-setup source editing
   - `dashboard/src/routes/onboarding/+page.svelte` -- step flow, config assembly, submit logic

2. **Charter produced** (`charter.md`) covering:
   - Current state analysis across all three layers (config, backend, frontend)
   - Six identified problems with impact assessment
   - Six design decisions (D1-D6) with rationale and mode-specific UX tables
   - Connector model with `connections` table schema
   - Google Drive OAuth 2.0 + PKCE flow design
   - Legacy migration boundaries with zero-breakage policy
   - Security rules for credential handling
   - Implementation breakdown across sessions 02-07 with named files per session
   - Risk/mitigation matrix

## Open Issues

None blocking Session 02.

**Design note:** The GCP OAuth client ID and secret needed for user-account
Google Drive linking are not yet discussed in terms of where they live in the
config. Session 02 should decide whether they go in `config.toml` under a new
`[connectors.google_drive]` section, in environment variables, or both. The
charter's precedence rule (CLI flags > env vars > TOML > defaults) applies.

## Inputs for Session 02

| Input | Location | Notes |
|-------|----------|-------|
| Charter | `docs/roadmap/deployment-aware-content-source-setup/charter.md` | Design decisions D1-D6, connector model, migration boundaries |
| Config types | `crates/tuitbot-core/src/config/types.rs` | Add `preferred_source_default` to `DeploymentCapabilities`, `connection_id` to `ContentSourceEntry` |
| Config validation | `crates/tuitbot-core/src/config/mod.rs` | Update to accept both legacy and new shapes |
| Config tests | `crates/tuitbot-core/src/config/tests.rs` | Add tests for legacy deserialization and capability reporting |
| Migrations dir | `migrations/` | Create `NNNN_create_connections_table.sql` |
| Storage module | `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Add CRUD for `connections` table |
| Settings routes | `crates/tuitbot-server/src/routes/settings.rs` | Redact `service_account_key` in GET response |
| Frontend API types | `dashboard/src/lib/api.ts` | Add `preferred_source_default` to `DeploymentCapabilities` type |

Session 02 deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/source-connection-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-02-handoff.md`
