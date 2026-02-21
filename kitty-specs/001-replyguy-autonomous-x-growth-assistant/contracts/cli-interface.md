# CLI Interface Contract: ReplyGuy

**Feature**: `001-replyguy-autonomous-x-growth-assistant`
**Date**: 2026-02-21

## Binary Name

`replyguy`

## Global Options

```
replyguy [OPTIONS] <COMMAND>

Options:
  -c, --config <PATH>    Path to config.toml [default: ~/.replyguy/config.toml]
  -v, --verbose          Enable verbose logging (debug level)
  -q, --quiet            Suppress all output except errors
  -h, --help             Print help
  -V, --version          Print version
```

## Commands

### `replyguy run`

Start the autonomous agent. Runs all enabled loops continuously.

```
replyguy run [OPTIONS]

Options:
  --status-interval <SECONDS>   Print periodic status summary (0 = disabled) [default: 0]
```

**Behavior**:
- Loads config, validates credentials, detects X API tier
- Starts enabled loops (mentions, discovery, content, threads)
- Quiet by default (errors only); `--verbose` for full logs
- Optional periodic status summary at configured interval
- Handles SIGTERM/SIGINT for graceful shutdown

**Exit codes**: 0 = clean shutdown, 1 = config/auth error, 2 = runtime error

---

### `replyguy auth`

Authenticate with X API via OAuth 2.0 PKCE.

```
replyguy auth [OPTIONS]

Options:
  --mode <MODE>    Auth mode override [possible values: manual, local_callback]
```

**Behavior**:
- Default mode from `config.toml` (`auth.mode`), overridable via `--mode`
- `manual`: prints authorization URL, waits for pasted code
- `local_callback`: opens browser, listens on `auth.callback_host:auth.callback_port`
- Stores tokens to `~/.replyguy/tokens.json` (chmod 600)
- Prints success/failure status

---

### `replyguy test`

Validate configuration and connectivity.

```
replyguy test
```

**Output**:
```
Configuration:  OK (loaded from ~/.replyguy/config.toml)
Business profile: OK (product_name: "Docklet", 3 keywords, 4 topics)
X API auth:     OK (token valid, expires in 1h 42m)
X API tier:     Basic (search: enabled, mentions: enabled, posting: enabled)
LLM provider:   OK (openai, model: gpt-4o-mini)
Database:       OK (replyguy.db, 142 records, 0.3 MB)
```

**Exit codes**: 0 = all checks pass, 1 = one or more checks failed

---

### `replyguy discover`

Run the discovery loop once.

```
replyguy discover [OPTIONS]

Options:
  --dry-run    Search and score tweets without posting replies
  --limit <N>  Maximum tweets to process [default: 50]
```

**Behavior**:
- Searches X for tweets matching configured keywords
- Scores each tweet and displays results
- Without `--dry-run`: replies to qualifying tweets (score >= threshold)
- With `--dry-run`: displays scores and generated replies without posting

---

### `replyguy mentions`

Check and reply to mentions.

```
replyguy mentions [OPTIONS]

Options:
  --dry-run    Retrieve mentions and generate replies without posting
  --limit <N>  Maximum mentions to process [default: 20]
```

---

### `replyguy post`

Generate and post an original educational tweet.

```
replyguy post [OPTIONS]

Options:
  --dry-run       Generate tweet without posting
  --topic <TOPIC> Override topic (default: random from industry_topics)
```

---

### `replyguy thread`

Generate and post an educational thread.

```
replyguy thread [OPTIONS]

Options:
  --dry-run       Generate thread without posting
  --topic <TOPIC> Override topic (default: random from industry_topics)
  --count <N>     Number of tweets in thread [default: 5-8, auto]
```

---

### `replyguy score`

Score a specific tweet on demand.

```
replyguy score <TWEET_ID>
```

**Output**:
```
Tweet: "Just discovered this amazing tool for..." by @user (1.2K followers)
Score: 82/100
  Keyword relevance:  35/40  (matched: "productivity", "mac")
  Author reach:       12/20  (1,200 followers, log scale)
  Recency:            15/15  (posted 12 minutes ago)
  Engagement rate:    20/25  (4.2% engagement vs 1.5% average)
Verdict: REPLY (threshold: 70)
```

**Exit codes**: 0 = success, 1 = tweet not found or API error

## Configuration File Contract

### Location

Default: `~/.replyguy/config.toml`
Override: `--config <PATH>` or `REPLYGUY_CONFIG` env var

### Structure

```toml
# X API Credentials
[x_api]
client_id = "your-client-id"
client_secret = "your-client-secret"        # optional for public clients

# Authentication
[auth]
mode = "manual"                              # "manual" or "local_callback"
callback_host = "127.0.0.1"                  # for local_callback mode
callback_port = 8080                          # for local_callback mode

# Business Profile
[business]
product_name = "Docklet"
product_description = "A macOS menu bar productivity app"
product_url = "https://docklet.app"
target_audience = "macOS power users, developers, productivity enthusiasts"
product_keywords = ["mac productivity", "menu bar apps", "macOS tips"]
competitor_keywords = ["bartender app", "hidden bar"]
industry_topics = ["SwiftUI", "macOS dev", "automation", "keyboard shortcuts"]

# Scoring Engine
[scoring]
threshold = 70
keyword_relevance_max = 40.0
follower_count_max = 20.0
recency_max = 15.0
engagement_rate_max = 25.0

# Safety Limits
[limits]
max_replies_per_day = 20
max_tweets_per_day = 4
max_threads_per_week = 1
min_action_delay_seconds = 30
max_action_delay_seconds = 120

# Automation Intervals
[intervals]
mentions_check_seconds = 300                  # 5 minutes
discovery_search_seconds = 600                # 10 minutes
content_post_window_seconds = 14400           # 4 hours
thread_interval_seconds = 604800              # 7 days

# LLM Provider
[llm]
provider = "openai"                           # "openai", "anthropic", or "ollama"
api_key = "sk-..."                            # not needed for ollama
model = "gpt-4o-mini"                         # provider-specific model name
base_url = ""                                 # override for custom endpoints

# Data Retention
[storage]
db_path = "~/.replyguy/replyguy.db"
retention_days = 90

# Observability
[logging]
status_interval_seconds = 0                   # 0 = disabled
```

### Environment Variable Overrides

All config values can be overridden with `REPLYGUY_` prefix:

```bash
REPLYGUY_X_API__CLIENT_ID=abc123
REPLYGUY_LLM__API_KEY=sk-...
REPLYGUY_LLM__PROVIDER=anthropic
REPLYGUY_LIMITS__MAX_REPLIES_PER_DAY=10
```

Double underscore (`__`) separates nested keys.

### CLI Flag Precedence

CLI flags > Environment variables > config.toml > built-in defaults
