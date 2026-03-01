# Backend Contract — Fresh-Install Claim Bootstrap

**Status:** Implemented (Session 02)
**Date:** 2026-02-28

---

## Modified Endpoints

### `GET /api/settings/status`

**Auth:** Not required (exempt).

**Response (200):**

```json
{
  "configured": false,
  "claimed": false,
  "deployment_mode": "desktop",
  "capabilities": {
    "local_folder": true,
    "manual_local_path": true,
    "file_picker_native": true,
    "google_drive": false
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `configured` | `bool` | Whether `config.toml` exists |
| `claimed` | `bool` | Whether `passphrase_hash` file exists (new) |
| `deployment_mode` | `string` | One of `desktop`, `self_host`, `cloud` |
| `capabilities` | `object` | Feature flags for the deployment mode |

The `claimed` field checks the filesystem (`data_dir/passphrase_hash`) on every call. This is a new additive field — existing callers that don't read it are unaffected.

---

### `POST /api/settings/init`

**Auth:** Not required (exempt).

#### Without claim (backward compatible)

**Request:**

```json
{
  "business": {
    "product_name": "MyProduct",
    "product_keywords": ["saas", "productivity"]
  },
  "llm": { "provider": "openai", "api_key": "sk-..." }
}
```

**Response (200):**

```json
{
  "status": "created",
  "config": { ... }
}
```

No `Set-Cookie` header. No `csrf_token` field. Identical to pre-Session-02 behavior.

#### With claim (new)

**Request:**

```json
{
  "business": {
    "product_name": "MyProduct",
    "product_keywords": ["saas", "productivity"]
  },
  "llm": { "provider": "openai", "api_key": "sk-..." },
  "claim": {
    "passphrase": "word1 word2 word3 word4"
  }
}
```

The `claim` object is extracted and removed before JSON-to-TOML conversion. It is not a config field.

**Response (200):**

```json
{
  "status": "created",
  "config": { ... },
  "csrf_token": "a1b2c3d4..."
}
```

**Response headers:**

```
Set-Cookie: tuitbot_session=<token>; HttpOnly; SameSite=Strict; Path=/; Max-Age=604800
```

The session cookie and CSRF token are identical in format to those returned by `POST /api/auth/login`. The frontend should store the CSRF token the same way it does after login.

#### Error responses

| Status | Condition | Body |
|--------|-----------|------|
| 400 | `claim.passphrase` shorter than 8 characters | `{"error": "passphrase must be at least 8 characters"}` |
| 400 | Invalid config values | `{"error": "invalid config: ..."}` |
| 409 | Config already exists | `{"error": "configuration already exists; use PATCH /api/settings to update"}` |
| 409 | `claim` present but `passphrase_hash` already exists | `{"error": "instance already claimed"}` |

#### Validation failure (200, not an error)

If the config fails validation but the body is structurally valid:

```json
{
  "status": "validation_failed",
  "errors": [
    { "field": "business.product_name", "message": "this field is required" }
  ]
}
```

---

## Passphrase Validation Rules

- **Minimum length:** 8 characters
- **Maximum length:** None
- **Format:** No format requirement (multi-word or single string both accepted)
- **Recommended:** 4-word EFF passphrase (~20+ characters). The frontend will suggest this format.

---

## Startup Behavior Change

| `--host` value | Passphrase behavior |
|----------------|-------------------|
| `0.0.0.0` | Auto-generate if none exists, print to terminal (unchanged) |
| `127.0.0.1` (default) | Load from disk if exists, skip generation if not. The browser claim flow handles creation. |
| `--reset-passphrase` | Always reset regardless of host (unchanged) |

---

## Security Properties (Preserved)

| Property | Status |
|----------|--------|
| Passphrase plaintext never written to disk | Preserved |
| Passphrase plaintext never logged | Preserved |
| Hash file has 0600 permissions on Unix | Preserved |
| Session cookies are HttpOnly | Preserved |
| CSRF required for mutating cookie-auth requests | Preserved |
| Claim is one-shot (409 on repeat) | New — enforced by `passphrase_hash` existence check |
| Bearer token auth unaffected | Preserved |

---

## Core Functions Added

| Function | Module | Purpose |
|----------|--------|---------|
| `is_claimed(data_dir)` | `tuitbot_core::auth::passphrase` | Check if `passphrase_hash` file exists |
| `create_passphrase_hash(data_dir, plaintext)` | `tuitbot_core::auth::passphrase` | Create hash from user-provided passphrase (claim operation) |

| Error variant | Module | Purpose |
|---------------|--------|---------|
| `AuthError::AlreadyClaimed` | `tuitbot_core::auth::error` | Maps to HTTP 409 when claim is attempted on already-claimed instance |
