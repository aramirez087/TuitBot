# Getting Started

## Prerequisites

- X Developer App credentials ([create one here](https://developer.x.com)) and funded X API credits (pay-per-usage; see [X API pricing](https://docs.x.com/x-api/getting-started/pricing))
- One LLM provider:
  - **OpenAI** — requires API key
  - **Anthropic** — requires API key
  - **Ollama** — free, runs locally, no API key needed
- Rust 1.75+ (only needed for source builds)

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
cargo install tuitbot-cli --locked
```

Precompiled binaries are also available on the [Releases](https://github.com/aramirez087/TuitBot/releases) page.

### Step 1: Initialize (5 questions)

```bash
tuitbot init
```

The quickstart wizard asks only **5 questions**:

1. Product name
2. Discovery keywords (comma-separated)
3. LLM provider (openai, anthropic, or ollama)
4. API key (skipped for ollama)
5. X API Client ID

Everything else gets safe defaults (UTC timezone, approval mode on, conservative rate limits). You'll see:

```
Configuration Summary
─────────────────────
  Product:     YourApp
  Keywords:    rust, cli, devtools
  LLM:         openai (gpt-4o-mini)
  X API:       abc123... (Client ID set)
  Approval:    on (all posts queued for review)

  Defaults applied: UTC schedule, no brand voice, no persona.
  Customize later: tuitbot init --advanced  or  tuitbot settings

Wrote ~/.tuitbot/config.toml

Get started:
  1. tuitbot auth           — connect your X account
  2. tuitbot test           — verify everything works
  3. tuitbot tick --dry-run — see the bot in action (no posts)
```

### Step 2: Authenticate

```bash
tuitbot auth
```

Opens a browser flow (or prints a URL for headless environments). Paste the callback URL when prompted:

```
Authenticated as @yourhandle. Tokens saved to ~/.tuitbot/tokens.json
```

### Step 3: Validate

```bash
tuitbot test
```

Runs diagnostic checks across configuration, auth, LLM, and database:

```
Configuration:    OK (loaded from ~/.tuitbot/config.toml)
Business profile: OK (product_name: "YourApp", 3 keywords, 3 topics)
X API token:      OK (valid, expires in 30d)
X API refresh:    OK (refresh token present)
X API scopes:     OK (all required scopes granted)
LLM provider:     OK (openai, model: gpt-4o-mini)
Database:         OK (will be created at ~/.tuitbot/tuitbot.db)
LLM connectivity: OK (openai: reachable)

All checks passed.

Profile: Voice --  Persona --  Targeting --
Tip: Run `tuitbot settings enrich` to configure voice
     (shapes every LLM-generated reply and tweet)

Ready! Try: tuitbot tick --dry-run
```

### Step 4: First dry run

```bash
tuitbot tick --dry-run
```

This runs every automation loop **without posting anything**. You'll see what the bot would do — tweets it would discover, replies it would draft, content it would generate. No posts are made.

```
Dry run: showing what the bot would do. No posts will be made.

tuitbot tick  tier=Free  schedule=active  dry_run=true  approval_mode=true

  analytics    OK     followers=142, replies_measured=0, tweets_measured=0
  discovery    OK     found=12, qualifying=3, replied=0, skipped=3, failed=0
  ...

Result: success

Tip: Run `tuitbot settings enrich` to configure voice
     — shapes every LLM-generated reply and tweet
```

You're done. From here, choose how to run Tuitbot:

| Mode | Command | Best for |
|------|---------|----------|
| **Daemon** | `tuitbot run` | Long-running process (tmux, VPS, systemd) |
| **Scheduler** | `tuitbot tick` | External schedulers (cron, launchd, systemd timer) |

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
