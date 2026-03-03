# Settings Flow — X Access Mode Selector

## UI Contract

### Settings Page: X Access Section

The settings section (id: `xapi`) presents X access as a choice between two modes via radio-style cards.

**Section title:** "X Access"
**Section description:** "How Tuitbot connects to X (Twitter)"
**Nav label:** "X Access" (renamed from "X API")

#### Mode Selector Cards

| Card | Config Value | Shown When |
|------|-------------|------------|
| **Official X API** — Full features with API credentials | `provider_backend = ""` (default) or `"x_api"` | Always (desktop, self_host) |
| **Local No-Key Mode** — No credentials needed | `provider_backend = "scraper"` | Desktop and self_host only |

In cloud deployment (`deployment_mode = "cloud"`), the mode selector is hidden entirely. Only the Official X API credential form is shown.

#### Conditional Content

**When Official X API is selected:**
- Info banner with OAuth setup instructions (existing copy)
- Client ID text input
- Client Secret password input with show/hide toggle
- Auth Mode select (Manual / Local Callback)

**When Local No-Key Mode is selected:**
- Info banner explaining the mode: "Run discovery and drafting without API credentials. Read-only by default. Some features like posting, mentions, and analytics are unavailable. Switch to Official X API anytime for full capabilities."
- Advanced section (collapsible):
  - "Allow write operations" checkbox bound to `scraper_allow_mutations`
  - Warning banner: "Posting via Local No-Key Mode carries elevated risk of account restrictions. The Official X API is recommended for posting."

### Onboarding Page: X Access Step

Step 1 (index 1) in the onboarding flow, labeled "X Access" in the progress bar.

#### Mode Selector Cards

Two vertically stacked cards:
- **Official X API** with "Recommended" badge — shows setup guide and credential fields when selected
- **Local No-Key Mode** — shows feature availability matrix when selected

In cloud mode, only the Official X API flow is shown (no selector).

#### Feature Availability Matrix (scraper mode)

Shown when Local No-Key Mode is selected:

| Status | Feature |
|--------|---------|
| Available | Search and discover tweets |
| Available | Score conversations for relevance |
| Available | Draft replies and original content |
| Available | Plan and preview threads |
| Unavailable | Post tweets and replies |
| Unavailable | Mentions and home timeline |

Footer: "You can switch to the Official X API anytime in Settings."

#### Step Advancement

- **Official X API:** `canAdvance()` requires `client_id.trim().length > 0`
- **Local No-Key Mode:** `canAdvance()` returns `true` (no credentials needed)

#### Config Payload (submit)

**x_api mode:**
```json
{
  "x_api": {
    "client_id": "user-entered-id",
    "client_secret": "optional"
  }
}
```

**scraper mode:**
```json
{
  "x_api": {
    "provider_backend": "scraper"
  }
}
```

## Validation Behavior

| Scenario | Result |
|----------|--------|
| `provider_backend = "scraper"`, empty `client_id` | Valid |
| `provider_backend = ""`, empty `client_id` | Error: `MissingField { field: "x_api.client_id" }` |
| `provider_backend = "x_api"`, empty `client_id` | Error: `MissingField { field: "x_api.client_id" }` |
| `provider_backend = "x_api"`, `client_id = "abc"` | Valid |
| `provider_backend = "scraper"`, `deployment_mode = "cloud"` | Error: `InvalidValue { field: "x_api.provider_backend" }` |
| `provider_backend = "scraper"`, `deployment_mode = "desktop"` | Valid |
| `provider_backend = "scraper"`, `deployment_mode = "self_host"` | Valid |
| `provider_backend = "magic"` | Error: `InvalidValue { field: "x_api.provider_backend" }` |

## Environment Variable Support

| Variable | Type | Maps To |
|----------|------|---------|
| `TUITBOT_X_API__PROVIDER_BACKEND` | string | `config.x_api.provider_backend` |
| `TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS` | bool | `config.x_api.scraper_allow_mutations` |

## TypeScript Interface

```typescript
x_api: {
    client_id: string;
    client_secret: string | null;
    provider_backend: string;          // "" | "x_api" | "scraper"
    scraper_allow_mutations: boolean;  // default false
};
```

## Dangerous Changes Detection

Changing `provider_backend` is flagged as a dangerous change in the settings save flow, requiring user confirmation alongside LLM provider and credential changes.

## Settings API Round-Trip

No server code changes were needed. The existing `GET/PATCH /api/settings` endpoints serialize the full `Config` struct via serde. The `XApiConfig` struct already had `provider_backend` and `scraper_allow_mutations` with `#[serde(default)]`, so they flow through the JSON/TOML serialization pipeline automatically.
