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

> **Note on X API access:** X API access is pay-per-usage (credits). Discovery, replies, mentions, target monitoring, and posting all require available credits. Legacy Basic/Pro subscriptions may still exist for older accounts. See [Getting Started](https://aramirez087.github.io/TuitBot/getting-started/) for setup details.

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
| **X API Developer Account** | [developer.x.com](https://developer.x.com) | Pay-per-usage (credits) |
| **AI Provider** | [OpenAI](https://platform.openai.com/), [Anthropic](https://console.anthropic.com/), or [Ollama](https://ollama.ai/) | Varies (Ollama is Free) |

### Install

**From crates.io (recommended):**
```bash
cargo install tuitbot-cli --locked
```

**Prebuilt binary (no Rust toolchain):**
```bash
curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | bash
```

**From source (contributors, Rust 1.75+):**
```bash
cargo install --path crates/tuitbot-cli --locked
```

**Windows:** download `tuitbot-x86_64-pc-windows-msvc.zip` from [Releases](https://github.com/aramirez087/TuitBot/releases), unzip, and add `tuitbot.exe` to your `PATH`.

### OS-specific quickstart

| OS | Install path | First run | Recommended mode |
|---|---|---|---|
| **Linux** | `cargo install tuitbot-cli --locked` or install script | `tuitbot init && tuitbot auth && tuitbot test` | `tuitbot run` under systemd, or `tuitbot tick` via cron/systemd timer |
| **macOS** | `cargo install tuitbot-cli --locked` or install script | `tuitbot init && tuitbot auth && tuitbot test` | `tuitbot run` in launchd/tmux, or `tuitbot tick` via launchd |
| **Windows** | Release zip + add `tuitbot.exe` to `PATH` (or `cargo install tuitbot-cli --locked`) | `tuitbot init`, then `tuitbot auth`, then `tuitbot test` in PowerShell | `tuitbot tick` via Task Scheduler (or run `tuitbot run` in a persistent terminal) |

If `tuitbot` is not found after `cargo install`, add the cargo bin directory to `PATH`:

- Linux/macOS: `$HOME/.cargo/bin`
- Windows: `%USERPROFILE%\\.cargo\\bin`

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

Use your scheduler to invoke `tuitbot tick --output json` every 15-30 minutes.

For full production examples (cron, systemd, launchd, OpenClaw), see:

- [Getting Started](https://aramirez087.github.io/TuitBot/getting-started/)
- [Operations](https://aramirez087.github.io/TuitBot/operations/)
- [MCP Reference](https://aramirez087.github.io/TuitBot/mcp-reference/)

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
