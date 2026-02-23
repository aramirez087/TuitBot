# Getting Started

## Prerequisites

- X Developer App credentials and funded X API credits (pay-per-usage; see [X API pricing](https://docs.x.com/x-api/getting-started/pricing))
- One LLM provider:
  - OpenAI
  - Anthropic
  - Ollama
- Rust 1.75+ (only needed for source builds)

## Install

### Recommended (crates.io)

```bash
cargo install tuitbot-cli --locked
```

### Prebuilt binary (no Rust toolchain)

```bash
curl -fsSL https://raw.githubusercontent.com/aramirez087/TuitBot/main/scripts/install.sh | bash
```

### From source (contributors)

```bash
cargo install --path crates/tuitbot-cli --locked
```

### Windows

Download the Windows asset from GitHub Releases and add `tuitbot.exe` to `PATH`.

### OS-specific quickstart

#### Linux

```bash
cargo install tuitbot-cli --locked
tuitbot init
tuitbot auth
tuitbot test
```

Recommended runtime: `tuitbot run` under systemd, or `tuitbot tick` via cron/systemd timer.

#### macOS

```bash
cargo install tuitbot-cli --locked
tuitbot init
tuitbot auth
tuitbot test
```

Recommended runtime: `tuitbot run` in launchd/tmux, or `tuitbot tick` via launchd.

#### Windows (PowerShell)

```powershell
# Option A: use release asset and add tuitbot.exe to PATH
# Option B: install from crates.io if Rust is installed
cargo install tuitbot-cli --locked
tuitbot init
tuitbot auth
tuitbot test
```

Recommended runtime: `tuitbot tick` via Task Scheduler.

If `tuitbot` is not found after install, add Cargo bin to PATH:

- Linux/macOS: `$HOME/.cargo/bin`
- Windows: `%USERPROFILE%\\.cargo\\bin`

## First-Time Setup

1. Create an app in the X Developer Portal.
2. Configure callback URI exactly as `http://127.0.0.1:8080/callback`.
3. Run initialization:

```bash
tuitbot init
```

4. Authenticate and validate:

```bash
tuitbot auth
tuitbot test
```

## Choose an Execution Mode

### Mode A: Daemon

```bash
tuitbot run
```

Use this mode on a VPS or always-on host.

### Mode B: External scheduler

```bash
tuitbot tick --output json
```

Use this mode with cron/systemd/launchd/OpenClaw.

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
