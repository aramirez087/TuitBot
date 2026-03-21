# Privacy and Deployment Matrix

## Deployment Modes

TuitBot operates in three deployment modes, each with different privacy envelopes:

| Mode | Runtime | Data Location | Network Boundary | User Trust Model |
|------|---------|---------------|------------------|------------------|
| **Desktop** | Tauri app with embedded server on `127.0.0.1:3001` | Local SQLite DB in `~/.tuitbot/` | Loopback only (no network exposure) | User controls the machine; all data is local |
| **Self-host** | Standalone server on user's infrastructure | User-controlled server + SQLite | User's network (LAN or VPN) | User controls the server; trusts their own network |
| **Cloud** | Hosted server (future) | Provider-controlled storage | Public internet | User trusts the provider; data leaves their machine |

## Capability Matrix

### Vault Content Access

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| Raw `body_text` in API responses | Never (current behavior preserved) | Never (current behavior preserved) | Never |
| Raw `chunk_text` in API responses | Never via HTTP (read from local DB internally) | Never via HTTP (read from local DB internally) | Never |
| Truncated snippets (120 chars) | Yes | Yes | Yes |
| Heading paths | Yes | Yes | Yes |
| Note titles and paths | Yes | Yes | Yes |
| Tags and front matter keys | Yes | Yes | Yes |

**Rationale**: The current privacy model already prohibits raw body/chunk text in API responses across all modes. The vault routes (`vault.rs`) enforce `SNIPPET_MAX_LEN = 120` universally. This is correct and should not change. The LLM processes full chunk text server-side and returns only generated content.

### Hook/Seed Access

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| Seed text in API responses (max 200 chars) | Yes | Yes | Yes |
| Archetype suggestions | Yes | Yes | Yes |
| Engagement weight | Yes | Yes | Yes |

**Rationale**: Seed text is a short, LLM-generated hook (max 200 chars) — not raw note content. It is a transformation of the source material, analogous to a title or summary. Exposing it in all modes is privacy-safe.

### Obsidian Integration

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| `obsidian://` deep-links (file level) | Yes | No (no Obsidian on server) | No |
| `obsidian://` deep-links (heading level) | Yes (new) | No | No |
| "Open in Obsidian" button | Yes | Hidden | Hidden |
| Local vault path in source response | Yes | Yes (user controls server) | Omitted |

**Implementation**: `CitationChips.svelte` already gates the "Open in Obsidian" button on `isDesktop` prop. The `VaultSourceStatusItem` already conditionally includes `path` only for `local_fs` sources. No new privacy gates needed for deep-links.

### Selection Ingress (Future)

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| `POST /api/vault/send-selection` | Yes (Obsidian plugin → localhost) | Yes (Obsidian plugin → local server) | Yes (with authentication) |
| Selected text storage | Transient (30min TTL), local DB | Transient (30min TTL), local DB | Transient (30min TTL), provider DB |
| Selected text in API response | Echoed back to same authenticated session | Echoed back to same authenticated session | Never returned; processed by LLM only |

**Cloud-mode guard**: In Cloud mode, the selected text is stored transiently for LLM processing only. The `GET` response for a selection session returns the generated content (tweet/thread draft), not the original selected text. This ensures that even if the Cloud provider's DB is compromised, raw vault content is not persistently stored.

### Content Generation

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| LLM processes full chunk text | Yes (local or cloud LLM) | Yes (local or cloud LLM) | Yes (cloud LLM only) |
| Generated content in API response | Yes | Yes | Yes |
| Vault citations in response | Yes | Yes | Yes |
| Provenance links persisted | Yes (local DB) | Yes (local DB) | Yes (provider DB) |

**Note**: In all modes, the LLM sees full chunk text during generation. The privacy boundary is the API response, not the LLM context. Users who configure a local LLM (e.g., Ollama) in Desktop or Self-host mode keep their content entirely on-device.

### Provenance and Citation

| Capability | Desktop | Self-host | Cloud |
|---|---|---|---|
| Full provenance with deep-links | Yes (clickable `obsidian://` links) | Partial (provenance stored, no deep-links) | Partial (provenance stored, no deep-links) |
| Seed provenance (`seed_id`) | Yes | Yes | Yes |
| Chunk provenance (`chunk_id`) | Yes | Yes | Yes |
| Source path in provenance | Yes | Yes | Yes (relative path only, no absolute) |
| Snippet in provenance | Yes (120 chars) | Yes (120 chars) | Yes (120 chars) |

## Privacy Invariants

These invariants hold across all deployment modes and must be enforced in every new endpoint:

1. **No raw `body_text` in HTTP responses**: The `content_nodes.body_text` column is never serialized into any API response.
2. **No raw `chunk_text` in HTTP responses**: The `content_chunks.chunk_text` column is never serialized into any API response. Only truncated snippets (120 chars) are returned.
3. **Account scoping**: Every vault, hook, and provenance query is scoped to the authenticated `AccountContext.account_id`. Cross-account access is impossible at the query level.
4. **Seed text is a transformation**: Hooks/seeds are LLM-generated summaries (max 200 chars), not verbatim excerpts. They are treated as derived content, not raw source material.
5. **Transient selection storage**: Selection ingress data has a 30-minute TTL. No permanent storage of externally-pushed raw text.
6. **Deep-links are Desktop-only UI**: The `isDesktop` check in `CitationChips.svelte` and the scheme whitelist in `open_external_url` (Tauri) ensure that Obsidian deep-links are only rendered and opened in Desktop mode.

## Deployment-Aware API Response Filtering

For new endpoints that return content beyond the current 120-char snippet limit (e.g., selection ingress responses), the server must check `AppState.deployment_mode` and filter accordingly:

```rust
// Pattern for deployment-aware responses
match state.deployment_mode {
    DeploymentMode::Desktop | DeploymentMode::SelfHost => {
        // Return full response (user controls the runtime)
    }
    DeploymentMode::Cloud => {
        // Omit raw text fields; return only generated/derived content
    }
}
```

This pattern is used sparingly — most endpoints already return only derived content (snippets, seeds, generated text) and need no mode-specific filtering.

## Testing Requirements

Each new endpoint must include test cases for:

1. **Account isolation**: Verify that queries return only data for the authenticated account.
2. **Snippet truncation**: Verify that no response field exceeds the documented maximum length.
3. **Empty state**: Verify graceful behavior when no seeds, chunks, or selections exist.
4. **Cloud-mode filtering**: When deployment-aware filtering is used, verify that Cloud-mode responses omit raw content fields.
