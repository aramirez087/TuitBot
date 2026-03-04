# No-X-API-Key Local Mode — Charter

## Problem Statement

The X Developer Portal is a significant barrier to Tuitbot adoption for non-technical users (founders, indie hackers, marketers). The free API tier is heavily restricted, the approval process is opaque, and many users abandon onboarding when asked for OAuth 2.0 credentials they don't have.

Tuitbot's core value — discovering relevant conversations, drafting authentic replies, and planning content — does not inherently require the user to have their own API key. Public tweet data is accessible without authentication. The API key is only strictly necessary for authenticated operations (posting, mentions, home timeline, bookmarks).

By offering a "no key" mode that uses a scraper transport for public data, Tuitbot can deliver immediate value on first launch while clearly guiding users toward the official API path for full capabilities.

## User Promise

**Run Tuitbot's full discovery, drafting, and content planning workflows on desktop or LAN without an X API key.**

Specifically, users in no-key mode can:

- Search for tweets matching their product keywords (public data)
- View public profiles and tweet histories
- Score discovered tweets for reply-worthiness
- Generate draft replies and original content via their configured LLM
- Plan and preview threads
- Use the approval queue to stage content for later posting

What they cannot do without upgrading to the official API:

- Post tweets, replies, or threads to X
- Like, retweet, or follow accounts on X
- See their own mentions or home timeline
- Upload media
- View analytics that require authenticated profile data

## Non-Goals

1. **No cloud support for scraper mode.** The scraper transport is inherently fragile and creates operational risk at scale. It is allowed only in `desktop` and `self_host` deployment modes. `deployment_mode = "cloud"` rejects `provider_backend = "scraper"` at validation time.

2. **No guarantee of X API parity.** The scraper transport returns public data with `data_confidence = "medium"`. Results may differ from the official API in completeness, format, or freshness. The product does not mask these differences — it surfaces them clearly.

3. **No scraping of auth-gated data.** Mentions, home timeline, bookmarks, and the user's own profile (`get_me`) are authentication-gated by X. Tuitbot will not attempt to scrape these, even if technically possible, because doing so would require user credentials in a transport with elevated risk.

4. **No embedded browser login flow.** The scraper mode does not implement or simulate browser-based X login. It operates solely on publicly accessible data.

5. **No write operations by default.** Scraper mode ships with `scraper_allow_mutations = false`. Users must explicitly opt in to mutations, and even then, writes are gated by transport health checks.

## Deployment Boundaries

| Deployment Mode | Scraper Backend Allowed | Rationale |
|----------------|------------------------|-----------|
| `desktop` | Yes | Single-user, local machine. User accepts risk. |
| `self_host` | Yes | LAN/VPS, operator-controlled. Operator accepts risk. |
| `cloud` | No | Multi-tenant, managed service. Scraper fragility and risk are unacceptable. Validation rejects this combination. |

## UI Framing

The settings surface presents X access as a choice between two modes:

| UI Label | Config Value | Description |
|----------|-------------|-------------|
| **Official X API** | `provider_backend = "x_api"` | Full capabilities. Requires Client ID from the X Developer Portal. Recommended for posting and automation. |
| **Local No-Key Mode** | `provider_backend = "scraper"` | Discovery and drafting without API credentials. Read-only by default. Some features unavailable. |

The UI does not use the word "scraper" in user-facing copy. It uses "Local No-Key Mode" or "No-Key Mode" to communicate the value (no credentials needed) rather than the mechanism (scraping).

## Feature Availability Matrix

| Feature | Official X API | Local No-Key Mode |
|---------|---------------|-------------------|
| Discovery (search tweets) | Full | Supported (public data) |
| Profile lookup | Full | Supported (public data) |
| Get tweet by ID | Full | Supported (public data) |
| User tweets timeline | Full | Supported (public data) |
| Scoring & ranking | Full | Full (operates on discovered data) |
| LLM draft generation | Full | Full (operates on discovered data) |
| Thread planning | Full | Full (operates on discovered data) |
| Approval queue (staging) | Full | Full |
| Post tweet | Full | Gated by `scraper_allow_mutations` |
| Reply to tweet | Full | Gated by `scraper_allow_mutations` |
| Like / retweet / follow | Full | Gated by `scraper_allow_mutations` |
| Mentions | Full | Unavailable (auth-gated) |
| Home timeline | Full | Unavailable (auth-gated) |
| Bookmarks | Full | Unavailable (auth-gated) |
| Own profile (`get_me`) | Full | Unavailable (auth-gated) |
| Media upload | Full | Unavailable |
| Analytics (own-profile) | Full | Degraded (no authenticated stats) |

## Risk Inventory

### R1: Scraper transport breaks without notice

- **Likelihood:** Medium
- **Impact:** High — users see stale or missing data
- **Mitigation:** Circuit breaker on consecutive failures. `data_confidence = "medium"` communicated in UI and API responses. Clear error messages directing users to the official API path when the scraper is unhealthy.

### R2: Users enable mutations and get their account restricted

- **Likelihood:** Low
- **Impact:** High — account damage, user trust loss
- **Mitigation:** `scraper_allow_mutations = false` by default. Mutation toggle is behind an "Advanced" section with explicit warning copy: *"Posting via Local No-Key Mode carries elevated risk of account restrictions. The official X API is recommended for posting."* Users must affirmatively enable mutations.

### R3: Cloud deployment with scraper bypasses safety

- **Likelihood:** Low (requires explicit misconfiguration)
- **Impact:** High — fragile transport in production
- **Mitigation:** Config validation rejects `provider_backend = "scraper"` when `deployment_mode = "cloud"`. Implemented in Session 02.

### R4: Users mistake no-key mode for full-featured

- **Likelihood:** Medium
- **Impact:** Medium — frustration, support burden
- **Mitigation:** Onboarding clearly shows what's available vs. unavailable. Dashboard surfaces "upgrade to Official API" prompts when users hit unavailable features. The mode selector explains tradeoffs at point of choice.

### R5: Data quality divergence causes bad scoring

- **Likelihood:** Low
- **Impact:** Medium — poor reply targeting
- **Mitigation:** The scoring engine operates on the same data structures regardless of transport. Public data from the scraper transport is structurally identical to API data. The risk is incomplete data (e.g., missing engagement counts), not malformed data. Scoring gracefully handles missing fields with zero-value defaults.

## Success Criteria

1. A new user can complete onboarding, run discovery, and see scored tweets without entering any X API credentials.
2. The settings UI clearly communicates what each mode offers and what it lacks.
3. Attempting a write operation in no-key mode (with mutations disabled) produces an actionable error that guides the user toward either enabling mutations or configuring the official API.
4. No code path allows `provider_backend = "scraper"` in `deployment_mode = "cloud"`.
5. Existing users with `provider_backend = "x_api"` (the default) experience zero behavior changes.
