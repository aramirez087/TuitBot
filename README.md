# ReplyGuy

Autonomous X (Twitter) growth assistant for founders and indie hackers.

ReplyGuy helps you grow your X account organically by discovering relevant conversations, replying with valuable content, posting educational tweets, and publishing weekly threads -- all on autopilot.

## Features

- **Tweet Discovery** -- Searches for conversations matching your product keywords and scores them for reply-worthiness.
- **Mention Replies** -- Monitors @-mentions and generates contextual replies automatically.
- **Content Generation** -- Posts educational tweets on your industry topics at configured intervals.
- **Thread Publishing** -- Creates and posts multi-tweet threads for deeper thought leadership.
- **Scoring Engine** -- Heuristic scoring (keyword relevance, follower count, recency, engagement) to prioritize high-value conversations.
- **Safety Limits** -- Configurable daily/weekly caps and action delays to avoid account restrictions.
- **Multi-Provider LLM** -- Supports OpenAI, Anthropic, and Ollama for content generation.

## Prerequisites

- **Rust 1.75+** (install via [rustup](https://rustup.rs/))
- **X API developer account** ([developer.x.com](https://developer.x.com))
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
# 1. Copy the example config
cp config.example.toml ~/.replyguy/config.toml

# 2. Edit the config: add your X API client_id, LLM API key,
#    and customize the business profile for your product.

# 3. Authenticate with X API
replyguy auth

# 4. Validate your configuration
replyguy test

# 5. Start the agent
replyguy run
```

## CLI Commands

| Command | Description |
|---------|-------------|
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
- **`[business]`** -- Product name, keywords, and topics
- **`[scoring]`** -- Tweet scoring weights and threshold
- **`[limits]`** -- Daily/weekly action caps
- **`[intervals]`** -- Automation loop timing
- **`[llm]`** -- LLM provider and model
- **`[storage]`** -- Database path and retention

Configuration can also be set via environment variables with the `REPLYGUY_` prefix (e.g., `REPLYGUY_LLM__API_KEY`).

## License

See [LICENSE](LICENSE) for details.
