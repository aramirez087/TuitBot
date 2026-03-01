# Session 06 Handoff -- Desktop Compatibility and Migration

## Completed Work

1. **Config regression tests (`config/tests.rs`).**
   Added 6 backward-compatibility tests:
   - `mixed_old_and_new_google_drive_source`: Config with both SA key and
     connection_id parses, round-trips, and validates.
   - `legacy_local_fs_config_unaffected_by_deployment_mode`: Desktop and
     SelfHost modes both validate local_fs sources.
   - `legacy_sa_key_only_config_still_valid`: SA-key-only Drive sources pass
     validation in all three deployment modes.
   - `empty_content_sources_valid`: Empty sources array validates OK.
   - `connection_id_without_sa_key_valid`: connection_id-only Drive validates OK.
   - `google_drive_source_no_auth_warns`: No-auth Drive source passes
     validation (non-blocking warning only).

2. **Validation enhancement (`config/validation.rs`).**
   Added `tracing::warn` for Google Drive sources that have neither
   `connection_id` nor `service_account_key`. This is non-blocking --
   validation still passes, but the warning surfaces the misconfiguration
   at startup.

3. **Upgrade wizard extension (`commands/upgrade/`).**
   Converted `upgrade.rs` to a module directory:
   - `upgrade/mod.rs`: Extended `UpgradeGroup` enum with `DeploymentMode`,
     `Connectors`, and `ContentSources` variants. Updated `all()`,
     `key_paths()`, `display_name()`, `description()`. Extended
     `UpgradeAnswers`, wizard dispatch, non-interactive defaults, and
     TOML patching.
   - `upgrade/content_sources.rs`: New submodule with `patch_deployment_mode()`,
     `patch_connectors()`, `patch_content_sources()`,
     `print_legacy_sa_key_notice()`, `prompt_deployment_mode()`,
     `prompt_connectors()`, and `has_legacy_sa_key()` (test helper).
   - 10 new tests added: `detect_missing_deployment_mode`,
     `detect_missing_connectors`, `detect_missing_content_sources`,
     `patch_deployment_mode_top_level`, `patch_connectors_section`,
     `patch_content_sources_scaffold`, `detect_legacy_sa_key_notice`,
     `patch_all_new_groups_together`, plus updated `detect_missing_from_old_config`
     (now expects 7 groups) and `detect_nothing_missing_from_full_config`
     (now includes all new sections).

4. **Server API tests (`api_tests.rs`).**
   Added 3 backward-compatibility tests:
   - `settings_patch_preserves_legacy_sa_key`: PATCH that doesn't touch
     content_sources preserves SA key on disk.
   - `settings_get_redacts_sa_key_alongside_connection_id`: When both auth
     methods present, SA key is redacted and connection_id returned intact.
   - `settings_init_with_connection_id`: POST `/api/settings/init` with
     connection_id-only Drive source succeeds and round-trips.

5. **config.example.toml.**
   Added deployment mode, content sources, and connectors sections at the
   end of the file. All commented out following existing style. Shows
   `connection_id` as primary (recommended) and `service_account_key` as
   legacy (deprecated).

6. **docs/configuration.md.**
   - Added `[connectors]` to config sections table.
   - Rewrote Google Drive section: "via Linked Account (Recommended)" with
     full field table including `connection_id`, and "via Service Account
     (Legacy)" subsection.
   - Added "Connectors" section documenting `[connectors.google_drive]`
     fields and env var overrides.
   - Added "Upgrading from Service Account to Linked Account" section
     with step-by-step migration instructions.

7. **docs/getting-started.md.**
   Added "Content Sources (Optional)" section before Progressive Enrichment.
   Per-deployment guidance table: Desktop = local folder, SelfHost = Google
   Drive via dashboard, Cloud = Google Drive during onboarding.

8. **docs/lan-mode.md.**
   Added "Content Sources" section before Troubleshooting. Explains that
   LAN mode users should connect Google Drive via the browser dashboard
   (OAuth popup works over LAN) and that local_fs paths resolve on the
   server filesystem.

9. **migration-plan.md (deliverable).**
   Comprehensive scenario matrix covering 6 upgrade paths:
   Desktop local_fs, SelfHost SA-key, SelfHost local path, Cloud SA-key,
   Fresh install, Pre-content-sources install. Includes rollback
   instructions, env var configuration for Docker/CI, and security notes.

10. **session-06-handoff.md (this file).**

## Design Decisions Made

- **DD1 (extend existing UpgradeGroup, no new CLI command):** Per plan,
  extended the existing `UpgradeGroup` enum with 3 new variants instead
  of adding a separate `upgrade-config` subcommand. The `tuitbot update`
  path automatically picks up new groups via `UpgradeGroup::all()`.

- **DD2 (module directory split):** Converted `upgrade.rs` (827 lines) to
  `upgrade/mod.rs` + `upgrade/content_sources.rs` following the
  `commands/init/` pattern. Production code in mod.rs is ~543 lines,
  content_sources.rs is ~225 lines. Tests remain in mod.rs's test module
  at ~706 lines.

- **DD3 (non-interactive defaults are conservative):** Non-interactive
  upgrade adds `deployment_mode = "desktop"` (safe default, matches
  `DeploymentMode::default()`), empty connector scaffold (ready for
  env-var override), and empty `[content_sources]` section. No sources
  are auto-configured.

- **DD4 (legacy SA-key notice is non-blocking):** The `print_legacy_sa_key_notice`
  function prints a tip to stderr but does not error or modify config.
  Users are directed to the dashboard for migration.

- **DD5 (no-auth Drive validation is non-blocking):** A Google Drive source
  with neither `connection_id` nor `service_account_key` passes validation
  with a `tracing::warn`. The Watchtower skips it at runtime. This avoids
  breaking configs during the transition period where a user has added a
  source entry but hasn't completed the OAuth flow yet.

## Open Issues

None blocking Session 07.

**Note:** The `prompt_connectors()` function in `content_sources.rs` uses
`dialoguer::Confirm` and `dialoguer::Input` for interactive credential entry.
These are adequate for the upgrade wizard but not suitable for sensitive
credential entry in a production context. The recommended path is env vars
or the dashboard OAuth flow, not CLI credential prompts.

## Inputs for Session 07

| Input | Location | Notes |
|-------|----------|-------|
| Config regression tests | `crates/tuitbot-core/src/config/tests.rs` | 6 new backward-compat tests |
| Validation warning | `crates/tuitbot-core/src/config/validation.rs` | No-auth Drive warning |
| Upgrade wizard | `crates/tuitbot-cli/src/commands/upgrade/` | 3 new groups |
| Server API tests | `crates/tuitbot-server/tests/api_tests.rs` | 3 new settings tests |
| config.example.toml | `config.example.toml` | New sections |
| Updated docs | `docs/configuration.md`, `docs/getting-started.md`, `docs/lan-mode.md` | |
| Migration plan | `docs/roadmap/.../migration-plan.md` | User-facing guide |
| Frontend components | `dashboard/src/lib/components/onboarding/` | From Session 05 |
| Connector routes | `crates/tuitbot-server/src/routes/connectors.rs` | From Session 04 |

Session 07 deliverables:
- End-to-end validation of fresh installs across all deployment modes
- End-to-end validation of upgraded installs (each scenario in migration-plan.md)
- Integration test for the full OAuth link flow (mock Google OAuth)
