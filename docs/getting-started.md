# Getting Started

## Prerequisites

### X Developer App (takes ~2 minutes)

1. Go to [developer.x.com/en/portal/dashboard](https://developer.x.com/en/portal/dashboard)
2. Create a Project and App (or select an existing one)
3. Under "User authentication settings", enable OAuth 2.0:
   - Type: **Web App**
   - Callback URL: `http://127.0.0.1:8080/callback`
4. Copy the **Client ID** from the "Keys and tokens" tab

You also need funded X API credits (pay-per-usage; see [X API pricing](https://docs.x.com/x-api/getting-started/pricing)).

### LLM Provider (pick one)

- **OpenAI** — requires API key ([platform.openai.com](https://platform.openai.com))
- **Anthropic** — requires API key ([console.anthropic.com](https://console.anthropic.com))
- **Ollama** — free, runs locally, no API key needed ([ollama.com](https://ollama.com))

### Rust (source builds only)

Rust 1.75+ is only needed if building from source. Precompiled binaries are available on the [Releases](https://github.com/aramirez087/TuitBot/releases) page.

## 1. Desktop App (Recommended for most users)

1. Download the latest `.dmg` (macOS), `.exe` (Windows), or `.AppImage` (Linux) from the [Releases](https://github.com/aramirez087/TuitBot/releases) page.
2. Open the app and follow the **Onboarding Wizard**.
3. The app will guide you through connecting your X account, choosing an AI provider, and setting up your business profile.

The app runs quietly as a system tray icon, discovering conversations and drafting content for your review.

## 2. Docker Self-Hosted

For 24/7 automation on a VPS:

```bash
git clone https://github.com/aramirez087/TuitBot.git
cd TuitBot
cp .env.example .env
# Edit .env with your API keys
docker compose up -d
```

Navigate to `http://localhost:3001` for the full web dashboard.

## 3. CLI — Hello World in Under 2 Minutes

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | sh
```

Or install from source:

```bash
cargo install tuitbot-cli --locked
```

Precompiled binaries are also available on the [Releases](https://github.com/aramirez087/TuitBot/releases) page.

### Run the setup wizard

```bash
tuitbot init
```

This single command handles everything:

1. **5 quick questions** — product name, keywords, LLM provider, API key, X Client ID
2. **LLM validation** — confirms your AI provider is reachable
3. **X authentication** — OAuth flow to connect your account
4. **Configuration check** — verifies everything works
5. **Dry-run preview** — shows what the bot would do (no posts)

The wizard includes inline guidance at the X API Client ID step (the same 4-step guide from [Prerequisites](#x-developer-app-takes-2-minutes) above).

Everything else gets safe defaults (UTC timezone, approval mode on, conservative rate limits).

> **Tip:** If you run any `tuitbot` command without a config file, it will offer to launch the setup wizard automatically.

### What you'll see

```
Tuitbot Quick Setup
───────────────────
Before we start, have these ready:
  • X API Client ID  — from https://developer.x.com
  • An LLM API key   — OpenAI, Anthropic, or Ollama (free, local)

5 questions to get you running. Use --advanced for full configuration.

? Product name: YourApp
? Discovery keywords: rust, cli, devtools
? LLM provider: openai
? OpenAI API key: sk-...
  ✓ Connected to openai (gpt-4o-mini, 340ms)
? X API Client ID: abc123...

Configuration Summary
─────────────────────
  Product:     YourApp
  Keywords:    rust, cli, devtools
  LLM:         openai (gpt-4o-mini)
  X API:       abc123... (Client ID set)
  Approval:    on (all posts queued for review)

? Save configuration? [Y/n]
? Connect your X account now? [Y/n]
? Verify everything works? [Y/n]
? Preview the bot? (dry run, no posts) [y/N]
```

### Standalone commands

Each step also works as a standalone command if you prefer:

```bash
tuitbot auth                         # authenticate with X
tuitbot test                         # verify everything works
tuitbot tick --dry-run               # see the bot in action (no posts)
```

### Choose your run mode

| Mode | Command | Best for |
|------|---------|----------|
| **Daemon** | `tuitbot run` | Long-running process (tmux, VPS, systemd) |
| **Scheduler** | `tuitbot tick` | External schedulers (cron, launchd, systemd timer) |

---

## 4. MCP-Only Setup (Claude Code, Cursor, etc.)

If you only need Tuitbot as an MCP server for your AI coding assistant (no LLM or business profile required):

```bash
tuitbot mcp setup
```

This streamlined wizard handles:

1. **X API Client ID** — with inline guide
2. **OAuth authentication** — connects your X account
3. **Profile selection** — write, readonly, or admin
4. **Auto-registration** — detects Claude Code and runs `claude mcp add` for you

For non-interactive environments, use env vars directly:

```bash
claude mcp add -s user -e TUITBOT_X_API__CLIENT_ID=your_client_id tuitbot -- tuitbot mcp serve
```

See the [MCP Reference](https://aramirez087.github.io/TuitBot/mcp-reference/) for profiles, tool counts, and configuration details.

---

## Progressive Enrichment (Optional)

The quickstart gets you running fast with safe defaults. When you're ready for better results, enrich your profile in stages:

```bash
tuitbot settings enrich
```

This walks you through three stages — complete them in any order, whenever you want:

| Stage | What it configures | Why it matters |
|-------|--------------------|----------------|
| **Voice** | Brand voice, reply style, content style | Shapes every LLM-generated reply and tweet |
| **Persona** | Opinions, experiences, content pillars | Makes content authentic and distinctive |
| **Targeting** | Target accounts, competitor keywords | Focuses discovery on high-value conversations |

You can also jump directly to any stage:

```bash
tuitbot settings voice
tuitbot settings persona
tuitbot settings targets
```

Check your enrichment status anytime:

```bash
tuitbot test
# Shows: Profile: Voice OK  Persona --  Targeting --
```

### Full setup wizard

If you prefer to configure everything upfront in one pass:

```bash
tuitbot init --advanced
```

This runs the full 8-step wizard covering X API credentials, business profile, brand voice, persona, target accounts, approval mode, schedule, and LLM provider.

### Non-interactive setup

For CI/CD, Docker, or scripted environments:

```bash
tuitbot init --non-interactive
```

This copies a template `config.toml` that you edit manually. Pair with environment variables for secrets:

```bash
export TUITBOT_X_API__CLIENT_ID=your_client_id
export TUITBOT_LLM__API_KEY=sk-your-key
```

## Updating

```bash
tuitbot update
```

Checks for new releases, updates the CLI binary (and `tuitbot-server` if installed on `PATH`), and upgrades your config with any new settings.

## Validation

After any config change, verify with:

```bash
tuitbot test                    # full diagnostic check
tuitbot settings --show         # read-only config view
```
