# Quickstart — First Reply in 5 Minutes

> **What you'll have:** TuitBot installed, your X account connected, and a real AI-generated reply draft ready to post — in under 5 minutes.

## Prerequisites

You'll need these before you start:

| Requirement | Where to get it |
|-------------|----------------|
| X Developer App (free tier) | [developer.x.com/en/portal/dashboard](https://developer.x.com/en/portal/dashboard) → Create a project/app → **Client ID** from the Keys tab |
| OAuth 2.0 callback URL configured | In your app settings: `http://127.0.0.1:8080/callback` |
| One LLM API key (pick one) | [OpenAI](https://platform.openai.com) · [Anthropic](https://console.anthropic.com) · or [Ollama](https://ollama.com) (free, local) |

---

## Step 1 — Install

**Option A: One-line installer (macOS/Linux)**
```bash
curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | sh
```

**Option B: Cargo**
```bash
cargo install tuitbot-cli --locked
```

**Option C: Precompiled binary**  
Download from [github.com/aramirez087/TuitBot/releases](https://github.com/aramirez087/TuitBot/releases) → unzip → move to your PATH.

---

## Step 2 — Run the setup wizard

```bash
tuitbot init
```

The wizard asks 5 questions, then handles everything else:

1. **Your product/project name** — used to focus content strategy
2. **Keywords** — topics you want to engage with (e.g. `rust, indie hacking, saas`)
3. **LLM provider** — choose `openai`, `anthropic`, or `ollama`
4. **LLM API key** — paste your key (or press Enter for Ollama)
5. **X Client ID** — paste the Client ID from your X Developer App

After answering, `tuitbot init` will:
- Validate your LLM connection
- Open a browser window for X OAuth login (click **Authorize**)
- Run a dry-run check (no posts yet)

> **Tip:** If you don't have an X Developer App yet, `tuitbot init` shows step-by-step guidance when you reach the Client ID question. It takes about 2 minutes on [developer.x.com](https://developer.x.com/en/portal/dashboard).

---

## Step 3 — Discover tweets

```bash
tuitbot discover
```

TuitBot searches for conversations matching your keywords and scores them by relevance. You'll see output like:

```
Discovering conversations...
Found 8 relevant tweets (score ≥ 50):
  [92] @buildinpublic: "Shipped v1 of my Rust CLI today after 3 months..."
  [87] @indiemakers: "What's your current stack for SaaS side projects?"
  [74] @rustlang: "New async features in Rust 2025 edition..."
  ...
```

---

## Step 4 — Generate your first reply

```bash
tuitbot compose
```

TuitBot picks the highest-scored discovery item, drafts a genuine reply using your LLM, and shows it to you:

```
Composing reply to @buildinpublic...

Draft:
  Congrats on shipping! What was the hardest part of the Rust async story
  for you? I'm working through similar challenges right now.

[a] approve  [e] edit  [r] reject  [q] quit
```

Press **a** to add it to the approval queue, **e** to edit, or **r** to skip this one.

---

## Step 5 — Approve and post (optional)

```bash
tuitbot approve
```

Review everything in your queue before it goes live. Approve individually (`a`) or in bulk (`--all`):

```bash
tuitbot approve --all   # approve everything in queue
```

> **Safe by default:** Nothing posts until you approve it. To enable fully autonomous posting later, see [Autonomous Mode](configuration.md#autonomous-posting).

---

## What's next?

| I want to… | Where to go |
|------------|-------------|
| Use the visual dashboard | [Dashboard guide](composer-mode.md) |
| Run 24/7 on a server | [Self-hosted setup](getting-started.md#2-docker-self-hosted) |
| Connect to Claude Code / Cursor | [MCP Setup](../README.md#mcp-setup) |
| Tune keywords and strategy | [Configuration](configuration.md) |
| See all CLI commands | [CLI Reference](cli-reference.md) |

---

*Having trouble? Check [Troubleshooting](troubleshooting.md) or open an issue on [GitHub](https://github.com/aramirez087/TuitBot/issues).*
