# Deployment Capability Matrix

## Deployment Modes

Tuitbot supports three deployment modes that determine which content source types and features are available:

| Mode | Context | How it's set |
|------|---------|-------------|
| **Desktop** (default) | Tauri native app on user's machine | Default — no config needed |
| **SelfHost** | Docker/VPS, accessed via browser | `TUITBOT_DEPLOYMENT_MODE=self_host` |
| **Cloud** | Managed cloud at app.tuitbot.dev | `TUITBOT_DEPLOYMENT_MODE=cloud` |

## Capability Matrix

| Capability | Desktop | SelfHost | Cloud | Notes |
|-----------|---------|----------|-------|-------|
| `local_folder` | Yes | Yes | **No** | Server-side filesystem read access |
| `manual_local_path` | Yes | Yes | **No** | User can type a path in the browser |
| `google_drive` | Yes | Yes | Yes | Remote source via Drive API v3 |
| `inline_ingest` | Yes | Yes | Yes | `POST /api/ingest` direct content |
| `file_picker_native` | Yes | No | No | Tauri native file dialog |

## Source Type Mapping

Each source type maps to one or more capabilities:

| Source Type | Required Capability | Available Modes |
|------------|-------------------|-----------------|
| `local_fs` | `local_folder` | Desktop, SelfHost |
| `google_drive` | `google_drive` | Desktop, SelfHost, Cloud |
| `manual` | `inline_ingest` | Desktop, SelfHost, Cloud |

## Design Principles

### Capabilities are derived, not configured

The capability set is a pure function of `DeploymentMode`. Operators do not toggle individual capabilities — they select a mode and the capabilities follow. This prevents impossible states (e.g., cloud with local_folder enabled).

### DeploymentMode is orthogonal to OperatingMode

A cloud user can run in Composer mode. A desktop user can run in Autopilot mode. The two axes are independent:

- **DeploymentMode** (Desktop/SelfHost/Cloud) controls *where the server runs* and what it can access.
- **OperatingMode** (Autopilot/Composer) controls *how autonomous* the agent is.

### Desktop is the default

Existing users who never set `deployment_mode` get `Desktop` with full capabilities. No config migration is needed for desktop or self-host users.

## Pre-existing Config Migration

### Desktop and SelfHost

All existing configs remain valid. `local_fs` sources with filesystem paths work exactly as before. No action required.

### Cloud

If a cloud server loads a config containing `local_fs` sources:

1. **On startup**: The server logs a warning for each incompatible source but does not crash. The Watchtower loop skips those sources silently.
2. **On save (PATCH /api/settings)**: Validation rejects the save with a clear error pointing to the incompatible source type and deployment mode.
3. **Config preservation**: The `local_fs` entries remain in the TOML file — they are not deleted. This allows easy migration back to self-host if the user moves their deployment.

## API Surface

The capability payload is exposed via `GET /api/runtime/status`:

```json
{
  "running": true,
  "task_count": 4,
  "deployment_mode": "desktop",
  "capabilities": {
    "local_folder": true,
    "manual_local_path": true,
    "google_drive": true,
    "inline_ingest": true,
    "file_picker_native": true
  }
}
```

The frontend reads this on app load and uses it to:
- Hide/show source type options in the content sources settings
- Disable the native file picker button in non-desktop modes
- Show explanatory messages when capabilities are unavailable
