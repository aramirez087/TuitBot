# Tuitbot

**Your autonomous X (Twitter) growth assistant.** Tuitbot runs in the background and grows your X account on autopilot — finding relevant conversations, replying with genuinely helpful content, posting educational tweets, and publishing weekly threads. It's like having a social media manager who never sleeps.

Built for **founders, indie hackers, and solo makers** who'd rather build their product than spend hours on X every day.

---

## What Does Tuitbot Actually Do?

Tuitbot runs up to six automated loops while you focus on building. It respects configurable **active hours** so it only posts during times that look natural (default: 8 AM – 10 PM in your timezone):

### 1. Finds Conversations That Matter
Tuitbot searches X for tweets matching your product's keywords (e.g., "mac productivity", "clipboard manager"). It scores every tweet using a 6-signal scoring engine — keyword relevance, follower sweet spot (1K-5K), recency, engagement rate, reply count (prefers underserved conversations), and content type (prefers text-only tweets).

### 2. Replies With Genuinely Helpful Content
When it finds a high-scoring tweet, Tuitbot uses AI to write a natural, helpful reply using varied **reply archetypes** — agree and expand, ask a question, share an experience, add data, or respectfully disagree. Each reply sounds different. It only mentions your product 20% of the time (configurable), and never uses banned phrases like "check out" or "you should try".

### 3. Posts Educational Tweets
Every few hours, Tuitbot posts original tweets using varied **formats** — lists, contrarian takes, storytelling, tips, questions, and more. It uses **epsilon-greedy topic selection**: 80% of the time it picks topics that perform well, 20% of the time it explores new ones.

### 4. Publishes Weekly Threads
Once a week, Tuitbot creates a multi-tweet thread using varied **structures** — transformation stories, frameworks, common mistakes, or deep analysis.

### 5. Monitors Your Mentions
When someone @-mentions you, Tuitbot generates a thoughtful reply automatically. You never leave someone hanging.

### 6. Tracks Target Accounts
Configure specific accounts you want to engage with. Tuitbot monitors their tweets and builds relationships over time — with optional auto-follow and a warmup period before first engagement.

### 7. Tracks Analytics
Tuitbot snapshots your follower count daily, measures engagement on your content after 24 hours, and alerts you if followers drop more than 2%. Use `tuitbot stats` to see your dashboard.

> **Note on X API access:** Features 1, 2, 5, and 6 (discovery, replies, mentions, and target monitoring) require a paid X API tier or pay-per-use credits. Features 3 and 4 (posting tweets and threads) work on the Free tier. See [X API Access & Pricing](#x-api-access--pricing) for details.

### Built-In Safety
Tuitbot is designed to keep your account safe:
- **Conservative reply limits**: 5 replies/day (default), 6 tweets/day, 1 thread/week
- **Per-author limits**: Max 1 reply per author per day — no harassment patterns
- **Banned phrase filter**: Automatically blocks replies containing spammy phrases ("check out", "you should try", "I recommend", "link in bio")
- **Product mention ratio**: Only mentions your product 20% of the time (configurable)
- **Active hours**: Configurable timezone-aware schedule — the bot sleeps outside your active hours (default: 8 AM – 10 PM). Wrapping ranges like 22-06 are supported for night owls
- **Human-like jitter**: All loop intervals and posting delays are randomized to prevent perfectly periodic, machine-detectable patterns
- **Random delays**: 45-180 seconds of random delay between actions to look natural
- **Deduplication**: Never replies to the same tweet twice, and detects when replies sound too similar to recent ones
- **Approval mode**: Optionally queue all posts for human review before posting (`approval_mode = true`)
- **Graceful shutdown**: Press Ctrl+C anytime — it stops cleanly within 30 seconds

---

## Prerequisites

You'll need three things before getting started:

| What | Where to get it | Cost |
|------|-----------------|------|
| **Rust 1.75+** | [rustup.rs](https://rustup.rs/) | Free |
| **X API developer account** | [developer.x.com](https://developer.x.com) | Pay-per-use (a few $/mo) or Free (limited — see [X API access](#x-api-access--pricing)) |
| **An AI provider** | [OpenAI](https://platform.openai.com/), [Anthropic](https://console.anthropic.com/), or [Ollama](https://ollama.ai/) (local, free) | Varies (Ollama is free) |

---

## Installation

```bash
# Option 1: Install the binary globally (recommended)
cargo install --path crates/tuitbot-cli

# Option 2: Build without installing
cargo build --release
# Binary will be at: target/release/tuitbot
```

After installation, verify it works:

```bash
tuitbot --help
```

---

## Getting Started (Step by Step)

The entire setup takes about 5 minutes. Tuitbot has a built-in wizard that walks you through everything.

### Step 1: Set Up Your X Developer App

Before running the wizard, you need to create an app on X's developer portal. This is what gives Tuitbot permission to post on your behalf.

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
tuitbot init
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
- Discovery keywords, comma-separated (e.g., `macos productivity, mac menu bar, clipboard manager`) — these are the terms Tuitbot searches for on X
- Content topics, comma-separated (e.g., `Mac productivity tips, macOS workflows`) — these are what Tuitbot tweets about

**Section 3 — Brand Voice (optional, but recommended):**
- Brand voice/personality — describe how you want to sound (e.g., "Friendly technical expert. Casual tone, occasionally witty.")
- Reply style — how should replies feel? (e.g., "Lead with genuine help. Only mention our product if directly relevant.")
- Content style — how should tweets/threads sound? (e.g., "Share practical tips with real examples.")

> **Tip**: If you skip these, Tuitbot uses sensible defaults ("be conversational and helpful, not salesy"). You can always edit them later with `tuitbot settings`.

**Section 4 — AI Provider:**
- Choose your provider: `openai` (default), `anthropic`, or `ollama`
- Paste your API key (not needed for Ollama)
- Choose a model (defaults: `gpt-4o-mini` for OpenAI, `claude-sonnet-4-6` for Anthropic, `llama3.2` for Ollama)

After the wizard, it will show you a summary and ask to save. Then it seamlessly offers to continue with authentication and testing.

### Step 3: Authenticate With X

If you didn't do this during the wizard, run:

```bash
tuitbot auth
```

This opens your browser to X's authorization page. Click **"Authorize app"**, and Tuitbot will automatically catch the callback. You'll see:

```
Callback server listening on 127.0.0.1:8080
Authenticated as @yourusername. Tokens saved to ~/.tuitbot/tokens.json
```

> **Running on a remote server?** Use manual mode: `tuitbot auth --mode manual`. It will print a URL to open in any browser and ask you to paste back the authorization code.

Your tokens are saved securely and refresh automatically — you only need to do this once.

### Step 4: Validate Everything

```bash
tuitbot test
```

This checks all five components and tells you if anything is wrong:

```
Configuration:     OK (loaded from /home/user/.tuitbot/config.toml)
Business profile:  OK (product_name: "Docklet", 3 keywords, 4 topics)
X API auth:        OK (token valid, expires in 1h 45m)
LLM provider:      OK (openai, model: gpt-4o-mini)
Database:          OK (will be created at ~/.tuitbot/tuitbot.db)
```

If any check fails, it tells you exactly what's wrong and how to fix it.

### Step 5: Start the Agent

```bash
tuitbot run
```

That's it. Tuitbot is now running. You'll see a startup banner:

```
Tuitbot v0.1.0
Tier: Free | Loops: content, threads, analytics
Press Ctrl+C to stop.
```

Leave it running in the background. Press **Ctrl+C** anytime to stop it gracefully.

> **Tip**: Run it in a terminal multiplexer like `tmux` or `screen` so it keeps running after you close your terminal, or see [Running as a Background Service](#running-as-a-background-service) below.

---

## One-Off Commands

You don't have to run the full agent. Tuitbot also has commands for doing things one at a time:

| Command | What it does |
|---------|-------------|
| `tuitbot discover` | Search for tweets matching your keywords, score them, and reply to the best ones — once |
| `tuitbot discover --dry-run` | Same as above, but **don't actually post** — just show what it would do |
| `tuitbot mentions` | Check your @-mentions and reply to new ones — once |
| `tuitbot post` | Generate and post a single educational tweet |
| `tuitbot post --topic "Mac shortcuts"` | Post a tweet on a specific topic |
| `tuitbot thread` | Generate and post a thread |
| `tuitbot thread --topic "5 macOS tips"` | Post a thread on a specific topic |
| `tuitbot score <tweet_id>` | Score a specific tweet to see if Tuitbot would reply to it |
| `tuitbot stats` | Show analytics dashboard — follower trend, top topics, engagement rates |
| `tuitbot approve` | Review and approve queued posts (when `approval_mode = true`) |
| `tuitbot settings` | Interactive settings editor — change any config value without editing TOML |
| `tuitbot settings --show` | Pretty-print your current configuration (secrets masked) |
| `tuitbot settings --set scoring.threshold=80` | Change a single setting directly (scriptable) |
| `tuitbot settings scoring` | Jump straight to a specific category (product, voice, persona, ai, x, targets, limits, scoring, timing, approval, schedule, storage) |
| `tuitbot upgrade` | Detect and configure new features added since your last setup |
| `tuitbot upgrade --non-interactive` | Same as above, but apply default values without prompting |

The `--dry-run` flag is great for testing. It shows you exactly what Tuitbot would do without actually posting anything.

> **Tip:** When you update Tuitbot, `tuitbot run` automatically detects new config features and offers to configure them. You can also run `tuitbot upgrade` explicitly at any time.

---

## Configuration Reference

Your config lives at `~/.tuitbot/config.toml`. The wizard creates it for you. To change settings, either run `tuitbot settings` for an interactive editor, use `tuitbot settings --set KEY=VALUE` for quick changes, or edit the file directly with any text editor. Here's every section explained in plain English.

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

This is the most important section — it tells Tuitbot what your product is, who to talk to, and what to talk about.

```toml
[business]
product_name = "Docklet"
product_description = "Floating command strip for macOS"
product_url = "https://getdocklet.app"      # Optional
target_audience = "Mac power users and developers"

# Keywords that Tuitbot searches for on X.
# Be specific! "productivity" is too broad. "macos productivity app" is better.
product_keywords = ["macos productivity", "mac menu bar", "mac clipboard manager"]

# Optional: competitor or alternative keywords
competitor_keywords = ["notchnook alternative", "bartender mac"]

# Topics for original tweets and threads.
# Tuitbot rotates through these so content stays fresh.
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

### Persona (Optional)

Make Tuitbot sound more human by giving it opinions, experiences, and content pillars to draw from:

```toml
[business]
# Strong opinions your persona holds (injected into reply/tweet prompts):
persona_opinions = [
    "Native apps will always beat Electron",
    "Keyboard shortcuts are underrated",
]

# Experiences to reference in content:
persona_experiences = [
    "Built 3 macOS apps over 5 years",
    "Switched from Windows to Mac in 2019",
]

# Core topics your content revolves around:
content_pillars = ["Mac productivity", "Swift development", "Indie hacking"]
```

### Target Accounts

Instead of only searching by keyword, you can monitor specific accounts and engage with their content. This builds relationships with the people who matter most to your growth.

```toml
[targets]
accounts = ["@jason", "@levelsio", "@paborenstein"]
max_target_replies_per_day = 3   # Separate limit from general replies
auto_follow = false               # Automatically follow target accounts
follow_warmup_days = 3            # Wait this many days after following before engaging
```

### Approval Mode

For peace of mind, enable approval mode to review all generated content before it's posted:

```toml
approval_mode = true  # Queue posts for human review instead of posting automatically
```

When enabled, all loops queue their output instead of posting. Use `tuitbot approve` to review:

```bash
tuitbot approve
# Shows each pending item with context. Type y/n/s/q to approve/reject/skip/quit.
```

### Active Hours Schedule

Control when Tuitbot is active. Outside these hours, posting loops sleep — no 3 AM tweets. The analytics loop runs 24/7 since it only measures, never posts.

```toml
[schedule]
timezone = "America/New_York"       # IANA timezone name (handles DST automatically)
active_hours_start = 8              # Hour (0-23) when the bot wakes up
active_hours_end = 22               # Hour (0-23) when the bot goes to sleep
active_days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
```

Wrapping ranges work too — set `start = 22` and `end = 6` if you want late-night-to-early-morning activity.

> **Tip**: Match your active hours to when your target audience is online. For US-focused accounts, `America/New_York` with 8-22 covers the full US workday across time zones.

### Scoring Engine

Controls how Tuitbot decides which tweets are worth replying to. Each tweet gets a score from 0-100 based on six signals:

```toml
[scoring]
threshold = 60  # Minimum score to trigger a reply (0-100)

# How much each signal contributes (must add up to 100):
keyword_relevance_max = 25.0    # How well the tweet matches your keywords
follower_count_max = 15.0       # Bell curve peaking at ~1K followers (sweet spot)
recency_max = 10.0              # How recently the tweet was posted
engagement_rate_max = 15.0      # Likes + retweets + replies relative to followers
reply_count_max = 15.0          # Fewer replies = higher score (underserved conversations)
content_type_max = 10.0         # Text-only tweets score highest (no media/quotes)
```

The scoring is designed to find **underserved conversations from mid-range accounts** — tweets with few replies where your reply will actually be seen, from accounts with 1K-5K followers who are likely to engage back.

> **Tip**: Start with the default threshold of 60. If Tuitbot is replying too much, raise it to 70. If it's not finding enough tweets, lower it to 50.

### Safety Limits

These caps prevent Tuitbot from posting too aggressively and getting your account flagged. The defaults are conservative on purpose.

```toml
[limits]
max_replies_per_day = 5                 # Max replies per 24 hours
max_tweets_per_day = 6                  # Max original tweets per 24 hours
max_threads_per_week = 1               # Max threads per 7 days
max_replies_per_author_per_day = 1     # Max replies to the same person per day
min_action_delay_seconds = 45          # Minimum random delay between actions
max_action_delay_seconds = 180         # Maximum random delay between actions
product_mention_ratio = 0.2           # Only mention your product 20% of the time

# Phrases that will be automatically blocked from replies:
banned_phrases = ["check out", "you should try", "I recommend", "link in bio"]
```

Tuitbot adds a random delay between `min` and `max` seconds before each action. This makes activity look natural rather than bot-like.

> **Tip**: The defaults are intentionally conservative. 5 replies/day with only 20% product mentions keeps you far from spam territory.

### Automation Intervals

How often each automation loop runs. Shorter intervals = more active, but uses more API quota.

```toml
[intervals]
mentions_check_seconds = 300        # Check mentions every 5 minutes
discovery_search_seconds = 900      # Search for tweets every 15 minutes
content_post_window_seconds = 10800 # Post a tweet at most every 3 hours
thread_interval_seconds = 604800    # Post a thread at most every 7 days
```

| Interval | Default | Human-readable |
|----------|---------|----------------|
| Mention checks | 300 | Every 5 minutes |
| Discovery searches | 900 | Every 15 minutes |
| Content tweets | 10,800 | Every 3 hours |
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

> **On a budget?** Use `gpt-4o-mini` (very cheap) or Ollama (completely free, runs on your machine). Tuitbot uses very few tokens — a few cents per day with OpenAI.

### Storage

```toml
[storage]
db_path = "~/.tuitbot/tuitbot.db"  # Where the SQLite database lives
retention_days = 90                   # Delete old data after this many days (0 = keep forever)
```

Tuitbot stores everything in a local SQLite database — discovered tweets, replies sent, original tweets, threads, and rate limit counters. Old data is cleaned up automatically based on `retention_days`.

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

The pattern is: `TUITBOT_<SECTION>__<KEY>` (double underscore between section and key, all uppercase).

**Common examples:**

```bash
# API keys (keep these out of config files)
export TUITBOT_X_API__CLIENT_ID="your-client-id"
export TUITBOT_LLM__API_KEY="sk-your-openai-key"

# Override settings without editing the config file
export TUITBOT_LIMITS__MAX_REPLIES_PER_DAY=10
export TUITBOT_SCORING__THRESHOLD=80
export TUITBOT_LLM__PROVIDER=anthropic
export TUITBOT_LLM__MODEL=claude-sonnet-4-6

# Schedule
export TUITBOT_SCHEDULE__TIMEZONE="America/New_York"
export TUITBOT_SCHEDULE__ACTIVE_HOURS_START=9
export TUITBOT_SCHEDULE__ACTIVE_HOURS_END=21

# List values use commas
export TUITBOT_BUSINESS__PRODUCT_KEYWORDS="rust, cli tools, developer productivity"
```

**Priority order** (highest wins):
1. Environment variables (`TUITBOT_*`)
2. Config file (`~/.tuitbot/config.toml`)
3. Built-in defaults

---

## X API Access & Pricing

X's API pricing has changed significantly. Here's what you need to know.

### Pay-Per-Use (recommended for most users)

As of early 2026, X's **default** for new developers is **pay-per-use** — no subscription, you buy credits and pay per API call. This gives you access to **all endpoints** Tuitbot needs:

| Operation | Cost per call |
|-----------|--------------|
| Read a tweet | $0.005 |
| Post a tweet / reply | $0.010 |
| Read user profile | $0.010 |

For typical Tuitbot usage (moderate discovery + a few replies/tweets per day), expect roughly **$1-5/month** depending on how active your settings are. You can set spending limits in the X developer dashboard.

> **This is the easiest way to get started.** Sign up at [developer.x.com](https://developer.x.com), add credits, and all features work immediately.

### Legacy Subscription Tiers

X still offers fixed subscription plans, which can be more cost-effective at high volume:

| Feature | Free ($0) | Basic ($200/mo) | Pro ($5,000/mo) |
|---------|-----------|-----------------|-----------------|
| Post tweets & replies | Yes (write-only) | Yes | Yes |
| Search tweets (discovery) | No | Yes (last 7 days) | Yes (full archive) |
| Read mentions | No | Yes | Yes |
| Read timelines | No | Yes | Yes |
| Monthly post cap | ~1,500 | 50,000 | 300,000 |

### What Works on Each Tier

Tuitbot auto-detects your tier at startup and enables/disables features accordingly:

| Tuitbot Feature | Free Tier | Basic / Pro / Pay-Per-Use |
|------------------|-----------|---------------------------|
| Post educational tweets | Yes | Yes |
| Post threads | Yes | Yes |
| Tweet discovery (search + reply) | No | Yes |
| Monitor & reply to mentions | No | Yes |

**On the Free tier**, only the content and thread loops run — Tuitbot can post tweets and threads on your behalf, but cannot search for conversations or monitor your mentions (these are read endpoints that the Free tier doesn't include).

**On Basic, Pro, or pay-per-use**, all loops run — discovery, mentions, content, threads, target monitoring, and analytics. Tuitbot automatically detects this and enables everything.

> **Bottom line:** If you want the full Tuitbot experience (finding and replying to relevant tweets + monitoring mentions), you need at least the pay-per-use tier with credits loaded. The Free tier only supports posting original content.

---

## Running as a Background Service

### Using tmux (recommended for servers)

```bash
# Start a new tmux session
tmux new -s tuitbot

# Start the agent
tuitbot run

# Detach from the session: press Ctrl+B, then D
# Re-attach later:
tmux attach -t tuitbot
```

### Using systemd (Linux)

Create `/etc/systemd/system/tuitbot.service`:

```ini
[Unit]
Description=Tuitbot - Autonomous X Growth Assistant
After=network.target

[Service]
Type=simple
User=youruser
ExecStart=/home/youruser/.cargo/bin/tuitbot run
Restart=on-failure
RestartSec=30
Environment=TUITBOT_LLM__API_KEY=sk-your-key-here

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl enable tuitbot
sudo systemctl start tuitbot
sudo systemctl status tuitbot    # Check if it's running
journalctl -u tuitbot -f         # Follow logs
```

### Using launchd (macOS)

Create `~/Library/LaunchAgents/com.tuitbot.agent.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.tuitbot.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/youruser/.cargo/bin/tuitbot</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/tuitbot.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tuitbot.error.log</string>
</dict>
</plist>
```

Then:

```bash
launchctl load ~/Library/LaunchAgents/com.tuitbot.agent.plist
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
| "No tokens found" when running `tuitbot test` | You haven't authenticated yet | Run `tuitbot auth` |
| "Token expired" | Auth tokens expired and couldn't auto-refresh | Run `tuitbot auth` again |

### Common Issues

**Tuitbot isn't finding any tweets to reply to**
- If you're on the Free X API tier, discovery is disabled. Upgrade to Basic ($100/mo) to enable it.
- Check your keywords — are they too specific? Try broader terms.
- Lower the scoring threshold: set `scoring.threshold = 60` in your config.

**Replies sound too robotic**
- Add a `brand_voice` in your config. Be specific: "Casual, friendly, like texting a knowledgeable friend" works better than "professional".
- Add `reply_style` guidelines: "Lead with empathy. Ask follow-up questions. Never be pushy."

**"Rate limited" messages in logs**
- This is normal and expected. It means Tuitbot hit its daily cap and is waiting. The limits reset every 24 hours.

**"Reply phrasing too similar" messages**
- Tuitbot detected it was about to post a reply that sounds too much like a recent one. This is a safety feature to avoid looking like a bot. It will try again with the next tweet.

**Database growing too large**
- Lower `storage.retention_days` (default: 90). Old tweets and replies are cleaned up automatically.
- Data at `~/.tuitbot/tuitbot.db` — you can delete it to start fresh (you won't lose your config or auth tokens).

### Verbose Logging

For debugging, enable verbose logging:

```bash
# Show debug-level logs
tuitbot run --verbose

# Or use RUST_LOG for fine-grained control
RUST_LOG=tuitbot_core=debug tuitbot run

# See only a specific module
RUST_LOG=tuitbot_core::automation=debug tuitbot run
```

---

## File Locations

| File | Purpose |
|------|---------|
| `~/.tuitbot/config.toml` | Your configuration |
| `~/.tuitbot/tokens.json` | X API auth tokens (auto-refreshed) |
| `~/.tuitbot/tuitbot.db` | SQLite database (tweets, replies, rate limits) |

All files are stored in `~/.tuitbot/`. To start completely fresh, delete the entire directory:

```bash
rm -rf ~/.tuitbot
```

Then run `tuitbot init` again.

---

## Global Flags

These work with any command, in either position (before or after the subcommand):

```bash
tuitbot --verbose run                   # Debug-level logging
tuitbot run --verbose                   # Same thing — flags work in either position
tuitbot --quiet run                     # Only show errors
tuitbot -c /path/to/config.toml run     # Use a custom config file
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
