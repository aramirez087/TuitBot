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

[Two Modes](#two-operating-modes) · [Features](#features) · [Getting Started](#getting-started) · [Quick Commands](#quick-commands) · [Release Strategy](#release-strategy-ci) · [Full Docs](https://aramirez087.github.io/TuitBot/)

</div>

---

## Three Deployment Modes

Tuitbot supports three ways to run — all using the exact same core automation engine:

| Mode | How it runs | Dashboard access | Best for |
|------|-------------|-----------------|----------|
| **Desktop App** | Native window (Tauri) on macOS/Windows. | Built-in native UI | Users who want a beautiful app with point-and-click settings. |
| **Self-Hosted** | `docker compose up` on a VPS. | Browser → `http://localhost:3001` | Technical users who want 24/7 automation and full control. |
| **Cloud Tier**  | Fully managed by us. | Browser → `app.tuitbot.dev` | Zero setup, always-on automation. |

### Desktop App (Native GUI)

The easiest way to use Tuitbot. Built with Tauri and SvelteKit, the desktop app provides a beautiful, data-rich dashboard:

- **Analytics:** 30-day follower charts, engagement stats, and top-performing topics.
- **Visual Approval Queue:** Edit, review, and manually approve AI-generated replies and tweets.
- **Content Calendar:** Schedule threads and tweets visually alongside autonomous content.
- **Target Accounts Manager:** Track relationships, view interaction history, and monitor warmup progress.
- **Settings Editor:** Configure your business profile, adjust LLM settings, and manipulate the 6-signal scoring engine without touching a `.toml` file.

### CLI & Scheduler-Driven Modes

For power users and self-hosters, Tuitbot still provides the robust `tuitbot-cli`:
- `tuitbot run`: Long-running daemon with internal scheduling.
- `tuitbot tick`: Idempotent execution for external schedulers (cron, systemd, OpenClaw).

---

## Features

Tuitbot runs up to six automated loops while you focus on building. You can monitor all of this through the **Dashboard's Live Activity Feed**, which provides real-time visibility into the AI's decision-making process.

### 1. Finds Conversations That Matter
Searches X for tweets matching your product's keywords. Tuitbot uses a **6-signal scoring engine** (configurable via the GUI) to find the perfect interactions:
* **Keyword relevance** · **Follower sweet spot (1K-5K)** · **Recency** · **Engagement rate** · **Reply count** · **Content type**.

### 2. Replies With Genuinely Helpful Content
When it finds a high-scoring tweet, Tuitbot uses AI to write a natural, helpful reply. With the **Visual Approval Queue**, you can opt to review, edit, and approve these replies manually before they are posted.

### 3. Posts Educational Tweets & Threads
Tuitbot posts original tweets and weekly threads. Through the **Content Calendar**, you can visualize scheduled autonomous posts and compose your own manual tweets to interleave with the AI-generated content.

### 4. Monitors Mentions & Tracks Targets
* Automatically generates thoughtful replies when someone @-mentions you.
* Uses the **Target Accounts CRM** to monitor specific people, build relationships over time, and visually track interaction history.

### 5. Rich Analytics
Snapshots your follower count daily and measures engagement after 24 hours. The **Analytics Dashboard** visualizes this data with beautiful charts and top-performing topic breakdowns.

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

### 1. The Desktop App (Recommended)

The easiest way to get started is by downloading the desktop app. You don't need to touch the terminal.

1. Download the latest `.dmg` (macOS), `.exe` (Windows), or `.AppImage` (Linux) from the [Releases](https://github.com/aramirez087/TuitBot/releases) page.
2. Open the app and follow the interactive **Onboarding Wizard**.
3. The app will guide you through connecting your X account, configuring your AI provider (OpenAI, Anthropic, or local Ollama), and setting up your business profile.

The app will run quietly as a system tray icon, managing the automation in the background.

### 2. Self-Hosted Docker

For users who want to run Tuitbot on a cloud VPS (like Hetzner or DigitalOcean) for 24/7 uptime without keeping a laptop open:

```bash
git clone https://github.com/aramirez087/TuitBot.git
cd TuitBot
cp .env.example .env
# Edit .env to add your API keys
docker compose up -d
```
Then navigate to `http://localhost:3001` to access the full graphical dashboard in your browser.

### 3. Command Line Interface (CLI)

For power users who prefer the terminal, Tuitbot is available as a Rust crate:

```bash
cargo install tuitbot-cli --locked
tuitbot init
tuitbot auth
tuitbot run    # Run daemon
# OR
tuitbot tick   # For cron/planners
```

---

## Quick Commands

```bash
# Core setup / validation
tuitbot init
tuitbot auth
tuitbot test

# Operating modes
tuitbot run
tuitbot tick --output json
tuitbot tick --dry-run

# Operations
tuitbot update
tuitbot stats --output json
tuitbot settings --show --output json
tuitbot approve --list --output json

# AI agent integration
tuitbot mcp serve
```

---

## Documentation

This README is intentionally concise. The full system docs are on GitHub Pages:

- [Docs Home](https://aramirez087.github.io/TuitBot/) - complete map
- [Getting Started](https://aramirez087.github.io/TuitBot/getting-started/) - install and first run
- [Configuration](https://aramirez087.github.io/TuitBot/configuration/) - config model and production guidance
- [CLI Reference](https://aramirez087.github.io/TuitBot/cli-reference/) - command reference
- [MCP Reference](https://aramirez087.github.io/TuitBot/mcp-reference/) - AI agent integration
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
   * Cross-platform binary builds (`linux`, `macOS Intel`, `macOS Apple Silicon`, `windows`)
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
