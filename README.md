# ReplyGuy

Autonomous X (Twitter) growth assistant for founders and indie hackers.

ReplyGuy helps you grow your X account organically by discovering relevant conversations, replying with valuable content, posting educational tweets, and publishing weekly threads -- all on autopilot.

## Features

- **Tweet Discovery** -- Searches for conversations matching your product keywords and scores them for reply-worthiness.
- **Mention Replies** -- Monitors @-mentions and generates contextual replies automatically.
- **Content Generation** -- Posts educational tweets on your industry topics at configured intervals.
- **Thread Publishing** -- Creates and posts multi-tweet threads for deeper thought leadership.
- **Scoring Engine** -- Heuristic scoring (keyword relevance, follower count, recency, engagement) to prioritize high-value conversations.
- **Brand Voice** -- Configurable voice, reply style, and content style so the bot sounds like you, not a bot.
- **Safety Limits** -- Configurable daily/weekly caps and action delays to avoid account restrictions.
- **Multi-Provider LLM** -- Supports OpenAI, Anthropic, and Ollama for content generation.

## Prerequisites

- **Rust 1.75+** (install via [rustup](https://rustup.rs/))
- **X API developer account** ([developer.x.com](https://developer.x.com)) -- see [setup guide](#x-developer-portal-setup) below
- **LLM provider API key** (OpenAI, Anthropic, or local Ollama)

## Installation

```bash
# From source
cargo install --path crates/replyguy-cli

# Or build without installing
cargo build --release
# Binary at: target/release/replyguy
```

## Quickstart

```bash
# 1. Run the interactive setup wizard
replyguy init

# 2. Authenticate with X API
replyguy auth

# 3. Validate your configuration
replyguy test

# 4. Start the agent
replyguy run
```

The wizard walks you through 4 steps: X API credentials, business profile, brand voice, and LLM provider. Defaults shown in `[brackets]` can be accepted by pressing Enter.

For CI/scripting, use `replyguy init --non-interactive` to copy the template config instead.

## X Developer Portal Setup

Before running `replyguy auth`, your X app must be configured correctly. Go to [developer.x.com](https://developer.x.com/en/portal/dashboard) and either create a new app or select an existing one.

### 1. User authentication settings

Under your app's **Settings** tab, click **Set up** under "User authentication settings" and configure:

| Setting | Value |
|---------|-------|
| **App permissions** | Read and write |
| **Type of App** | Native App (public client) |
| **Callback URI / Redirect URL** | `http://127.0.0.1:8080/callback` |
| **Website URL** | Your product URL (e.g. `https://yoursite.com`) |

The callback URI must match **exactly** -- no trailing slash, no `https`, no `localhost`.

### 2. Copy your Client ID

After saving, go to **Keys and tokens** and copy the **OAuth 2.0 Client ID**. You'll paste this during `replyguy init`.

If you chose "Confidential client" instead of "Native App", you'll also need the **Client Secret**. The wizard will ask for it.

### 3. Common errors

| Error | Cause | Fix |
|-------|-------|-----|
| "Something went wrong, you weren't able to give access to the App" | Callback URI not registered or doesn't match | Add `http://127.0.0.1:8080/callback` to your app's redirect URLs |
| "Unauthorized" after pasting the code | Client ID is wrong, or app type mismatch | Double-check the Client ID and app type (Native App vs Web App) |
| "Invalid scopes" | App permissions too restrictive | Set app permissions to "Read and write" |

## CLI Commands

| Command | Description |
|---------|-------------|
| `replyguy init` | Interactive setup wizard (creates `~/.replyguy/config.toml`) |
| `replyguy run` | Start the autonomous agent (all loops) |
| `replyguy auth` | Authenticate with X API (OAuth 2.0 PKCE) |
| `replyguy test` | Validate configuration and connectivity |
| `replyguy discover` | Run one discovery cycle (find + score tweets) |
| `replyguy mentions` | Process pending @-mentions once |
| `replyguy post` | Generate and post a single tweet |
| `replyguy thread` | Generate and post a thread |
| `replyguy score <tweet_id>` | Score a specific tweet |

### Global Flags

- `-c, --config <path>` -- Path to config file (default: `~/.replyguy/config.toml`)
- `-v, --verbose` -- Enable debug-level logging
- `-q, --quiet` -- Suppress all output except errors

Use `RUST_LOG` environment variable for fine-grained log control (e.g., `RUST_LOG=replyguy_core::automation=debug`).

## Configuration

See [`config.example.toml`](config.example.toml) for the full configuration reference with all available options and their defaults.

Key sections:

- **`[x_api]`** -- X API credentials
- **`[business]`** -- Product name, description, keywords, topics, and brand voice
- **`[scoring]`** -- Tweet scoring weights and threshold
- **`[limits]`** -- Daily/weekly action caps and delays
- **`[intervals]`** -- Automation loop timing
- **`[llm]`** -- LLM provider, model, and API key
- **`[storage]`** -- Database path and retention

Configuration can also be set via environment variables with the `REPLYGUY_` prefix (e.g., `REPLYGUY_LLM__API_KEY`).

## License

See [LICENSE](LICENSE) for details.
