# Feature Specification: ReplyGuy — Autonomous X Growth Assistant

**Feature Branch**: `001-replyguy-autonomous-x-growth-assistant`
**Created**: 2026-02-21
**Status**: Draft
**Mission**: software-dev

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Configure Business Profile and Authenticate (Priority: P1)

A founder who just launched a product wants to set up ReplyGuy so the agent understands their business, audience, and niche. They create a `config.toml` with their product details (name, description, URL, target audience, keywords, competitor keywords, and industry topics), add their X API credentials, and configure their preferred LLM provider. They then authenticate with X using the CLI.

**Why this priority**: Without configuration and authentication, no other feature can function. This is the foundational setup that enables all automation.

**Independent Test**: Can be fully tested by creating a config file and running `replyguy auth` to verify X API authentication succeeds, then running `replyguy test` to validate the full configuration.

**Acceptance Scenarios**:

1. **Given** a user has no existing configuration, **When** they create a `config.toml` with required fields and run `replyguy auth`, **Then** the CLI walks them through OAuth 2.0 PKCE authentication and stores tokens locally.
2. **Given** a user has configured `auth.mode = "manual"`, **When** they run `replyguy auth`, **Then** the CLI displays an authorization URL, waits for the user to paste the authorization code, and completes authentication without requiring a browser or GUI.
3. **Given** a user has configured `auth.mode = "local_callback"`, **When** they run `replyguy auth`, **Then** the CLI opens the browser, starts a local HTTP server on the configured port, and captures the callback automatically.
4. **Given** a user has a complete config file, **When** they run `replyguy test`, **Then** the CLI validates all credentials (X API, LLM provider), confirms connectivity, and reports the configuration status.

---

### User Story 2 — Discover and Reply to Relevant Tweets (Priority: P1)

A founder wants ReplyGuy to find tweets related to their niche and engage early with valuable, human-sounding replies. The discovery loop searches X using the configured keywords, scores each tweet for relevance and viral potential, and only replies to tweets meeting the score threshold. Replies are contextual, mention the product naturally when appropriate, and never exceed three sentences.

**Why this priority**: Discovery and reply is the core value proposition — it's the primary way users grow their audience organically.

**Independent Test**: Can be tested by running `replyguy discover --dry-run` to verify tweet discovery and scoring, then running the discovery loop to confirm replies are posted only to qualifying tweets.

**Acceptance Scenarios**:

1. **Given** the agent is running with configured keywords, **When** the discovery loop executes, **Then** it retrieves tweets matching the configured keywords from X API v2 search.
2. **Given** a list of discovered tweets, **When** each tweet is evaluated, **Then** the scoring engine assigns a relevance+engagement score between 0 and 100.
3. **Given** a tweet scores at or above the configured threshold (default: 70), **When** the agent decides to reply, **Then** it generates a reply of three sentences or fewer using the configured LLM provider, incorporating business context where natural.
4. **Given** a tweet scores below the threshold, **When** the agent evaluates it, **Then** it skips the tweet without replying.
5. **Given** the agent has already replied to a tweet, **When** it encounters the same tweet again, **Then** it skips it (no duplicate replies).
6. **Given** the daily reply limit has been reached, **When** a new high-scoring tweet is found, **Then** the agent queues or skips it until the next day.

---

### User Story 3 — Monitor and Reply to Mentions (Priority: P2)

A founder wants ReplyGuy to automatically respond to people mentioning their X handle. The mentions loop checks for new mentions, generates helpful replies, and posts them. This ensures the founder never misses an engagement opportunity.

**Why this priority**: Mentions represent direct engagement opportunities where someone is already talking about or to the user. Responding promptly builds relationships and trust.

**Independent Test**: Can be tested by running `replyguy mentions --dry-run` to verify mention retrieval and reply generation without posting.

**Acceptance Scenarios**:

1. **Given** the agent is running, **When** new mentions are detected, **Then** the agent retrieves them via the X API v2 mentions endpoint.
2. **Given** a new mention is retrieved, **When** the agent processes it, **Then** it generates a helpful, contextual reply using the LLM and business profile context.
3. **Given** the agent has already replied to a mention, **When** it encounters the same mention again, **Then** it skips it.
4. **Given** the daily reply limit has been reached, **When** a new mention arrives, **Then** the agent defers the reply until limits reset.

---

### User Story 4 — Post Original Educational Content (Priority: P2)

A founder wants ReplyGuy to keep their X account active with original educational content related to their niche. When no tweet has been posted within a configurable time window, the agent generates and posts an educational tweet — covering topics like productivity tips, developer workflows, industry insights, or lessons learned building a product.

**Why this priority**: Consistent original content establishes the founder as a thought leader and keeps the account active between engagement opportunities.

**Independent Test**: Can be tested by running `replyguy post --dry-run` to verify tweet generation without posting.

**Acceptance Scenarios**:

1. **Given** no original tweet has been posted within the configured time window, **When** the content loop triggers, **Then** the agent generates an educational tweet related to the user's industry topics and niche.
2. **Given** an original tweet is generated, **When** it is posted, **Then** it is recorded in the local database with a timestamp to prevent premature re-triggering.
3. **Given** the daily tweet limit has been reached, **When** the content loop triggers, **Then** the agent defers posting until limits reset.

---

### User Story 5 — Publish Weekly Educational Threads (Priority: P3)

A founder wants ReplyGuy to publish in-depth educational threads once per configurable period (default: weekly). Each thread consists of 5 to 8 connected tweets derived from the configured industry topics — for example, "10 macOS productivity tricks developers love" or "Lessons learned building a SaaS as a solo founder."

**Why this priority**: Threads drive significantly more engagement and follower growth than individual tweets, but are less frequent and depend on the content generation infrastructure from P2.

**Independent Test**: Can be tested by running `replyguy thread --dry-run` to verify thread generation and structure without posting.

**Acceptance Scenarios**:

1. **Given** the configured thread interval has elapsed since the last thread, **When** the thread loop triggers, **Then** the agent generates a 5-to-8-tweet educational thread on a topic derived from industry_topics.
2. **Given** a thread is generated, **When** it is posted, **Then** each tweet is posted as a reply to the previous one, forming a proper X thread.
3. **Given** the weekly thread limit has been reached, **When** the thread loop triggers, **Then** the agent defers until the next period.

---

### User Story 6 — Run as a Continuous Background Agent (Priority: P1)

A founder wants to start ReplyGuy once and have it run autonomously in the background, executing all four loops (mentions, discovery, content, threads) on their configured schedules. The agent must handle rate limits, network errors, and API failures gracefully without crashing.

**Why this priority**: The core promise of ReplyGuy is autonomous operation. Without a reliable runtime that orchestrates all loops, the tool is just a collection of one-off commands.

**Independent Test**: Can be tested by running `replyguy run` and observing that all loops execute on schedule, handle errors gracefully, and respect rate limits.

**Acceptance Scenarios**:

1. **Given** a fully configured ReplyGuy, **When** the user runs `replyguy run`, **Then** the agent starts all four automation loops and runs continuously.
2. **Given** the agent is running, **When** a network error or API rate limit is encountered, **Then** the agent logs the error, backs off appropriately, and retries without crashing.
3. **Given** the agent is running, **When** safety limits are approached, **Then** the agent slows down or pauses the relevant loop until limits reset.
4. **Given** the agent is running, **When** the user sends a termination signal (Ctrl+C / SIGTERM), **Then** the agent shuts down gracefully, completing in-progress actions and saving state.

---

### User Story 7 — Score Individual Tweets On Demand (Priority: P3)

A founder wants to manually evaluate a specific tweet's viral potential and relevance before deciding to engage. They run `replyguy score <tweet_id>` and receive a detailed breakdown of the score.

**Why this priority**: This is a diagnostic/utility feature that helps users understand and tune the scoring engine, but is not required for autonomous operation.

**Independent Test**: Can be tested by running `replyguy score <tweet_id>` with a valid tweet ID and verifying the score breakdown is displayed.

**Acceptance Scenarios**:

1. **Given** a valid tweet ID, **When** the user runs `replyguy score <tweet_id>`, **Then** the CLI fetches the tweet, runs the scoring engine, and displays the score (0-100) with a breakdown of contributing factors.
2. **Given** an invalid or inaccessible tweet ID, **When** the user runs `replyguy score <tweet_id>`, **Then** the CLI displays a clear error message.

---

### Edge Cases

- What happens when X API rate limits are exhausted mid-loop? The agent must back off with exponential delay and resume when limits reset, without losing state.
- What happens when the configured LLM provider is unreachable? The agent must log the failure, skip content generation for that cycle, and retry on the next loop iteration.
- What happens when the SQLite database is corrupted or inaccessible? The agent must detect the error at startup and report it clearly rather than silently failing.
- What happens when the user's X account gets temporarily suspended or restricted? The agent must detect API error codes indicating account issues and pause all loops with a clear warning.
- What happens when a generated reply or tweet exceeds X's character limit? The agent must validate content length before posting and regenerate if necessary.
- What happens when the config.toml is missing required fields? The `replyguy test` command must identify and report all missing or invalid fields before the agent starts.
- What happens when two loops try to post at the same time? The agent must serialize posting actions to avoid race conditions and respect rate limits.
- What happens when the OAuth tokens expire? The agent must detect expired tokens, attempt to refresh them, and prompt the user to re-authenticate if refresh fails.

## Clarifications

### Session 2026-02-21

- Q: What does the user see while the agent runs? → A: Quiet by default (errors only) with `--verbose` for full structured logs, plus an optional periodic status summary (configurable interval, off by default).
- Q: How are OAuth tokens and API keys stored locally? → A: Default plaintext file with strict filesystem permissions (`~/.replyguy/tokens.json`, chmod 600) for maximum compatibility (servers, Raspberry Pi, containers). Optional OS keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service) when available.
- Q: What X API tier is required? → A: Design for Basic tier but degrade gracefully on Free. Free tier: mentions + posting only (discovery loop disabled). Basic tier: full functionality (search + discovery). API capability auto-detected at startup so the agent adapts to the user's tier.
- Q: Does the SQLite database grow unbounded? → A: Configurable retention with automatic cleanup. Default 90 days, configurable in config.toml. Periodic background job prunes old discovered tweets, action logs, and metrics records. Never deletes rate limit counters or recent replies/posts needed for deduplication.
- Q: What is explicitly out of scope for v1? → A: No web UI, no multi-account support, no DM automation, no analytics dashboard, no scheduled content calendar, no reply-to-reply conversations (only direct mentions and top-level discovered tweets).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST authenticate with X API v2 using OAuth 2.0 PKCE, supporting both manual code-entry (default) and local browser callback modes, selectable via `auth.mode` in `config.toml`.
- **FR-002**: System MUST load all behavior configuration from a `config.toml` file, including X API credentials, business profile, discovery keywords, scoring weights, rate limits, automation intervals, delays, and LLM settings.
- **FR-003**: System MUST provide a business profile configuration section containing: `product_name`, `product_description`, `product_url`, `target_audience`, `product_keywords`, `competitor_keywords`, and `industry_topics`.
- **FR-004**: System MUST discover tweets by searching X API v2 using the configured `product_keywords` and `competitor_keywords`.
- **FR-005**: System MUST score each discovered tweet on a 0-to-100 scale evaluating relevance to the business profile and engagement/viral potential.
- **FR-006**: System MUST only generate and post replies to tweets scoring at or above the configurable threshold (default: 70).
- **FR-007**: System MUST generate human-sounding replies of three sentences or fewer using the configured LLM provider, incorporating business context naturally.
- **FR-008**: System MUST support multiple LLM providers: OpenAI, Anthropic (Claude), and Ollama (local models), selectable via configuration.
- **FR-009**: System MUST monitor the authenticated user's mentions via X API v2 and generate helpful replies.
- **FR-010**: System MUST post original educational tweets related to the user's niche when no tweet has been posted within the configured time window.
- **FR-011**: System MUST generate and post educational threads of 5 to 8 tweets at a configurable interval (default: weekly), with topics derived from `industry_topics`.
- **FR-012**: System MUST persist all state to a local SQLite database, including: tweets discovered, replies sent, tweets posted, rate limit status, and timestamps of last actions.
- **FR-013**: System MUST enforce configurable safety limits: maximum replies per day (default: 20), maximum tweets per day (default: 4), and maximum threads per week (default: 1).
- **FR-014**: System MUST enforce minimum delays between actions, with randomized jitter to appear natural.
- **FR-015**: System MUST never reply to the same tweet twice and MUST avoid repeating reply phrasing across recent replies.
- **FR-016**: System MUST run as a continuous background agent orchestrating all four loops (mentions, discovery, content, threads) concurrently.
- **FR-017**: System MUST handle network errors, API rate limits, and provider failures gracefully with exponential backoff and retry, without crashing.
- **FR-018**: System MUST expose the following CLI commands: `run`, `auth`, `test`, `discover --dry-run`, `mentions --dry-run`, `post --dry-run`, `thread --dry-run`, and `score <tweet_id>`.
- **FR-019**: System MUST validate the entire configuration and report all issues when the user runs `replyguy test`.
- **FR-020**: System MUST support configurable callback host and port for the local browser callback auth mode via `auth.callback_host` and `auth.callback_port` in `config.toml`.
- **FR-021**: System MUST provide sensible defaults for all configurable values so that users only need to supply credentials and a business profile to get started.
- **FR-022**: System MUST shut down gracefully on termination signals, completing in-progress actions and saving state.
- **FR-023**: System MUST default to quiet output (errors only) during `replyguy run`, with a `--verbose` flag that enables full structured logging (debug/info/warn/error levels).
- **FR-024**: System MUST support an optional periodic status summary (e.g., replies sent, tweets scored, next cycle time) at a configurable interval, disabled by default.
- **FR-025**: System MUST store OAuth tokens and API keys in a local file (`~/.replyguy/tokens.json`) with strict filesystem permissions (mode 600) by default.
- **FR-026**: System SHOULD support optional OS keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service) as an alternative credential storage backend, selectable via configuration.
- **FR-027**: System MUST auto-detect the user's X API tier at startup and adapt available features accordingly: Free tier enables mentions monitoring, posting, and threads only; Basic tier (or higher) enables full functionality including keyword search and the discovery loop.
- **FR-028**: System MUST inform the user at startup which API tier was detected and which loops are enabled or disabled as a result.
- **FR-029**: System MUST automatically prune old discovered tweets, action logs, and metrics records beyond a configurable retention period (default: 90 days).
- **FR-030**: System MUST NOT delete rate limit counters or recent reply/post records required for deduplication during cleanup.

### Key Entities

- **Business Profile**: Represents the user's product and audience context — product name, description, URL, target audience, keywords, competitor keywords, and industry topics. Used by all content generation and discovery logic.
- **Discovered Tweet**: A tweet retrieved from X search matching configured keywords. Attributes include tweet ID, author, content, metrics, discovery timestamp, and assigned score.
- **Reply**: A response generated and posted by the agent to a discovered tweet or mention. Attributes include the target tweet ID, reply content, timestamp, and generation metadata.
- **Original Tweet**: An educational tweet generated and posted by the agent. Attributes include content, topic, timestamp, and generation metadata.
- **Thread**: A series of 5-to-8 connected tweets posted as a thread. Attributes include topic, individual tweet contents, posting timestamps, and thread root ID.
- **Rate Limit State**: Tracks current usage against configured limits (replies/day, tweets/day, threads/week) and timestamps for limit resets.
- **Action Log**: A record of every action taken by the agent (discovery, reply, post, thread) with timestamps, used to enforce delays, prevent duplicates, and provide audit history.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user can go from zero to a fully configured and authenticated ReplyGuy in under 10 minutes.
- **SC-002**: The discovery loop identifies relevant tweets and assigns accurate scores within 30 seconds of each search cycle.
- **SC-003**: All generated replies are three sentences or fewer, contextually relevant to the target tweet, and incorporate the user's business context naturally.
- **SC-004**: The agent operates continuously for 24+ hours without crashing, even when encountering network errors, rate limits, or provider outages.
- **SC-005**: Memory usage remains below 50 MB and CPU usage is near zero when the agent is idle between loop cycles.
- **SC-006**: The agent never exceeds configured safety limits (replies/day, tweets/day, threads/week) under any circumstances.
- **SC-007**: The agent never replies to the same tweet twice, verified across restarts via persistent state.
- **SC-008**: All dry-run commands (`discover`, `mentions`, `post`, `thread`) execute and display results without making any changes to the user's X account.
- **SC-009**: The agent compiles and runs on macOS, Linux, and Windows from a single codebase.
- **SC-010**: All configuration is driven by `config.toml` with documented defaults, requiring only credentials and business profile as mandatory inputs.

## Out of Scope (v1)

- **Web UI** — ReplyGuy is CLI-only; no browser-based dashboard or management interface.
- **Multi-account support** — The agent operates a single X account per configuration.
- **DM automation** — No reading, sending, or responding to direct messages.
- **Analytics dashboard** — No built-in reporting, charts, or engagement analytics.
- **Scheduled content calendar** — No queue management or pre-planned posting schedule; content is generated on-demand by the automation loops.
- **Reply-to-reply conversations** — The agent only engages with direct mentions and top-level discovered tweets. It does not follow up on replies to its own replies or participate in threaded conversations.

## Assumptions

- Users have an approved X API v2 developer account. Basic tier ($100/mo) is recommended for full functionality; Free tier is supported with reduced capabilities (no discovery loop).
- Users have an API key for at least one supported LLM provider (OpenAI, Anthropic, or a running Ollama instance).
- The X API v2 rate limits are sufficient for the default safety limits (20 replies/day, 4 tweets/day).
- SQLite is adequate for local persistence needs; no multi-user or networked database access is required.
- The agent runs on a machine with persistent internet connectivity (e.g., a personal computer, VPS, or Raspberry Pi).
