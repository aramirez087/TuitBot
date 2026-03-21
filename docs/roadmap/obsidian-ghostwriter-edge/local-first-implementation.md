# Local-First Privacy Implementation — Decision Log

## D1: `is_local_first()` method on `DeploymentMode`

**Decision:** A computed method `DeploymentMode::is_local_first() -> bool` returns `true` only for `Desktop`.

**Rationale:** Self-host has local filesystem access but data crosses a network boundary (browser → server), so it cannot claim local-first. Desktop with embedded server on `127.0.0.1:3001` is the only mode where data never leaves the machine.

**Implementation:** `crates/tuitbot-core/src/config/types.rs`

## D2: `privacy_envelope()` method on `DeploymentMode`

**Decision:** Returns `"local_first"`, `"user_controlled"`, or `"provider_controlled"` as a `&'static str`.

**Rationale:** Gives a single source of truth for the privacy label across server responses and UI copy. Avoids hardcoding mode checks in multiple places.

**Implementation:** `crates/tuitbot-core/src/config/types.rs`

## D3: New fields on `DeploymentCapabilities`

**Decision:** Added `privacy_envelope: String` and `ghostwriter_local_only: bool` with `#[serde(default)]` for backward compatibility.

**Rationale:** The frontend already consumes `DeploymentCapabilities` from the runtime status endpoint. Adding these fields lets settings UI and composer show honest privacy copy without extra round-trips. Old clients that don't know about these fields will ignore them (additive API change). Old JSON without these fields will deserialize with defaults (empty string, false).

**Implementation:** `crates/tuitbot-core/src/config/types.rs`

## D4: Privacy envelope in vault route responses

**Decision:** Added `privacy_envelope` to `GetSelectionResponse` and `VaultSourcesResponse`. Also added `deployment_mode` to `VaultSourcesResponse` and `SourceStatusResponse`.

**Rationale:** The frontend needs the privacy context when rendering vault source status and selection review without extra API calls. Cloud mode already gates `selected_text` — now Self-host mode includes `selected_text` with `privacy_envelope: "user_controlled"`.

**Implementation:**
- `crates/tuitbot-server/src/routes/vault/selections.rs`
- `crates/tuitbot-server/src/routes/vault/mod.rs`
- `crates/tuitbot-server/src/routes/sources.rs`

## D5: Cloud guard on local vault path

**Decision:** Added explicit Cloud guard on the `path` field in `vault_sources` handler — `path` is always `None` in Cloud mode, even if a `local_fs` source somehow exists.

**Rationale:** Defense in depth. Cloud mode already rejects `local_fs` sources at validation time, but the vault sources endpoint now also guards against it at read time.

**Implementation:** `crates/tuitbot-server/src/routes/vault/mod.rs`

## D6: Frontend privacy banners

**Decision:** Conditional banners in ContentSourcesSection (settings) and a privacy badge in FromVaultPanel (composer):
- Desktop + local_fs: "Your notes are processed locally — content never leaves this machine." (green success notice)
- Self-host: "Self-hosted — notes stay on your server." (info notice)
- Cloud: "Cloud mode — notes are processed server-side." (info notice)

**Rationale:** Honest copy per the operator constraint: "never market a path as local-first unless the runtime can guarantee it." Self-host copy is positive ("you control the server") rather than negative.

**Implementation:**
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`

## D7: Obsidian plugin transport awareness

**Decision:** Added `isLocalTransport()` method that checks if `serverUrl` hostname is `localhost` or `127.0.0.1`. When false, shows a non-blocking Notice before sending.

**Rationale:** Users may point the plugin at a remote self-hosted server. The plugin should be transparent about where data goes. The notice is non-blocking — it doesn't prevent the send.

**Implementation:** `plugins/obsidian-tuitbot/src/main.ts`

## D8: Tauri `get_privacy_envelope` command

**Decision:** Added a minimal Tauri command that returns `"local_first"` — a fast sync fallback before the API runtime status endpoint is available.

**Rationale:** Desktop always has the same privacy envelope. The frontend can use this before the embedded server is fully ready. Keeps the Tauri surface area minimal as required by the operator rules.

**Implementation:** `dashboard/src-tauri/src/lib.rs`
