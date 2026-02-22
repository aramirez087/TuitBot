<div align="center">

<img src="assets/logo.svg" alt="Tuitbot Logo" width="120" />

# Tuitbot

**Your autonomous X (Twitter) growth assistant.**

[![Rust](https://img.shields.io/badge/built_with-Rust-d32822?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![X API](https://img.shields.io/badge/X_API-Ready-black?style=for-the-badge&logo=x)](https://developer.x.com)
[![CI](https://img.shields.io/github/actions/workflow/status/aramirez087/TuitBot/ci.yml?branch=main&style=for-the-badge&label=CI)](https://github.com/aramirez087/TuitBot/actions/workflows/ci.yml)
[![Releases](https://img.shields.io/github/v/release/aramirez087/TuitBot?style=for-the-badge)](https://github.com/aramirez087/TuitBot/releases)

Tuitbot grows your X account on autopilot — finding relevant conversations, replying with genuinely helpful content, posting educational tweets, and publishing weekly threads. **It's like having a social media manager who never sleeps.**

Built for **founders**, **indie hackers**, and **solo makers** who'd rather build their product than spend hours on X.

[Two Modes](#-two-operating-modes) · [Features](#-features) · [Getting Started](#-getting-started) · [Configuration](#-configuration-reference) · [Deployment](#-deployment)

</div>

---

## Two Operating Modes

Tuitbot runs in two distinct modes. Pick the one that fits your infrastructure:

| | **Standalone Daemon** | **Scheduler-Driven (Tick)** |
|---|---|---|
| **Command** | `tuitbot run` | `tuitbot tick` |
| **How it works** | Long-running process with 6 concurrent loops | Single pass through all loops, then exits |
| **Best for** | VPS, tmux, dedicated server | Cron, systemd timers, launchd, [OpenClaw](https://openclaw.org) |
| **Scheduling** | Built-in (active hours + jitter) | External scheduler decides when to invoke |
| **Process model** | Always running, graceful shutdown on SIGTERM | Runs once, exits with JSON summary |
| **Resource usage** | Persistent (~15 MB RSS) | Zero when idle, spins up on schedule |

Both modes share the same config, the same database, and the same safety guardrails. You can switch between them freely — they use a process lock to prevent overlap.

### Standalone Daemon (`tuitbot run`)

The traditional mode. Tuitbot runs as a long-lived process, managing its own loop timing, jitter, and active-hours gating:

```bash
tuitbot run
# Leave running. Ctrl+C to gracefully stop.
```

### Scheduler-Driven (`tuitbot tick`)

The new idempotent mode. Each invocation runs every enabled loop exactly once and exits with a structured summary. Your external scheduler (cron, systemd timer, OpenClaw) controls the cadence:

```bash
# Run all loops once
tuitbot tick

# Dry run — see what would happen without posting
tuitbot tick --dry-run

# Run only specific loops
tuitbot tick --loops discovery,content,analytics

# Ignore active-hours window (useful for testing)
tuitbot tick --ignore-schedule

# Get machine-readable JSON output
tuitbot tick --output json
```

**Process lock:** Only one `tuitbot tick` (or `tuitbot run`) can execute at a time. A second invocation exits immediately with a clear error.

**Exit summary:** Every tick prints a per-loop status report (completed / skipped / failed) with counts and timing. With `--output json`, this is a structured JSON object ready for log aggregation.

---

## Features

Tuitbot runs up to six automated loops while you focus on building. It respects configurable **active hours** and supports precise **slot-based scheduling** so you can target peak engagement windows or fall back to a natural, interval-based cadence.

### 1. Finds Conversations That Matter
Searches X for tweets matching your product's keywords. Tuitbot uses a **6-signal scoring engine** to find the perfect interactions:
* **Keyword relevance** · **Follower sweet spot (1K-5K)** · **Recency** · **Engagement rate** · **Reply count** (prefers underserved conversations) · **Content type** (prefers text-only).

### 2. Replies With Genuinely Helpful Content
When it finds a high-scoring tweet, Tuitbot uses AI to write a natural, helpful reply using varied **reply archetypes** (agree and expand, ask a question, share an experience). It mentions your product sparingly (configurable) and never uses banned phrases like *"check out"* or *"link in bio"*.

### 3. Posts Educational Tweets
Tuitbot posts original tweets at your preferred precise times (e.g., peak engagement windows) or automatically every few hours using varied formats (lists, contrarian takes, tips, questions). It uses **epsilon-greedy topic selection** to continuously explore new topics while doubling down on what performs best.

### 4. Publishes Weekly Threads
Once a week, Tuitbot crafts a multi-tweet thread using proven structures: transformation stories, frameworks, common mistakes, or deep analysis. You can target specific high-traction days and times for your threads to maximize reach.

### 5. Monitors Mentions & Tracks Targets
* Automatically generates thoughtful replies when someone @-mentions you.
* Monitors specific target accounts and builds relationships over time with an optional auto-follow and engagement warmup period.

### 6. Tracks Analytics
Snapshots your follower count daily, measures engagement after 24 hours, and alerts you if followers drop. View your dashboard anytime using `tuitbot stats`.

> **Note on X API access:** Discovery, replies, mentions, and target monitoring require a paid X API tier or pay-per-use credits. Posting tweets and threads works perfectly on the Free tier. See [X API Access](#-x-api-access--pricing) for details.

---

## Built-In Safety & Anti-Spam

Tuitbot is engineered to keep your account totally safe and maintain your pristine reputation:

| Feature | Description |
|---|---|
| **Conservative Limits** | Defaults to **5 replies/day**, **6 tweets/day**, **1 thread/week**. |
| **Anti-Harassment** | Maximum 1 reply per author per day. |
| **Spam Filter** | Automatically blocks replies containing salesy phrases. |
| **Active Hours** | Sleeps completely outside your active hours (default 8 AM - 10 PM). |
| **Human-like Jitter** | Intervals and post delays are randomized (45-180s) to avoid bot detection. |
| **Deduplication** | Never replies to the same tweet twice; detects and prevents repetitive phrasing. |
| **Approval Mode** | Set `approval_mode = true` to queue all posts for simple human review. |
| **Process Lock** | Only one instance runs at a time — no accidental double-posting. |

---

## Getting Started

### Prerequisites

| Requirement | Where to get it | Cost |
|---|---|---|
| **X API Developer Account** | [developer.x.com](https://developer.x.com) | Pay-per-use or Free |
| **AI Provider** | [OpenAI](https://platform.openai.com/), [Anthropic](https://console.anthropic.com/), or [Ollama](https://ollama.ai/) | Varies (Ollama is Free) |

### Install

**Fastest (recommended):**
```bash
curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | bash
```

**From source (Rust 1.75+):**
```bash
cargo install --path crates/tuitbot-cli --locked
```

**Windows:** download `tuitbot-x86_64-pc-windows-msvc.zip` from [Releases](https://github.com/aramirez087/TuitBot/releases), unzip, and add `tuitbot.exe` to your `PATH`.

### Setup (both modes)

Steps 1-3 are identical regardless of which mode you choose.

#### 1. Set Up X Developer App
Go to the [X Developer Portal](https://developer.x.com/en/portal/dashboard) and create an app.
* **App permissions:** Read and write
* **Type of App:** Native App *(also called "public client")*
* **Callback URI:** `http://127.0.0.1:8080/callback` **(Must be exact — no HTTPS or trailing slash)**
* **Website URL:** Your product's URL

Copy your **OAuth 2.0 Client ID**.

#### 2. Initialize Tuitbot
```bash
tuitbot init
```
Follow the interactive prompts to define your Business Profile, Brand Voice, and AI Provider.

#### 3. Authenticate & Test
```bash
tuitbot auth        # Authorize your X account via browser
tuitbot test        # Validate all components
```

#### 4a. Run as Standalone Daemon

```bash
tuitbot run
# Leave running. Ctrl+C to gracefully stop.
```

That's it. Tuitbot manages its own timing, active hours, and loop scheduling internally.

#### 4b. Run with an External Scheduler

Set up your scheduler to invoke `tuitbot tick` on a cadence you control. Here are examples for common schedulers:

**Cron (every 30 minutes):**
```bash
crontab -e
# Add:
*/30 * * * * /usr/local/bin/tuitbot tick --output json >> /var/log/tuitbot-tick.log 2>&1
```

**systemd timer (every 20 minutes):**
```ini
# /etc/systemd/system/tuitbot-tick.timer
[Unit]
Description=Tuitbot tick timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=20min

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/tuitbot-tick.service
[Unit]
Description=Tuitbot tick

[Service]
Type=oneshot
ExecStart=/usr/local/bin/tuitbot tick --output json
User=tuitbot
```

```bash
sudo systemctl enable --now tuitbot-tick.timer
```

**macOS launchd (every 30 minutes):**
```xml
<!-- ~/Library/LaunchAgents/com.tuitbot.tick.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.tuitbot.tick</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/tuitbot</string>
        <string>tick</string>
        <string>--output</string>
        <string>json</string>
    </array>
    <key>StartInterval</key>
    <integer>1800</integer>
    <key>StandardOutPath</key>
    <string>/tmp/tuitbot-tick.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tuitbot-tick.log</string>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/com.tuitbot.tick.plist
```

**OpenClaw:**

Tuitbot ships with a `SKILL.md` that teaches OpenClaw-compatible assistants how to operate the agent. The `tick` command is the recommended integration point — OpenClaw calls `tuitbot tick --output json` on its schedule and parses the JSON summary for monitoring:

```bash
# OpenClaw invokes this on its own cadence
tuitbot tick --output json --dry-run    # Safe testing
tuitbot tick --output json              # Production
```

> **Choosing a cadence:** Tuitbot's built-in rate limits and safety checks mean you can tick aggressively (every 15-30 minutes) without worry. Loops that have nothing to do (e.g., content posted too recently) will report `skipped` and exit instantly. A tick with nothing to do takes <2 seconds.

---

## CLI Commands

You don't have to run the full un-supervised agent. Tuitbot has granular commands for precise control:

```bash
# Operating modes
tuitbot run                   # Start as long-running daemon
tuitbot tick                  # Run all loops once and exit
tuitbot tick --dry-run        # See what would happen, post nothing
tuitbot tick --loops discovery,content  # Run only specific loops

# Individual actions
tuitbot discover              # Search, score, and reply to tweets once
tuitbot mentions              # Check and reply to mentions once
tuitbot post                  # Generate and post a single educational tweet
tuitbot thread                # Generate and post a deep-dive thread
tuitbot stats                 # Show comprehensive analytics dashboard

# Testing & Configuration
tuitbot discover --dry-run    # See what it WOULD do, without posting
tuitbot score <tweet_id>      # Score a specific tweet through the engine
tuitbot settings              # Open interactive settings editor
tuitbot settings scoring      # Jump straight to scoring config
tuitbot approve               # Review queued posts (if approval_mode = true)

# AI Agent Integration
tuitbot mcp serve             # Start MCP server for AI agents (stdio transport)
```

---

## Configuration Reference

Your configuration is durably stored at `~/.tuitbot/config.toml`. Use `tuitbot settings` to interactively edit them, or modify the file directly with your favorite editor.

### Business & Persona Profile
Tell Tuitbot exactly what your product is and how you want to sound:
```toml
[business]
product_name = "Docklet"
product_description = "Floating command strip for macOS"
product_keywords = ["macos productivity", "mac menu bar"]
industry_topics = ["Mac productivity tips", "macOS power user workflows"]

brand_voice = "Friendly technical expert. Casual, occasionally witty."
reply_style = "Lead with genuine help. Only mention product if directly relevant."

persona_opinions = ["Native apps will always beat Electron"]
content_pillars = ["Mac productivity", "Swift development", "Indie hacking"]
```

### Automation & Limits
Tuitbot safeguards your account with highly conservative defaults.
```toml
[limits]
max_replies_per_day = 5
max_tweets_per_day = 6
product_mention_ratio = 0.2           # Mention product only 20% of the time (1 in 5)
banned_phrases = ["check out", "you should try", "I recommend", "link in bio"]

[schedule]
timezone = "America/New_York"
active_hours_start = 8
active_hours_end = 22

# Target peak engagement windows for tweets (HH:MM, 24h)
# Use "auto" for research-backed defaults: ["09:15", "12:30", "17:00"]
preferred_times = ["auto"]

# Schedule weekly threads for maximum reach
thread_preferred_day = "Tue"
thread_preferred_time = "10:00"
```

> **Pro Tip:** Need ultimate peace of mind? Set `approval_mode = true` in your config. Tuitbot will queue all outgoing posts for your manual seal of approval via `tuitbot approve` before they ever go live!

<details>
<summary><b>View Advanced Environment Variables</b></summary>
<br/>
Every config option can seamlessly be set via env variables. Highly useful for Docker/CI workflows.
<br/>Format: <code>TUITBOT_&lt;SECTION&gt;__&lt;KEY&gt;</code>

```bash
export TUITBOT_X_API__CLIENT_ID="your-client-id"
export TUITBOT_LLM__API_KEY="sk-your-openai-key"
export TUITBOT_LIMITS__MAX_REPLIES_PER_DAY=10
export TUITBOT_BUSINESS__PRODUCT_KEYWORDS="rust, cli tools, productivity"
```
</details>

---

## AI Providers

Tuitbot supports the leading AI heavyweights right out of the box.

| Provider | Recommended Model | Cost Estimate | Notes |
|---|---|---|---|
| **OpenAI** | `gpt-4o-mini` | ~$0.15 / 1M tokens | Extremely fast and highly economical. |
| **Anthropic** | `claude-sonnet-4-6` | ~$3.00 / 1M tokens | World-class reasoning and prose generation. |
| **Ollama** | `llama3.2` | **Free** | Ultimate privacy. Runs locally on your machine. |

---

## X API Access & Pricing

X's API changed, but Tuitbot natively navigates the complex ecosystem perfectly.

| Plan | Discovery & Replies | Posting Tweets & Threads | Monthly Cost |
|---|:---:|:---:|---|
| **Pay-Per-Use** *(Recommended)* | Yes | Yes | **~$1-5** |
| **Free Tier** | No | Yes | **$0** |
| **Basic Tier** | Yes | Yes | **$100** |

*Tuitbot automatically detects your tier and unlocks features based on what your permissions allow.*

---

## Deployment

### Standalone Daemon

**Linux (tmux):**
```bash
tmux new -s tuitbot
tuitbot run
# Detach with: Ctrl+B, then D
```

**Linux (systemd service):**
```ini
# /etc/systemd/system/tuitbot.service
[Unit]
Description=Tuitbot autonomous X growth assistant
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/tuitbot run
Restart=on-failure
User=tuitbot

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now tuitbot
```

**macOS (launchd daemon):**
```bash
launchctl load ~/Library/LaunchAgents/com.tuitbot.agent.plist
```

### Scheduler-Driven (Tick)

See the [detailed examples in Getting Started](#4b-run-with-an-external-scheduler) for cron, systemd timer, launchd, and OpenClaw configurations.

**Key points for production tick deployments:**
- Use `--output json` and pipe to your log aggregation system
- A 15-30 minute cadence works well; built-in rate limits prevent over-posting
- Tuitbot's active-hours gate still applies by default; use `--ignore-schedule` to let your scheduler own all timing
- The process lock file lives at `~/.tuitbot/tuitbot.lock` and is released automatically on exit or crash

---

## Release Strategy (CI)

Releases are fully automated in GitHub Actions and follow a release-PR model:

1. Every push to `main` runs `.github/workflows/release.yml`.
2. `release-plz` keeps a release PR open with version/changelog updates (`CHANGELOG.md`).
3. Merging that release PR triggers:
   * Tag + GitHub release (`vX.Y.Z`)
   * Cross-platform binary builds (`linux`, `macOS Intel`, `macOS Apple Silicon`, `windows`)
   * Asset uploads + `SHA256SUMS` checksum file

To keep versioning clean, use Conventional Commit prefixes (`feat:`, `fix:`, `chore:`, etc.) and `!` for breaking changes.
Repository setup required once: enable `Settings -> Actions -> General -> Allow GitHub Actions to create and approve pull requests`.
Optional but recommended: set a `RELEASE_PLZ_TOKEN` (PAT) secret so workflows also run on release PRs created by automation.

---

## AI Assistant Integration

Tuitbot ships with a built-in **MCP (Model Context Protocol) server**, making it a first-class tool for AI agents like Claude Code, Cursor, and any MCP-compatible client.

### MCP Server (Recommended)

The MCP server exposes **22 structured tools** over stdio — no CLI parsing required. AI agents can natively discover and call Tuitbot operations with typed inputs and JSON outputs.

**Setup** — add to your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve"]
    }
  }
}
```

With a custom config path:

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["-c", "/path/to/config.toml", "mcp", "serve"]
    }
  }
}
```

**Available MCP Tools:**

| Category | Tools |
|---|---|
| **Analytics** | `get_stats`, `get_follower_trend` |
| **Action Log** | `get_action_log`, `get_action_counts` |
| **Rate Limits** | `get_rate_limits` |
| **Replies** | `get_recent_replies`, `get_reply_count_today` |
| **Targets** | `list_target_accounts` |
| **Discovery** | `list_unreplied_tweets` |
| **Scoring** | `score_tweet` (6-signal engine) |
| **Approval Queue** | `list_pending_approvals`, `get_pending_count`, `approve_item`, `reject_item`, `approve_all` |
| **Content Generation** | `generate_reply`, `generate_tweet`, `generate_thread` (requires LLM provider) |
| **Config & Health** | `get_config`, `validate_config`, `health_check` |

### Machine-Readable CLI Output

All read-only commands also support `--output json` for structured output:

```bash
tuitbot test --output json              # Validate setup
tuitbot settings --show --output json   # View config (secrets redacted)
tuitbot stats --output json             # Analytics dashboard
tuitbot approve --list --output json    # List pending approval items
tuitbot tick --output json              # Tick summary with per-loop status
```

Non-interactive approve commands work without a terminal:

```bash
tuitbot approve --approve 1             # Approve item by ID
tuitbot approve --reject 2              # Reject item by ID
tuitbot approve --approve-all           # Approve all pending
```

---

## Troubleshooting

**Common Fixes:**
* **Auth Errors:** Ensure your X App callback URL is exactly `http://127.0.0.1:8080/callback`. No trailing slashes.
* **No Tweets Found:** Relax your configured keywords or lower the scoring threshold in `tuitbot settings scoring`.
* **Verbose Logging:** If all else fails, use `RUST_LOG=tuitbot_core=debug tuitbot run` to get exhaustive logs.
* **Lock Errors on tick:** If `tuitbot tick` reports a lock error, check that no other `tuitbot run` or `tuitbot tick` process is active. The lock file at `~/.tuitbot/tuitbot.lock` is released automatically, but if the process was killed with `SIGKILL`, you may need to delete it manually.

**Local File Storage:**
All data is stored locally. Total privacy. Total agency.
* `~/.tuitbot/config.toml` — User configuration
* `~/.tuitbot/tokens.json` — Rotating X API tokens
* `~/.tuitbot/tuitbot.db` — Lightweight SQLite database
* `~/.tuitbot/tuitbot.lock` — Process lock (tick mode)

*To confidently reset your entire system, completely purge: `rm -rf ~/.tuitbot`.*

---

<div align="center">
  <p>Built with Rust.</p>
  <p>Released under the <a href="LICENSE">MIT License</a>.</p>
</div>
