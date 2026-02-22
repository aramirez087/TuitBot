# ReplyGuy

**Your autonomous X (Twitter) growth assistant.** ReplyGuy runs in the background and grows your X account on autopilot — finding relevant conversations, replying with genuinely helpful content, posting educational tweets, and publishing weekly threads. It's like having a social media manager who never sleeps.

Built for **founders, indie hackers, and solo makers** who'd rather build their product than spend hours on X every day.

---

## What Does ReplyGuy Actually Do?

ReplyGuy runs four automated loops, 24/7, while you focus on building:

### 1. Finds Conversations That Matter
ReplyGuy searches X for tweets matching your product's keywords (e.g., "mac productivity", "clipboard manager"). It scores every tweet using a smart scoring system to find the ones most worth replying to — prioritizing tweets with high engagement, relevant keywords, recent posts, and authors with real followings.

### 2. Replies With Genuinely Helpful Content
When it finds a high-scoring tweet, ReplyGuy uses AI to write a natural, helpful reply in **your brand voice**. It sounds like you, not a bot. It only mentions your product when it's genuinely relevant to the conversation — no spam, no "check out my app!" energy.

### 3. Posts Educational Tweets
Every few hours, ReplyGuy posts original educational tweets about your industry topics. This builds your authority and keeps your profile active without you lifting a finger.

### 4. Publishes Weekly Threads
Once a week, ReplyGuy creates a 5-8 tweet thread on one of your topics — the kind of deep-dive content that gets bookmarks and retweets.

### 5. Monitors Your Mentions
When someone @-mentions you, ReplyGuy generates a thoughtful reply automatically. You never leave someone hanging.

### Built-In Safety
ReplyGuy is designed to keep your account safe:
- **Daily caps**: Limits on replies (15/day), tweets (3/day), and threads (1/week) to avoid looking spammy
- **Random delays**: 45-180 seconds of random delay between actions to look natural
- **Deduplication**: Never replies to the same tweet twice, and detects when replies sound too similar to recent ones
- **Graceful shutdown**: Press Ctrl+C anytime — it stops cleanly within 30 seconds

---

## Prerequisites

You'll need three things before getting started:

| What | Where to get it | Cost |
|------|-----------------|------|
| **Rust 1.75+** | [rustup.rs](https://rustup.rs/) | Free |
| **X API developer account** | [developer.x.com](https://developer.x.com) | Free tier works (with limitations — see [X API tiers](#x-api-tiers) below) |
| **An AI provider** | [OpenAI](https://platform.openai.com/), [Anthropic](https://console.anthropic.com/), or [Ollama](https://ollama.ai/) (local, free) | Varies (Ollama is free) |

---

## Installation

```bash
# Option 1: Install the binary globally (recommended)
cargo install --path crates/replyguy-cli

# Option 2: Build without installing
cargo build --release
# Binary will be at: target/release/replyguy
```

After installation, verify it works:

```bash
replyguy --help
```

---

## Getting Started (Step by Step)

The entire setup takes about 5 minutes. ReplyGuy has a built-in wizard that walks you through everything.

### Step 1: Set Up Your X Developer App

Before running the wizard, you need to create an app on X's developer portal. This is what gives ReplyGuy permission to post on your behalf.

1. Go to [developer.x.com/en/portal/dashboard](https://developer.x.com/en/portal/dashboard)
2. Create a new project and app (or use an existing one)
3. Under your app's **Settings** tab, click **"Set up"** under **"User authentication settings"**
4. Configure these settings **exactly**:

| Setting | What to select |
|---------|---------------|
| **App permissions** | **Read and write** |
| **Type of App** | **Native App** (also called "public client") |
| **Callback URI / Redirect URL** | `http://127.0.0.1:8080/callback` |
| **Website URL** | Your product's URL (e.g., `https://yoursite.com`) |

> **The callback URI must be exactly `http://127.0.0.1:8080/callback`** — no `https`, no `localhost`, no trailing slash. This is the most common source of auth errors.

5. Save, then go to **Keys and tokens** and copy your **OAuth 2.0 Client ID**. You'll need it in the next step.

### Step 2: Run the Setup Wizard

```bash
replyguy init
```

The wizard walks you through 4 sections. Defaults are shown in `[brackets]` — just press Enter to accept them.

**Section 1 — X API Credentials:**
- Paste your OAuth 2.0 Client ID
- If you chose "Confidential client" instead of "Native App", you'll also need your Client Secret (most people don't need this)

**Section 2 — Business Profile:**
- Your product name (e.g., "Docklet")
- A one-line description (e.g., "Floating command strip for macOS")
- Your product URL (optional, press Enter to skip)
- Your target audience (e.g., "Mac power users and developers")
- Discovery keywords, comma-separated (e.g., `macos productivity, mac menu bar, clipboard manager`) — these are the terms ReplyGuy searches for on X
- Content topics, comma-separated (e.g., `Mac productivity tips, macOS workflows`) — these are what ReplyGuy tweets about

**Section 3 — Brand Voice (optional, but recommended):**
- Brand voice/personality — describe how you want to sound (e.g., "Friendly technical expert. Casual tone, occasionally witty.")
- Reply style — how should replies feel? (e.g., "Lead with genuine help. Only mention our product if directly relevant.")
- Content style — how should tweets/threads sound? (e.g., "Share practical tips with real examples.")

> **Tip**: If you skip these, ReplyGuy uses sensible defaults ("be conversational and helpful, not salesy"). You can always edit them later in `~/.replyguy/config.toml`.

**Section 4 — AI Provider:**
- Choose your provider: `openai` (default), `anthropic`, or `ollama`
- Paste your API key (not needed for Ollama)
- Choose a model (defaults: `gpt-4o-mini` for OpenAI, `claude-sonnet-4-6` for Anthropic, `llama3.2` for Ollama)

After the wizard, it will show you a summary and ask to save. Then it seamlessly offers to continue with authentication and testing.

### Step 3: Authenticate With X

If you didn't do this during the wizard, run:

```bash
replyguy auth
```

This opens your browser to X's authorization page. Click **"Authorize app"**, and ReplyGuy will automatically catch the callback. You'll see:

```
Callback server listening on 127.0.0.1:8080
Authenticated as @yourusername. Tokens saved to ~/.replyguy/tokens.json
```

> **Running on a remote server?** Use manual mode: `replyguy auth --mode manual`. It will print a URL to open in any browser and ask you to paste back the authorization code.

Your tokens are saved securely and refresh automatically — you only need to do this once.

### Step 4: Validate Everything

```bash
replyguy test
```

This checks all five components and tells you if anything is wrong:

```
Configuration:     OK (loaded from /home/user/.replyguy/config.toml)
Business profile:  OK (product_name: "Docklet", 3 keywords, 4 topics)
X API auth:        OK (token valid, expires in 1h 45m)
LLM provider:      OK (openai, model: gpt-4o-mini)
Database:          OK (will be created at ~/.replyguy/replyguy.db)
```

If any check fails, it tells you exactly what's wrong and how to fix it.

### Step 5: Start the Agent

```bash
replyguy run
```

That's it. ReplyGuy is now running. You'll see a startup banner:

```
ReplyGuy v0.1.0
Tier: Free | Loops: mentions, content, threads
Press Ctrl+C to stop.
```

Leave it running in the background. Press **Ctrl+C** anytime to stop it gracefully.

> **Tip**: Run it in a terminal multiplexer like `tmux` or `screen` so it keeps running after you close your terminal, or see [Running as a Background Service](#running-as-a-background-service) below.

---

## One-Off Commands

You don't have to run the full agent. ReplyGuy also has commands for doing things one at a time:

| Command | What it does |
|---------|-------------|
| `replyguy discover` | Search for tweets matching your keywords, score them, and reply to the best ones — once |
| `replyguy discover --dry-run` | Same as above, but **don't actually post** — just show what it would do |
| `replyguy mentions` | Check your @-mentions and reply to new ones — once |
| `replyguy post` | Generate and post a single educational tweet |
| `replyguy post --topic "Mac shortcuts"` | Post a tweet on a specific topic |
| `replyguy thread` | Generate and post a thread |
| `replyguy thread --topic "5 macOS tips"` | Post a thread on a specific topic |
| `replyguy score <tweet_id>` | Score a specific tweet to see if ReplyGuy would reply to it |

The `--dry-run` flag is great for testing. It shows you exactly what ReplyGuy would do without actually posting anything.

---

## Configuration Reference

Your config lives at `~/.replyguy/config.toml`. The wizard creates it for you, but you can edit it anytime with any text editor. Here's every section explained in plain English.

### X API Credentials

```toml
[x_api]
client_id = "your-client-id-here"       # Required. From developer.x.com
# client_secret = "your-secret-here"    # Only if you chose "Confidential client"
```

### Authentication

```toml
[auth]
mode = "local_callback"     # "local_callback" (auto) or "manual" (paste code yourself)
callback_host = "127.0.0.1" # Don't change unless you know what you're doing
callback_port = 8080        # Port for the auth callback server
```

### Business Profile

This is the most important section — it tells ReplyGuy what your product is, who to talk to, and what to talk about.

```toml
[business]
product_name = "Docklet"
product_description = "Floating command strip for macOS"
product_url = "https://getdocklet.app"      # Optional
target_audience = "Mac power users and developers"

# Keywords that ReplyGuy searches for on X.
# Be specific! "productivity" is too broad. "macos productivity app" is better.
product_keywords = ["macos productivity", "mac menu bar", "mac clipboard manager"]

# Optional: competitor or alternative keywords
competitor_keywords = ["notchnook alternative", "bartender mac"]

# Topics for original tweets and threads.
# ReplyGuy rotates through these so content stays fresh.
industry_topics = [
    "Mac productivity tips",
    "macOS power user workflows",
    "Building native Swift apps",
]

# Optional but recommended — makes the bot sound like YOU:
brand_voice = "Friendly technical expert. Casual, occasionally witty."
reply_style = "Lead with genuine help. Only mention product if relevant."
content_style = "Share practical tips. Prefer 'here's what I learned' framing."
```

> **Keyword tips**:
> - Use 2-3 word phrases, not single words ("mac clipboard manager" not "clipboard")
> - Include competitor names if you want to engage in those conversations
> - Multi-word keywords are weighted 2x in scoring, so they're more effective

### Scoring Engine

Controls how ReplyGuy decides which tweets are worth replying to. Each tweet gets a score from 0-100 based on four signals:

```toml
[scoring]
threshold = 70  # Minimum score to trigger a reply (0-100)

# How much each signal contributes (must add up to 100):
keyword_relevance_max = 40.0  # How well the tweet matches your keywords
follower_count_max = 20.0     # Author's follower count (logarithmic scale)
recency_max = 15.0            # How recently the tweet was posted
engagement_rate_max = 25.0    # Likes + retweets + replies relative to followers
```

**What the scores mean in practice:**
- **90-100**: Perfect match — your exact keywords, engaged author, very recent
- **70-89**: Good match — worth replying to (this is the default threshold)
- **50-69**: Marginal — might be relevant but not a sure thing
- **Below 50**: Probably not relevant

> **Tip**: Start with the default threshold of 70. If ReplyGuy is replying too much, raise it to 80. If it's not finding enough tweets, lower it to 60.

### Safety Limits

These caps prevent ReplyGuy from posting too aggressively and getting your account flagged. The defaults are conservative on purpose.

```toml
[limits]
max_replies_per_day = 15        # Max replies per 24 hours
max_tweets_per_day = 3          # Max original tweets per 24 hours
max_threads_per_week = 1        # Max threads per 7 days
min_action_delay_seconds = 45   # Minimum random delay between any actions
max_action_delay_seconds = 180  # Maximum random delay between any actions
```

ReplyGuy adds a random delay between `min` and `max` seconds before each action. This makes activity look natural rather than bot-like.

> **Tip**: Don't raise the limits above their defaults unless you're sure your account can handle it. Getting flagged by X is much worse than posting less.

### Automation Intervals

How often each automation loop runs. Shorter intervals = more active, but uses more API quota.

```toml
[intervals]
mentions_check_seconds = 300        # Check mentions every 5 minutes
discovery_search_seconds = 900      # Search for tweets every 15 minutes
content_post_window_seconds = 18000 # Post a tweet at most every 5 hours
thread_interval_seconds = 604800    # Post a thread at most every 7 days
```

| Interval | Default | Human-readable |
|----------|---------|----------------|
| Mention checks | 300 | Every 5 minutes |
| Discovery searches | 900 | Every 15 minutes |
| Content tweets | 18,000 | Every 5 hours |
| Thread publishing | 604,800 | Every 7 days |

### AI Provider

```toml
[llm]
provider = "openai"          # "openai", "anthropic", or "ollama"
api_key = "sk-..."           # Required for openai/anthropic, not needed for ollama
model = "gpt-4o-mini"        # Which model to use
# base_url = "http://localhost:11434/v1"  # Only needed for Ollama or custom endpoints
```

**Provider comparison:**

| Provider | Model | Cost | Quality | Setup |
|----------|-------|------|---------|-------|
| **OpenAI** | `gpt-4o-mini` | ~$0.15/1M tokens | Great | API key from [platform.openai.com](https://platform.openai.com/) |
| **Anthropic** | `claude-sonnet-4-6` | ~$3/1M tokens | Excellent | API key from [console.anthropic.com](https://console.anthropic.com/) |
| **Ollama** | `llama3.2` | Free (runs locally) | Good | Install [Ollama](https://ollama.ai/), then `ollama pull llama3.2` |

> **On a budget?** Use `gpt-4o-mini` (very cheap) or Ollama (completely free, runs on your machine). ReplyGuy uses very few tokens — a few cents per day with OpenAI.

### Storage

```toml
[storage]
db_path = "~/.replyguy/replyguy.db"  # Where the SQLite database lives
retention_days = 90                   # Delete old data after this many days (0 = keep forever)
```

ReplyGuy stores everything in a local SQLite database — discovered tweets, replies sent, original tweets, threads, and rate limit counters. Old data is cleaned up automatically based on `retention_days`.

### Logging

```toml
[logging]
status_interval_seconds = 3600  # Print a status summary every hour (0 = disabled)
```

When enabled, you'll see periodic summaries like:
```
Last 1 hour: 12 tweets scored, 3 replies sent, 1 tweet posted, 0 threads posted.
```

---

## Environment Variables

Every config option can also be set via environment variables. This is useful for CI/CD, Docker, or keeping secrets out of files.

The pattern is: `REPLYGUY_<SECTION>__<KEY>` (double underscore between section and key, all uppercase).

**Common examples:**

```bash
# API keys (keep these out of config files)
export REPLYGUY_X_API__CLIENT_ID="your-client-id"
export REPLYGUY_LLM__API_KEY="sk-your-openai-key"

# Override settings without editing the config file
export REPLYGUY_LIMITS__MAX_REPLIES_PER_DAY=10
export REPLYGUY_SCORING__THRESHOLD=80
export REPLYGUY_LLM__PROVIDER=anthropic
export REPLYGUY_LLM__MODEL=claude-sonnet-4-6

# List values use commas
export REPLYGUY_BUSINESS__PRODUCT_KEYWORDS="rust, cli tools, developer productivity"
```

**Priority order** (highest wins):
1. Environment variables (`REPLYGUY_*`)
2. Config file (`~/.replyguy/config.toml`)
3. Built-in defaults

---

## X API Tiers

ReplyGuy works on X's **Free** tier, but with limitations:

| Feature | Free Tier | Basic Tier ($100/mo) |
|---------|-----------|---------------------|
| Tweet discovery (search) | Not available | Available |
| Reply to tweets | Available | Available |
| Post tweets | Available | Available |
| Post threads | Available | Available |
| Monitor mentions | Available | Available |

On the **Free** tier, the discovery loop is automatically disabled. ReplyGuy will still reply to your @-mentions, post educational tweets, and publish threads — just without the proactive tweet-finding feature.

If you upgrade to Basic, ReplyGuy automatically detects it and enables discovery.

---

## Running as a Background Service

### Using tmux (recommended for servers)

```bash
# Start a new tmux session
tmux new -s replyguy

# Start the agent
replyguy run

# Detach from the session: press Ctrl+B, then D
# Re-attach later:
tmux attach -t replyguy
```

### Using systemd (Linux)

Create `/etc/systemd/system/replyguy.service`:

```ini
[Unit]
Description=ReplyGuy - Autonomous X Growth Assistant
After=network.target

[Service]
Type=simple
User=youruser
ExecStart=/home/youruser/.cargo/bin/replyguy run
Restart=on-failure
RestartSec=30
Environment=REPLYGUY_LLM__API_KEY=sk-your-key-here

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl enable replyguy
sudo systemctl start replyguy
sudo systemctl status replyguy    # Check if it's running
journalctl -u replyguy -f         # Follow logs
```

### Using launchd (macOS)

Create `~/Library/LaunchAgents/com.replyguy.agent.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.replyguy.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/youruser/.cargo/bin/replyguy</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/replyguy.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/replyguy.error.log</string>
</dict>
</plist>
```

Then:

```bash
launchctl load ~/Library/LaunchAgents/com.replyguy.agent.plist
```

---

## Troubleshooting

### Auth Errors

| Error | What went wrong | How to fix it |
|-------|----------------|---------------|
| "Something went wrong, you weren't able to give access to the App" | Callback URL doesn't match | Go to developer.x.com → your app → Settings → User authentication. Set the callback URL to exactly `http://127.0.0.1:8080/callback` |
| "Unauthorized" after authorizing | Wrong Client ID or app type mismatch | Double-check your Client ID. Make sure app type is "Native App" (public client) |
| "Invalid scopes" | App permissions too restrictive | Set app permissions to "Read and write" in developer.x.com |
| Port 8080 already in use | Another app is using port 8080 | Either stop the other app, or change `auth.callback_port` in config and update the callback URL in developer.x.com to match |
| "No tokens found" when running `replyguy test` | You haven't authenticated yet | Run `replyguy auth` |
| "Token expired" | Auth tokens expired and couldn't auto-refresh | Run `replyguy auth` again |

### Common Issues

**ReplyGuy isn't finding any tweets to reply to**
- If you're on the Free X API tier, discovery is disabled. Upgrade to Basic ($100/mo) to enable it.
- Check your keywords — are they too specific? Try broader terms.
- Lower the scoring threshold: set `scoring.threshold = 60` in your config.

**Replies sound too robotic**
- Add a `brand_voice` in your config. Be specific: "Casual, friendly, like texting a knowledgeable friend" works better than "professional".
- Add `reply_style` guidelines: "Lead with empathy. Ask follow-up questions. Never be pushy."

**"Rate limited" messages in logs**
- This is normal and expected. It means ReplyGuy hit its daily cap and is waiting. The limits reset every 24 hours.

**"Reply phrasing too similar" messages**
- ReplyGuy detected it was about to post a reply that sounds too much like a recent one. This is a safety feature to avoid looking like a bot. It will try again with the next tweet.

**Database growing too large**
- Lower `storage.retention_days` (default: 90). Old tweets and replies are cleaned up automatically.
- Data at `~/.replyguy/replyguy.db` — you can delete it to start fresh (you won't lose your config or auth tokens).

### Verbose Logging

For debugging, enable verbose logging:

```bash
# Show debug-level logs
replyguy run --verbose

# Or use RUST_LOG for fine-grained control
RUST_LOG=replyguy_core=debug replyguy run

# See only a specific module
RUST_LOG=replyguy_core::automation=debug replyguy run
```

---

## File Locations

| File | Purpose |
|------|---------|
| `~/.replyguy/config.toml` | Your configuration |
| `~/.replyguy/tokens.json` | X API auth tokens (auto-refreshed) |
| `~/.replyguy/replyguy.db` | SQLite database (tweets, replies, rate limits) |

All files are stored in `~/.replyguy/`. To start completely fresh, delete the entire directory:

```bash
rm -rf ~/.replyguy
```

Then run `replyguy init` again.

---

## Global Flags

These work with any command:

```bash
replyguy -c /path/to/config.toml run   # Use a custom config file
replyguy --verbose run                   # Debug-level logging
replyguy --quiet run                     # Only show errors
```

---

## Building From Source

```bash
# Debug build (faster to compile, slower to run)
cargo build

# Release build (slower to compile, faster to run)
cargo build --release

# Run tests
cargo test

# Check code style
cargo clippy --workspace
cargo fmt --all -- --check
```

---

## License

MIT — see [LICENSE](LICENSE) for details.
