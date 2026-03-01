# Configuration

> **Note**: The Desktop App provides a **Visual Settings Editor** under the Settings tab. CLI users can edit files directly or use `tuitbot settings` for interactive editing.

## Config File

Default path: `~/.tuitbot/config.toml`

Custom path:

```bash
tuitbot -c /path/to/config.toml <command>
```

## Quickstart vs Advanced Config

`tuitbot init` generates a minimal config with safe defaults. Only 5 fields are required to start:

| Field | Section | Required by |
|-------|---------|-------------|
| `product_name` | `[business]` | Quickstart |
| `product_keywords` | `[business]` | Quickstart |
| `provider` | `[llm]` | Quickstart |
| `api_key` | `[llm]` | Quickstart (except ollama) |
| `client_id` | `[x_api]` | Quickstart |

Everything else uses defaults. Run `tuitbot init --advanced` for the full 8-step wizard, or enrich progressively after setup.

## Config Sections

| Section | Purpose |
|---------|---------|
| `mode` | Operating mode (`autopilot` or `composer`) |
| `deployment_mode` | Deployment mode (`desktop`, `self_host`, or `cloud`) |
| `[x_api]` | OAuth credentials for X integration |
| `[business]` | Product profile, keywords, voice, persona |
| `[llm]` | LLM provider, model, and API key |
| `[targets]` | Target account monitoring |
| `[scoring]` | 6-signal scoring engine weights and threshold |
| `[limits]` | Rate limits and safety guardrails |
| `[intervals]` | Automation loop timing |
| `[schedule]` | Active hours and timezone |
| `[storage]` | Database path and retention |
| `[logging]` | Log level and status interval |
| `[mcp_policy]` | MCP mutation policy enforcement |
| `[circuit_breaker]` | X API rate-limit protection |
| `[content_sources]` | Content source configuration (local folders, Google Drive) |
| `[connectors]` | OAuth credentials for remote source linking (Google Drive) |

## Progressive Enrichment

The quickstart config is intentionally minimal. Enrich your profile in stages for better results:

```bash
tuitbot settings enrich
```

| Stage | Fields | Impact |
|-------|--------|--------|
| **Voice** | `brand_voice`, `reply_style`, `content_style` | Shapes every LLM-generated reply and tweet |
| **Persona** | `persona_opinions`, `persona_experiences`, `content_pillars` | Makes content authentic and distinctive |
| **Targeting** | `targets.accounts`, `competitor_keywords` | Focuses discovery on high-value conversations |

Check enrichment status with `tuitbot test` — it reports which stages are complete and suggests the next one.

## Operating Mode

| Mode | Behavior |
|------|----------|
| `autopilot` (default) | All automation loops run. Posts automatically when `approval_mode = false`, or queues for review when `approval_mode = true`. |
| `composer` | Autonomous loops disabled. Discovery runs read-only. Approval mode is always on. You write content with AI Assist and the Discovery Feed. |

```toml
mode = "composer"
```

```bash
export TUITBOT_MODE=composer
```

Setting `mode = "composer"` implies `approval_mode = true`. See the [Composer Mode guide](composer-mode.md) for details.

## Deployment Mode

Controls which content source types and features are available based on where the server runs.

| Mode | Context | Local folder | Google Drive | Manual ingest |
|------|---------|-------------|-------------|---------------|
| `desktop` (default) | Tauri native app | Yes | Yes | Yes |
| `self_host` | Docker/VPS browser | Yes | Yes | Yes |
| `cloud` | Managed cloud | **No** | Yes | Yes |

```toml
deployment_mode = "self_host"
```

```bash
export TUITBOT_DEPLOYMENT_MODE=self_host
```

Defaults to `desktop` — existing users need no config changes. The env var accepts `self_host`, `selfhost`, and `self-host` as synonyms for the self-hosted mode.

Deployment mode is orthogonal to operating mode. A cloud user can run in Composer mode; a desktop user can run in Autopilot mode.

In cloud mode, validation rejects `local_fs` content sources on save. Pre-existing `local_fs` entries in the config file are preserved (not deleted) but skipped at runtime with a log warning.

## Safety Defaults

The default config is intentionally conservative:

| Setting | Default | Description |
|---------|---------|-------------|
| `approval_mode` | `true` | All posts queued for human review |
| `max_replies_per_day` | `5` | Hard cap on daily replies |
| `max_tweets_per_day` | `6` | Hard cap on daily tweets |
| `max_replies_per_author_per_day` | `1` | Anti-harassment limit |
| `product_mention_ratio` | `0.2` | Max 20% of replies mention product |
| `banned_phrases` | `["check out", "you should try", ...]` | Blocked salesy phrases |
| Active hours | 8 AM – 10 PM UTC | Sleeps outside these hours |

## Environment Variable Overrides

Override any config value using the `TUITBOT_` prefix with `__` (double underscore) as the section separator:

```bash
export TUITBOT_X_API__CLIENT_ID=your_client_id
export TUITBOT_X_API__CLIENT_SECRET=your_secret
export TUITBOT_LLM__API_KEY=sk-your-key
export TUITBOT_LLM__PROVIDER=openai
export TUITBOT_MODE=composer
export TUITBOT_DEPLOYMENT_MODE=self_host
```

**Precedence:** CLI flags > environment variables > `config.toml` > built-in defaults.

This is particularly useful for Docker and CI environments where you don't want secrets in config files.

## MCP Mutation Policy

The `[mcp_policy]` section controls how MCP mutation tools are gated before execution:

```toml
[mcp_policy]
enforce_for_mutations = true
require_approval_for = ["post_tweet", "reply_to_tweet", "follow_user", "like_tweet"]
blocked_tools = []
dry_run_mutations = false
max_mutations_per_hour = 20
```

| Field | Default | Description |
|-------|---------|-------------|
| `enforce_for_mutations` | `true` | Master switch for policy checks |
| `require_approval_for` | `[...]` | Tools routed to the approval queue |
| `blocked_tools` | `[]` | Tools completely blocked |
| `dry_run_mutations` | `false` | Return dry-run responses without executing |
| `max_mutations_per_hour` | `20` | Aggregate hourly rate limit for all MCP mutations |

**Evaluation order** (safest wins): disabled? > blocked? > dry_run? > rate limited? > requires approval? > allow.

**Composer mode**: All mutations require approval regardless of `require_approval_for`.

**Admin profile**: When running `tuitbot mcp serve --profile admin`, 27 additional tools are available: 16 Ads API tools, 4 Compliance tools, 3 Stream Rules tools, and 4 universal request tools (`x_get`, `x_post`, `x_put`, `x_delete`). All typed enterprise mutations (Ads, Compliance, Stream Rules) are policy-gated with approval routing, rate limiting, and dry-run mode. Universal request mutations are constrained by the host allowlist (`api.x.com`, `upload.x.com`, `upload.twitter.com`, `ads-api.x.com`), SSRF guards, and header blocklist — but are **not** currently subject to the MCP policy engine. See the [MCP Reference](mcp-reference.md) for profile details.

## Enterprise API Access

Some MCP tools require additional X API access beyond a standard developer account. These tools will return `x_forbidden` if your credentials lack the required authorization.

### Direct Message Access

DM tools (8 tools, available from API-readonly profile and above) require DM-scoped OAuth tokens:

- **Scopes needed:** `dm.read`, `dm.write` (write only for mutations), `users.read`
- **How to enable:** Request DM scopes during `tuitbot auth`. If you authenticated before DM tools were available, re-run `tuitbot auth` to request updated scopes.
- **Verification:** `tuitbot test` reports whether DM scopes are granted.

### Ads API Access

Ads tools (16 tools, Admin profile only) require a separate Ads API developer account:

- **Prerequisite:** Apply for [X Ads API access](https://developer.x.com/en/docs/twitter-ads-api/getting-started) through the developer portal.
- **Host:** All Ads tools route to `ads-api.x.com` (included in the host allowlist).
- **Financial risk:** Ads mutations can incur ad spend. TuitBot does not enforce budget caps — manage spend limits in the X Ads dashboard.
- **Scopes needed:** Ads-specific OAuth scopes as required by the X Ads API.

### Compliance & Stream Rules Access

Compliance and Stream Rules tools (7 tools, Admin profile only) require elevated API access:

- **Prerequisite:** Enterprise or Academic Research API tier.
- **Scopes needed:** `compliance.write` (Compliance tools), `tweet.read` (Stream Rules tools), `usage.read` (tweet usage).
- **Note:** The filtered stream *connection* endpoint is not supported (SSE does not fit MCP's request/response model). Only stream rule CRUD is available.

### Verifying Enterprise Access

After configuring enterprise access, verify with:

```bash
tuitbot test                              # reports scope status
tuitbot mcp serve --profile admin         # starts with all 139 tools
tuitbot mcp manifest --profile admin      # lists all available tools
```

If enterprise tools return `x_forbidden`, check your developer account permissions in the X developer portal.

## Validation

After any config change, verify with:

```bash
tuitbot test                    # full diagnostic check
tuitbot settings --show         # read-only config view
```

## Content Sources

Configure external content sources for the Watchtower ingest pipeline.
Content is ingested as notes, processed into draft seeds, and used to
enrich AI-generated content via Winning DNA retrieval.

> **Deployment mode note:** `local_fs` sources require `local_folder` capability, available only in Desktop and SelfHost modes. Cloud mode supports `google_drive` and manual ingest only. See [Deployment Mode](#deployment-mode) above.

### Local Folder Source

```toml
[[content_sources.sources]]
source_type = "local_fs"
path = "~/Obsidian/my-vault"
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = true
```

| Field | Default | Description |
|-------|---------|-------------|
| `source_type` | `"local_fs"` | Source type identifier |
| `path` | — | Path to content directory (supports `~` expansion) |
| `watch` | `true` | Watch for real-time file changes |
| `file_patterns` | `["*.md", "*.txt"]` | Glob patterns for files to ingest |
| `loop_back_enabled` | `true` | Write tweet metadata back to source file front-matter |

### Google Drive via Linked Account (Recommended)

The recommended way to connect Google Drive is through the dashboard's OAuth
flow. This creates a linked account connection that is simpler to set up and
does not require a service account key file.

1. Configure your GCP OAuth credentials in `[connectors.google_drive]` (see [Connectors](#connectors) below).
2. Open the dashboard: **Settings > Content Sources > Connect Google Drive**.
3. Complete the Google OAuth consent flow in the popup.
4. The dashboard writes a `connection_id` to your config automatically.

```toml
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "1abc..."
connection_id = 1
watch = true
file_patterns = ["*.md", "*.txt"]
poll_interval_seconds = 300
loop_back_enabled = false
```

| Field | Default | Description |
|-------|---------|-------------|
| `source_type` | -- | Must be `"google_drive"` |
| `folder_id` | -- | Google Drive folder ID to monitor (optional; omit to index entire Drive) |
| `connection_id` | -- | Linked account ID from the dashboard OAuth flow |
| `service_account_key` | -- | **(Legacy)** Path to Google service account JSON key file |
| `poll_interval_seconds` | `300` | Seconds between Drive API polls |
| `loop_back_enabled` | `false` | Not supported for Drive (read-only) |

**Auth precedence:** If both `connection_id` and `service_account_key` are present, `connection_id` takes precedence. A validation warning is logged.

### Google Drive via Service Account (Legacy)

Existing configs that use `service_account_key` continue to work. No action is required. If you want to migrate to the linked account flow, connect via the dashboard and the `connection_id` field will be added automatically. See [Upgrading from Service Account to Linked Account](#upgrading-from-service-account-to-linked-account) below.

```toml
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "1abc..."
service_account_key = "~/.tuitbot/service-account.json"
watch = true
file_patterns = ["*.md", "*.txt"]
poll_interval_seconds = 300
loop_back_enabled = false
```

### Operational Limits

| Parameter | Value | Notes |
|-----------|-------|-------|
| Max file size | Unbounded (content truncated at 2000 chars for seed extraction) | Full content stored in DB |
| File types | `.md`, `.txt` only | Configurable via `file_patterns` |
| Dedup | SHA-256 content hash per (source, path) | Unchanged content is skipped |
| Seed generation | 5 nodes per batch, every 5 minutes | Low-priority background worker |
| RAG context | Max 5 ancestors or 5 cold-start seeds, 2000 chars | Injected into LLM prompts |

### Manual Ingest API

Content can also be submitted directly via the HTTP API:

```bash
curl -X POST http://localhost:3001/api/ingest \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "inline_nodes": [{
      "relative_path": "idea.md",
      "body_text": "# My Idea\nContent here...",
      "title": "My Idea"
    }]
  }'
```

## Connectors

OAuth application credentials for linking remote content sources. Currently
supports Google Drive; the design is extensible to future connectors.

```toml
[connectors.google_drive]
client_id = "YOUR_GCP_OAUTH_CLIENT_ID.apps.googleusercontent.com"
client_secret = "GOCSPX-YOUR_CLIENT_SECRET"
redirect_uri = "http://localhost:3001/api/connectors/google-drive/callback"
```

| Field | Default | Description |
|-------|---------|-------------|
| `client_id` | -- | GCP OAuth 2.0 Client ID |
| `client_secret` | -- | GCP OAuth 2.0 Client Secret |
| `redirect_uri` | `http://localhost:3001/api/connectors/google-drive/callback` | OAuth callback URL |

Get credentials from [Google Cloud Console > APIs & Services > Credentials](https://console.cloud.google.com/apis/credentials). Create an "OAuth 2.0 Client ID" of type "Web application" and add the redirect URI above.

Environment variable overrides:

```bash
export TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_ID=...
export TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_SECRET=...
export TUITBOT_CONNECTORS__GOOGLE_DRIVE__REDIRECT_URI=...
```

The connector config is required before the OAuth link flow works in the
dashboard. Desktop apps may bundle default credentials via environment
variables.

## Upgrading from Service Account to Linked Account

If your existing config uses `service_account_key` for Google Drive:

1. Your current setup continues to work -- no immediate action needed.
2. To upgrade, add `[connectors.google_drive]` credentials to your config (or set the env vars).
3. Open the dashboard: **Settings > Content Sources > Connect Google Drive**.
4. Complete the OAuth flow. The dashboard adds `connection_id` to the source.
5. Once `connection_id` is set, it takes precedence over `service_account_key`.
6. You can optionally remove `service_account_key` after verifying the linked account works.

Running `tuitbot update` also detects missing connector config and offers to
add a scaffold section.

## Production Guidance

- Keep secrets out of shell history — use environment variables or a secrets manager.
- Store config on a persistent volume in Docker deployments.
- Back up the SQLite database before major upgrades: `tuitbot backup`.
- Start with `approval_mode = true` until you trust the AI's output quality.
- Use Composer mode for new accounts until confident in content tone.
