# Session 05 Handoff -- Onboarding and Settings UX

## Completed Work

1. **Connector API types and methods (`api.ts`).**
   Added `Connection`, `LinkResponse`, `ConnectorStatusResponse`, and
   `DisconnectResponse` TypeScript interfaces. Added
   `api.connectors.googleDrive` with `link()`, `status()`, and
   `disconnect()` methods. Exported `resolveBaseUrl()` for postMessage
   origin validation.

2. **DESKTOP_DEFAULTS fix (`runtime.ts`).**
   Added missing `preferred_source_default: 'local_fs'` field to the
   fallback defaults used when the server is unreachable.

3. **Connector store (`connectors.ts`, new file).**
   Standalone store managing connection state:
   - `connections` writable store with `loadConnections()` action
   - `activeGoogleDrive` / `expiredGoogleDrive` derived stores
   - `linkingState` / `linkError` for OAuth flow tracking
   - `startLink(force?)` action: opens OAuth popup, listens for
     `postMessage`, validates origin, handles popup blocked/closed/timeout
   - `disconnectConnection(id)` action: calls DELETE, refreshes list

4. **Onboarding store update (`onboarding.ts`).**
   Replaced `service_account_key: string` with
   `connection_id: number | null` in `OnboardingData` interface,
   initial state, and reset function.

5. **DriveConnectCard component (new file).**
   Shared Svelte 5 component with five visual states:
   - Idle: "Connect Google Drive" button
   - Linking: spinner + cancel
   - Connected: green badge + email + disconnect
   - Expired: warning + reconnect
   - Error: message + retry
   - Plus: unconfigured state when `google_drive` capability is false
   Accepts `onconnected(id, email)` and `ondisconnected()` callbacks.
   Used by both SourcesStep and ContentSourcesSection.

6. **SourcesStep.svelte refactor.**
   - Deployment-aware source type initialization from
     `preferred_source_default` (fires once on mount).
   - Desktop: "Obsidian Vault / Notes Folder" first, native Browse
     button, Obsidian-friendly copy.
   - SelfHost: "Google Drive" first, local folder under collapsed
     "Advanced: Local Server Folder" toggle.
   - Cloud: Google Drive only, no source type selector, info notice.
   - DriveConnectCard replaces service-account-key inputs.
   - Folder ID input shown only after connection established.
   - Warning hint when no connection established.
   - Removed all `service_account_key` references.

7. **Onboarding submit update (`+page.svelte`).**
   Google Drive branch now uses `connection_id` instead of
   `service_account_key`. Condition changed from
   `data.folder_id` (required) to `data.connection_id || data.folder_id`
   (either sufficient). `service_account_key` hardcoded to `null`.

8. **ReviewStep.svelte update.**
   Added "Content Source" summary section showing source type, vault path
   (local_fs), connected email (google_drive), folder ID, or "skipped"
   status.

9. **ContentSourcesSection.svelte refactor.**
   - DriveConnectCard replaces service-account-key input.
   - Deployment-aware source type selector (Desktop vs SelfHost vs Cloud).
   - Legacy SA key notice for existing users with `service_account_key`
     but no `connection_id`.
   - Expired connection warning banner.
   - `connection_id` added to `updateSource()` helper.
   - Advanced toggle for local folder in SelfHost mode.
   - `loadConnections()` called on mount.

10. **Documentation.**
    - `frontend-flow.md`: complete frontend flow by mode.
    - `session-05-handoff.md`: this file.

## Design Decisions Made

- **KD1 (standalone connector store):** Kept connector state separate
  from the settings store. The settings store manages TOML config
  lifecycle (load/draft/save) while connector state is an independent
  API surface with its own OAuth flow lifecycle. Coupling them would
  create unnecessary complexity in the save/discard pattern.

- **KD2 (DriveConnectCard as shared component):** Both onboarding and
  settings need identical connect/status/disconnect UX. A single
  component with `onconnected`/`ondisconnected` callbacks avoids
  duplication and ensures consistent behavior.

- **KD3 (source_type init from preferred_source_default):** The
  `$effect` fires once when capabilities load and only overrides the
  default `source_type` if the user hasn't already changed it. This
  prevents resetting an explicit user choice.

- **KD4 (SelfHost local_fs as collapsed advanced):** Per the charter,
  SelfHost users can still use local_fs. It's available under an
  expandable "Advanced" toggle, collapsed by default, to discourage
  raw server-side path entry while keeping it accessible.

- **KD5 (connection_id replaces service_account_key in onboarding):**
  New onboarding flows never write `service_account_key`. Existing
  configs with SA keys continue to work at the Watchtower level
  (priority 2 after connection_id, per Session 04 contract).

- **KD6 (folder_id optional after connection):** The folder ID input
  only appears after a Drive connection is established. Without a
  folder ID, the Watchtower indexes the entire Drive -- a valid
  default for many users.

- **KD7 (postMessage validation):** The OAuth callback listener
  validates `event.origin` against both `resolveBaseUrl()` (for
  Tauri/dev proxy) and `window.location.origin` (for direct access).
  This prevents cross-origin injection while supporting all deployment
  modes.

## Open Issues

None blocking Session 06.

**Note:** The `linkingState` store is module-level writable, not
component-scoped. If multiple DriveConnectCard instances were rendered
simultaneously (not currently possible), they would share state. This
is acceptable because only one OAuth flow can be in progress at a time
(the backend rejects concurrent link attempts via the CONFLICT check).

## Inputs for Session 06

| Input | Location | Notes |
|-------|----------|-------|
| Connector store | `dashboard/src/lib/stores/connectors.ts` | State management for link/disconnect |
| DriveConnectCard | `dashboard/src/lib/components/onboarding/DriveConnectCard.svelte` | Shared UI component |
| Updated SourcesStep | `dashboard/src/lib/components/onboarding/SourcesStep.svelte` | Deployment-aware onboarding |
| Updated ContentSourcesSection | `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` | Settings with connector flow |
| Updated onboarding store | `dashboard/src/lib/stores/onboarding.ts` | connection_id field |
| Updated submit | `dashboard/src/routes/onboarding/+page.svelte` | connection_id in payload |
| Frontend flow docs | `docs/roadmap/.../frontend-flow.md` | Complete UX reference |

Session 06 deliverables:
- CLI `upgrade-config` subcommand for legacy Drive config migration
- Documentation updates (configuration.md, architecture.md)
- Integration tests for legacy config migration path
