<div align="center">

<img src="assets/logo.svg" alt="Tuitbot Logo" width="120" />

# Tuitbot

**Your AI-powered X (Twitter) growth co-pilot.**

[![Rust](https://img.shields.io/badge/built_with-Rust-d32822?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![X API](https://img.shields.io/badge/X_API-Ready-black?style=for-the-badge&logo=x)](https://developer.x.com)
[![CI](https://img.shields.io/github/actions/workflow/status/aramirez087/TuitBot/ci.yml?branch=main&style=for-the-badge&label=CI)](https://github.com/aramirez087/TuitBot/actions/workflows/ci.yml)
[![Releases](https://img.shields.io/github/v/release/aramirez087/TuitBot?style=for-the-badge)](https://github.com/aramirez087/TuitBot/releases)

Tuitbot discovers relevant conversations, drafts genuinely helpful replies, and queues everything for your review. When you're ready, flip the switch to let it post autonomously within strict safety guardrails. **Think of it as a growth-focused writing partner** that handles the research so you can focus on building.

Built for **founders**, **indie hackers**, and **solo makers** who'd rather build their product than spend hours on X.

[Deployment](#three-deployment-modes) · [Features](#features) · [Getting Started](#getting-started) · [Quick Commands](#quick-commands) · [Release Strategy](#release-strategy-ci) · [Full Docs](https://aramirez087.github.io/TuitBot/)

</div>

---

## Three Deployment Modes

Tuitbot supports three ways to run — all using the exact same core automation engine:

| Mode | How it runs | Dashboard access | Best for |
|------|-------------|-----------------|----------|
| **Desktop App** | Native window (Tauri) on macOS/Windows. | Built-in native UI | Users who want a beautiful app with point-and-click settings. |
| **LAN Mode** | Server on a local machine (Pi, NAS, spare laptop). `--host 0.0.0.0` | Browser → `http://<server-ip>:3001` | Access from any device on your network without keeping a laptop open. |
| **Self-Hosted** | `docker compose up` on a VPS. | Browser → `http://localhost:3001` | Technical users who want 24/7 automation and full control. |
| **Cloud Tier**  | Fully managed by us. | Browser → `app.tuitbot.dev` | Zero setup, always-on automation. |

### Desktop App (Native GUI)

The easiest way to use Tuitbot. Built with Tauri and SvelteKit, the desktop app provides a beautiful, data-rich dashboard:

- **Analytics:** 30-day follower charts, engagement stats, and top-performing topics.
- **Visual Approval Queue:** Edit, review, and manually approve AI-generated replies and tweets.
- **Content Calendar:** Schedule threads and tweets visually alongside autonomous content.
- **Target Accounts Manager:** Track relationships and view interaction history.
- **Settings Editor:** Configure your business profile, adjust LLM settings, and manipulate the 6-signal scoring engine without touching a `.toml` file.
- **Strategy Dashboard:** Weekly growth scorecard with follower trends, reply acceptance rates, top/bottom topics, and AI-generated recommendations for what to post more (or less) of.

### CLI & Scheduler-Driven Modes

For power users and self-hosters, Tuitbot still provides the robust `tuitbot-cli`:
- `tuitbot run`: Long-running daemon with internal scheduling.
- `tuitbot tick`: Idempotent execution for external schedulers (cron, systemd, OpenClaw).

---

## Features

Tuitbot runs background loops — discovering conversations, drafting content, and routing everything through your approval queue (or posting automatically in Autopilot mode). You can monitor all of this through the **Dashboard's Live Activity Feed**, which provides real-time visibility into the AI's decision-making process.

### 1. Finds Conversations That Matter
Searches X for tweets matching your product's keywords. Tuitbot uses a **6-signal scoring engine** (configurable via the GUI) to find the perfect interactions:
* **Keyword relevance** · **Follower sweet spot (1K-5K)** · **Recency** · **Engagement rate** · **Reply count** · **Content type**.

### 2. Replies With Genuinely Helpful Content
When it finds a high-scoring tweet, Tuitbot drafts a natural, helpful reply and queues it for your review. In the **Visual Approval Queue**, you can edit, approve, or reject before anything is posted. (In Autopost Mode, replies post automatically within your configured limits.)

### 3. Posts Educational Tweets & Threads
Tuitbot posts original tweets and weekly threads. Through the **Content Calendar**, you can visualize scheduled autonomous posts and compose your own manual tweets to interleave with the AI-generated content.

### 4. Monitors Mentions & Tracks Targets
* Drafts thoughtful replies when someone @-mentions you, queued for your review.
* Uses the **Target Accounts CRM** to monitor specific people, build relationships over time, and visually track interaction history.

### 5. Rich Analytics
Snapshots your follower count daily and measures engagement after 24 hours. The **Analytics Dashboard** visualizes this data with beautiful charts and top-performing topic breakdowns.

### 6. Weekly Strategy Reports
Tuitbot generates a weekly scorecard that aggregates your engagement data into actionable insights. The **Strategy Dashboard** shows your growth loop — inputs, engine, outputs, and metrics — in a single view. An 8-rule recommendation engine automatically identifies winning topics to double down on, underperformers to cut, follower stalls, and engagement regressions.

### 7. Composer Mode: AI-Assisted Writing
When you want to stay in the driver's seat, switch to Composer mode for a hands-on writing experience powered by the same AI engine:
* **AI Assist** — generate tweets, replies, and threads on demand; improve existing drafts with one click.
* **Draft System** — create, edit, schedule, and publish content at your own pace.
* **Discovery Feed** — browse scored conversations and compose replies manually instead of letting the agent queue them.

---

## Two Operating Modes

| Mode | Behavior | Recommended for |
|------|----------|-----------------|
| **Autopilot** (default) | Discovers conversations and drafts content autonomously. Posts automatically when `approval_mode = false`, or queues for review when `approval_mode = true`. | Established accounts that trust the AI's tone. |
| **Composer** | Autonomous loops are disabled. Discovery runs read-only to feed the Discovery Feed. Approval mode is always on. You write and schedule content with AI Assist, drafts, and the Discovery Feed. | New accounts, cautious users, anyone who wants full creative control. |

```toml
# config.toml
mode = "composer"   # or "autopilot" (default)
```

**Start in Composer mode.** Graduate to Autopilot after you've reviewed enough AI-generated content to trust the tone and quality. You can switch between modes at any time without losing drafts, scheduled posts, or approval queue items.

---

## Built-In Safety & Anti-Spam

Tuitbot is engineered to keep your account safe and maintain your reputation:

| Feature | Description |
|---|---|
| **Approval Mode** | Enabled by default — all posts are queued for human review before posting. Always on in Composer mode. |
| **Conservative Limits** | Defaults to **5 replies/day**, **6 tweets/day**, **1 thread/week**. |
| **Anti-Harassment** | Maximum 1 reply per author per day. |
| **Spam Filter** | Automatically blocks replies containing salesy phrases. |
| **Active Hours** | Sleeps completely outside your active hours (default 8 AM - 10 PM). |
| **Human-like Jitter** | Intervals and post delays are randomized (45-180s), matching natural human posting cadence. |
| **Deduplication** | Never replies to the same tweet twice; detects and prevents repetitive phrasing. |
| **Process Lock** | Only one instance runs at a time — no accidental double-posting. |

---

## X Platform Compliance

Tuitbot is designed to operate within X's [Automation Rules](https://help.x.com/en/rules-and-policies/x-automation) and [Developer Agreement](https://developer.x.com/en/developer-terms/agreement-and-policy). The following behaviors are **not implemented and never will be**:

| Forbidden behavior | How Tuitbot prevents it |
|-|-|
| **Auto-follow / auto-like / auto-retweet** | Not in the codebase. Tuitbot only posts original text content. |
| **Trend-jacking** | Discovery uses your configured keywords only — never the Trending API. |
| **Duplicate or near-duplicate content** | The dedup engine (Jaccard similarity >= 0.8) blocks repetitive phrasing across all posts. |
| **Mass-reply patterns** | Hard cap of 5 replies/day (default) and 1 reply per author per day. |
| **Engagement manipulation** | No coordinated behavior, vote manipulation, or artificial amplification. |
| **DM spam** | TuitBot provides typed DM API tools for legitimate conversation management (Admin/Write profiles), but does not send unsolicited bulk DMs. DM mutations are policy-gated and audit-logged. |
| **Autonomous ad spend** | TuitBot provides typed Ads API tools for campaign reads and management (Admin profile only), but does not autonomously create or fund ad campaigns. All Ads mutations require explicit invocation and are audit-logged. |

All generated content is original — created per-request by your configured LLM, not from templates or recycled text.

> **You are responsible** for reviewing Tuitbot's output and ensuring compliance with X's Terms of Service and your local regulations. Tuitbot is a tool, not a compliance guarantee.

---

## Getting Started

### 1. The Desktop App (Recommended)

The easiest way to get started is by downloading the desktop app. You don't need to touch the terminal.

1. Download the latest `.dmg` (macOS), `.exe` (Windows), or `.AppImage` (Linux) from the [Releases](https://github.com/aramirez087/TuitBot/releases) page.
2. Open the app and follow the interactive **Onboarding Wizard**.
3. The app will guide you through connecting your X account, configuring your AI provider (OpenAI, Anthropic, or local Ollama), and setting up your business profile.

The app will run quietly as a system tray icon, discovering and drafting content for your review.

### 2. LAN Mode (Run on a Pi, access from anywhere on your network)

Run the server on any always-on machine and access the dashboard from your phone, tablet, or laptop:

```bash
cargo run -p tuitbot-server -- --host 0.0.0.0
```

The server prints a 4-word passphrase to the terminal on first start. Open `http://<server-ip>:3001` from any device on your network and enter the passphrase to log in. Sessions last 7 days. Full setup guide: [LAN Mode](https://aramirez087.github.io/TuitBot/lan-mode/).

### 3. Self-Hosted Docker

For users who want to run Tuitbot on a cloud VPS (like Hetzner or DigitalOcean) for 24/7 uptime without keeping a laptop open:

```bash
git clone https://github.com/aramirez087/TuitBot.git
cd TuitBot
cp .env.example .env
# Edit .env to add your API keys
docker compose up -d
```
Then navigate to `http://localhost:3001` to access the full graphical dashboard in your browser.

### 4. Command Line Interface (CLI)

For power users who prefer the terminal. **Hello world in under 2 minutes:**

```bash
cargo install tuitbot-cli --locked   # or grab a binary from Releases

tuitbot init                         # guided setup: config → auth → test → preview
```

That's it — one command. `init` asks 5 questions (product name, keywords, LLM provider, API key, X Client ID), validates your LLM connection, then walks you through auth, validation, and a dry-run preview. Everything else gets safe defaults. Customize later with `tuitbot settings` or `tuitbot init --advanced` for full control.

Individual commands (`tuitbot auth`, `tuitbot test`, `tuitbot tick --dry-run`) still work standalone if you prefer step-by-step setup.

---

## Quick Commands

```bash
# Setup (hello world path)
tuitbot init                               # 5-question quickstart
tuitbot init --advanced                    # full 8-step wizard
tuitbot auth                               # OAuth 2.0 with X
tuitbot test                               # validate config + connectivity
tuitbot tick --dry-run                     # preview what the bot would do

# Run
tuitbot run                                # long-running daemon
tuitbot tick --output json                 # single pass for cron/schedulers
tuitbot tick --loops discovery,content     # run specific loops only

# Configure
tuitbot settings                           # interactive editor
tuitbot settings enrich                    # guided profile enrichment
tuitbot settings --show                    # read-only config view

# Operations
tuitbot approve --list                     # review queued posts
tuitbot stats --output json                # analytics snapshot
tuitbot backup                             # back up database
tuitbot update                             # check for updates

# AI agent integration (MCP)
tuitbot mcp serve                          # Write profile (112 tools, default)
tuitbot mcp serve --profile admin          # Admin profile (139 tools)
tuitbot mcp serve --profile api-readonly   # API read-only (45 tools)
tuitbot mcp serve --profile readonly       # Read-only (14 tools)
```

---

## AI Agent Integration (MCP)

Tuitbot includes an MCP server that exposes up to **140 tools** for AI agents (Claude Code, custom agents, etc.), covering the X API v2 public surface plus enterprise APIs (DMs, Ads, Compliance, Stream Rules). Four profiles serve different use cases:

| Profile | Command | Tools | Use Case |
|---------|---------|-------|----------|
| **Write** (default) | `tuitbot mcp serve` | 112 | Full growth co-pilot: reads, writes, DMs, analytics, content gen, approval workflows, generated X API tools |
| **Admin** | `tuitbot mcp serve --profile admin` | 139 | Superset of Write — adds Ads API, Compliance, Stream Rules, and universal request tools |
| **API read-only** | `tuitbot mcp serve --profile api-readonly` | 45 | X API reads + DM reads + utility tools, no mutations |
| **Read-only** | `tuitbot mcp serve --profile readonly` | 14 | Minimal safe surface — utility, config, and health tools only |

Read-only profiles are safe by construction — mutation tools are never registered, not policy-blocked. You get typed schemas, structured errors, rate-limit awareness, retry/backoff, and stable output formats with zero mutation risk.

**Quickest start (interactive — auto-registers with Claude Code):**

```bash
tuitbot mcp setup
```

**One-liner with env vars (non-interactive):**

```bash
claude mcp add -s user -e TUITBOT_X_API__CLIENT_ID=your_client_id tuitbot -- tuitbot mcp serve
```

**Claude Code config (Write — default, recommended):**

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

**Claude Code config (Read-only):**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve", "--profile", "readonly"]
    }
  }
}
```

All tools return structured `{ success, data, error, meta }` envelopes with 28 typed error codes, retryable flags, and per-invocation telemetry. Mutations are policy-gated with approval routing, dry-run mode, and hourly rate limiting.

**Enterprise APIs:** DM tools are available from API-readonly and above. Ads, Compliance, and Stream Rules tools are Admin-only and require separate X API access (Ads API approval, Compliance API access). See the [MCP Reference](https://aramirez087.github.io/TuitBot/mcp-reference/) for profiles, scopes, and safety controls.

---

## Architecture

Tuitbot's core is organized into three layers with strict dependency rules:

| Layer | Module | Role | Dependencies |
|-------|--------|------|-------------|
| **Toolkit** | `core::toolkit/` | Stateless X API utilities (read, write, engage, media) | `&dyn XApiClient` only |
| **Workflow** | `core::workflow/` | Stateful composites (discover, draft, queue, publish, orchestrate) | DB + LLM + Toolkit |
| **Autopilot** | `core::automation/` | Scheduled loops (discovery, mentions, content, threads, analytics) | Workflow + Toolkit |

Every layer only calls the layer below it. Toolkit functions are usable from any context (MCP, CLI, tests) without DB or LLM initialization. Workflow functions compose toolkit calls with state. Autopilot schedules workflow cycles on timers. MCP handlers and HTTP routes are thin adapters over these layers.

Four workspace crates: `tuitbot-core` (all business logic), `tuitbot-cli` (CLI), `tuitbot-mcp` (MCP server, 140 tools), `tuitbot-server` (HTTP/WS API). Full details in [Architecture](https://aramirez087.github.io/TuitBot/architecture/).

---

## Documentation

This README is intentionally concise. The full system docs are on GitHub Pages:

- [Docs Home](https://aramirez087.github.io/TuitBot/) - complete map
- [Getting Started](https://aramirez087.github.io/TuitBot/getting-started/) - install and first run
- [Configuration](https://aramirez087.github.io/TuitBot/configuration/) - config model and production guidance
- [CLI Reference](https://aramirez087.github.io/TuitBot/cli-reference/) - command reference
- [MCP Reference](https://aramirez087.github.io/TuitBot/mcp-reference/) - AI agent integration
- [LAN Mode](https://aramirez087.github.io/TuitBot/lan-mode/) - access the dashboard from any device on your network
- [Composer Mode](https://aramirez087.github.io/TuitBot/composer-mode/) - AI-assisted writing, drafts, and discovery feed
- [Operations](https://aramirez087.github.io/TuitBot/operations/) - deployment and runbook
- [Release and Publishing](https://aramirez087.github.io/TuitBot/release-and-publishing/) - release-plz and crates.io

---

## Release Strategy (CI)

Releases are fully automated in GitHub Actions and follow a release-PR model:

1. Every push to `main` runs `.github/workflows/release.yml`.
2. `release-plz` keeps a release PR open with version/changelog updates (`CHANGELOG.md`).
3. Merging that release PR triggers:
   * crates.io publish for workspace crates (`tuitbot-core`, `tuitbot-mcp`, `tuitbot-cli`)
   * Tag + GitHub release for CLI (`tuitbot-cli-vX.Y.Z`)
   * Cross-platform binary builds for `tuitbot` and `tuitbot-server` (`linux`, `macOS Intel`, `macOS Apple Silicon`, `windows`)
   * Asset uploads + `SHA256SUMS` checksum file
4. If a release contains only library crate updates and no `tuitbot-cli` release, binary asset jobs are skipped automatically.
To keep versioning clean, use Conventional Commit prefixes (`feat:`, `fix:`, `chore:`, etc.) and `!` for breaking changes.
Repository setup required once: enable `Settings -> Actions -> General -> Allow GitHub Actions to create and approve pull requests`.
Set a `CARGO_REGISTRY_TOKEN` secret from crates.io (`Account Settings -> API Tokens`) so CI can publish crates.
Optional but recommended: set a `RELEASE_PLZ_TOKEN` (PAT) secret so workflows also run on release PRs created by automation.

Full details: [Release and Publishing](https://aramirez087.github.io/TuitBot/release-and-publishing/).

---

## Troubleshooting

- Auth issues: callback URL must be exactly `http://127.0.0.1:8080/callback`.
- Lock issues on `tick`: ensure no other `tuitbot run` or `tuitbot tick` process is active.
- Debug logs: `RUST_LOG=tuitbot_core=debug tuitbot run`.
- Full troubleshooting guide: [Troubleshooting](https://aramirez087.github.io/TuitBot/troubleshooting/).

---

<div align="center">
  <p>Built with Rust.</p>
  <p>Released under the <a href="LICENSE">MIT License</a>.</p>
</div>
