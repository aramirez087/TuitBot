# Getting Started

## Prerequisites

- X Developer App credentials and funded X API credits (pay-per-usage; see [X API pricing](https://docs.x.com/x-api/getting-started/pricing))
- One LLM provider:
  - OpenAI
  - Anthropic
  - Ollama
- Rust 1.75+ (only needed for source builds)

## 1. Desktop App (Recommended for most users)

The native GUI desktop application is the easiest way to get up and running. 

1. Head to the [GitHub Releases](https://github.com/aramirez087/TuitBot/releases) page.
2. Download the installer for your OS (macOS `.dmg`, Windows `.exe`, or Linux `.AppImage`).
3. Run the installer and launch the app.
4. You will be greeted by the **Onboarding Wizard**. Follow the steps to connect your X developer account (or authenticate via OAuth) and set your business profile. Tuitbot will run in the background as a system tray app.

## 2. Docker Self-Hosted

If you prefer to keep Tuitbot running on a remote server/VPS for 24/7 automation:

```bash
git clone https://github.com/aramirez087/TuitBot.git
cd TuitBot
cp .env.example .env
```

Edit your `.env` file with your X API Credentials, your LLM provider keys, and then boot the container:

```bash
docker compose up -d
```

Head to `http://localhost:3001` or your server's IP. Tuitbot's fully functional web dashboard will walk you through the rest. 

## 3. Command Line Interface (CLI)

For integration with other scripts or users who prefer strict terminal output.

**Install via crates.io:**
```bash
cargo install tuitbot-cli --locked
```
*(Precompiled binaries exist on the Releases page as well).*

**Initialization:**
```bash
tuitbot init
tuitbot auth
tuitbot test
```

**Choose an Execution Mode:**

- `tuitbot run` (Daemon) - runs on a continuous interval. Best for tmux/VPS.
- `tuitbot tick --output json` (Scheduler) - runs one pass. Best for Cron/launchd.

## Updating

```bash
tuitbot update
```

This checks for a new release, updates the binary, and upgrades your config with any new feature groups. Run it periodically to stay current.

## Health Check

```bash
tuitbot health
tuitbot stats --output json
```
