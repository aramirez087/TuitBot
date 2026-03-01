# Frontend Flow -- Deployment-Aware Content Source Setup

This document describes the frontend UX flow for content source setup
across deployment modes after Session 05.

---

## 1. Architecture

### New Files
- `dashboard/src/lib/stores/connectors.ts` -- Connector state management
  (connections list, linking state, link/disconnect actions, postMessage
  listener for OAuth callback).
- `dashboard/src/lib/components/onboarding/DriveConnectCard.svelte` --
  Shared component for Google Drive connect/status/disconnect. Used by
  both onboarding SourcesStep and settings ContentSourcesSection.

### Modified Files
- `dashboard/src/lib/api.ts` -- Added `Connection`, `LinkResponse`,
  `ConnectorStatusResponse`, `DisconnectResponse` types. Added
  `api.connectors.googleDrive` methods (link, status, disconnect).
  Exported `resolveBaseUrl()` for origin validation.
- `dashboard/src/lib/stores/runtime.ts` -- Fixed `DESKTOP_DEFAULTS`
  missing `preferred_source_default` field.
- `dashboard/src/lib/stores/onboarding.ts` -- Replaced
  `service_account_key: string` with `connection_id: number | null`.
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte` --
  Deployment-aware source type default, DriveConnectCard integration,
  Obsidian-friendly copy, collapsed advanced section for SelfHost.
- `dashboard/src/routes/onboarding/+page.svelte` -- Updated submit to
  use `connection_id` instead of `service_account_key`.
- `dashboard/src/lib/components/onboarding/ReviewStep.svelte` -- Added
  content source summary section.
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` --
  DriveConnectCard, legacy SA key notice, expired connection warning,
  deployment-aware source type selector.

---

## 2. Deployment Mode Behavior

### Desktop (default: local_fs)

**Onboarding:**
- Source type selector shows "Obsidian Vault / Notes Folder" first,
  "Google Drive" second.
- Local folder shows Browse button (native Tauri picker) + path input.
- Copy: "Select your Obsidian vault or Markdown notes folder."
- Switching to Google Drive shows DriveConnectCard.

**Settings:**
- Same selector order. Browse button when native picker available.
- Obsidian-friendly labels.

### Self-Hosted (default: google_drive)

**Onboarding:**
- Source type selector shows "Google Drive" first, "Local Server Folder"
  second.
- Google Drive selected by default via `preferred_source_default`.
- DriveConnectCard shown immediately.
- Local folder available but under "Advanced: Local Server Folder"
  collapsed toggle when switched to.

**Settings:**
- Same deployment-aware selector order.
- Advanced toggle for local folder path in SelfHost mode.

### Cloud (google_drive only)

**Onboarding:**
- No source type selector shown.
- Informational notice: "Local folder sources are not available in cloud
  deployments."
- DriveConnectCard shown directly.

**Settings:**
- Same cloud-only behavior.

---

## 3. Google Drive Connection Flow

### States (DriveConnectCard)

1. **Idle / No connection**: "Connect Google Drive" button. Clicking
   calls `startLink()` which opens OAuth popup.

2. **Linking in progress**: Spinner + "Waiting for Google
   authorization..." with Cancel button.

3. **Connected (active)**: Green badge with account email. "Disconnect"
   button.

4. **Expired**: Warning badge with email + "Reconnect" button (calls
   `startLink(true)` with force flag).

5. **Error**: Error message + "Try Again" button.

6. **Unconfigured**: Shown when `capabilities.google_drive` is false.
   "Google Drive connector not configured" with setup hint.

### OAuth Popup Flow

1. Frontend calls `POST /api/connectors/google-drive/link`.
2. Server returns `{ authorization_url, state }`.
3. Frontend opens popup via `window.open()`.
4. User completes Google OAuth consent.
5. Google redirects to callback endpoint.
6. Server exchanges code for tokens, stores encrypted refresh token,
   creates connection row.
7. Callback page posts `window.opener.postMessage({ type:
   "connector_linked", id: <connection_id> })`.
8. Frontend listener validates origin and message type, updates store.

### Security

- `postMessage` origin validated against `resolveBaseUrl()` and
  `window.location.origin`.
- Popup blocked detection: if `window.open()` returns null, shows error
  message.
- Popup closed detection: interval check every 1s resets linking state.
- 5-minute timeout prevents indefinite waiting.
- `Connection` type never includes `encrypted_credentials`.
- `service_account_key` returned as `[redacted]` from `GET /settings`.
- DriveConnectCard only displays `account_email` and `status`.

---

## 4. Onboarding Data Model

### Before (Session 04)
```typescript
interface OnboardingData {
    // ...
    service_account_key: string;
    folder_id: string;
    // ...
}
```

### After (Session 05)
```typescript
interface OnboardingData {
    // ...
    connection_id: number | null;
    folder_id: string;
    // ...
}
```

### Submit Payload

When `source_type === 'google_drive'`:
```json
{
    "content_sources": {
        "sources": [{
            "source_type": "google_drive",
            "path": null,
            "folder_id": "<optional>",
            "service_account_key": null,
            "connection_id": 1,
            "watch": true,
            "file_patterns": ["*.md", "*.txt"],
            "loop_back_enabled": false,
            "poll_interval_seconds": 300
        }]
    }
}
```

`connection_id` is the primary field; `service_account_key` is always
null in new onboarding flows.

---

## 5. Settings: Legacy Migration

Existing users with `service_account_key` in their config see:
- A read-only notice: "Using legacy service account key. Connect a
  Google account above to upgrade to the linked account flow."
- The DriveConnectCard to connect a new linked account.
- Once connected, `connection_id` replaces `service_account_key` on save.
- The legacy SA key path continues to work silently at the Watchtower
  level (priority 2 after `connection_id`).

---

## 6. Review Step

The onboarding review step now shows a "Content Source" summary section:
- Source type label
- For local_fs: vault path
- For google_drive: connected account email or "Not connected"
- For google_drive: folder ID if set
- If nothing configured: "(skipped -- configure later in Settings)"
