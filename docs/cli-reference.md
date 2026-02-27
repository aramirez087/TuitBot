# CLI Reference

## Global Options

```bash
tuitbot [OPTIONS] <COMMAND>

Options:
  -c, --config <PATH>       Path to config.toml (default: ~/.tuitbot/config.toml)
  -v, --verbose              Enable debug-level logging
  -q, --quiet                Suppress output except errors
      --output <FORMAT>      Output format: text or json (default: text)
```

## Setup Commands

### init — Create configuration

```bash
tuitbot init                     # 5-question quickstart (default)
tuitbot init --advanced          # full 8-step wizard
tuitbot init --non-interactive   # copy template config for manual editing
tuitbot init --force             # overwrite existing config file
```

**Quickstart** (default) asks 5 questions: product name, keywords, LLM provider, API key, and X Client ID. Safe defaults are applied for everything else.

**Advanced** runs the full 8-step wizard: X API credentials, business profile, brand voice, persona, target accounts, approval mode, schedule, and LLM provider.

**Non-interactive** writes a template `config.toml` for manual editing or scripted environments.

### auth — Authenticate with X

```bash
tuitbot auth                        # default: manual paste mode (headless-safe)
tuitbot auth --mode local_callback  # start local HTTP server + open browser
tuitbot auth --mode manual          # explicit manual paste mode
```

In manual mode, a URL is printed for you to open in any browser. After authorizing, paste the callback URL back. In `local_callback` mode, a local server handles the redirect automatically. Headless environments automatically fall back to manual mode.

### test — Validate configuration and connectivity

```bash
tuitbot test                  # text output
tuitbot test --output json    # structured JSON output
```

Runs diagnostic checks across configuration, auth, LLM, and database:

| Check | What it validates |
|-------|-------------------|
| Configuration | Config file loads and passes validation |
| Business profile | Product name and keywords are set |
| X API token | OAuth token exists and is not expired |
| X API refresh | Refresh token is present for auto-renewal |
| X API scopes | All required API scopes are granted |
| LLM provider | Provider is known and API key is set (if required) |
| Database | Database path is accessible |
| LLM connectivity | Provider is reachable (network check) |

Reports enrichment status and next-step guidance on success.

## Run Commands

### run — Start the daemon

```bash
tuitbot run                          # start all automation loops
tuitbot run --status-interval 300    # log status summary every 5 minutes
```

Runs continuously until stopped with Ctrl+C or SIGTERM. Spawns all enabled automation loops with internal scheduling, jitter, and active-hours enforcement.

### tick — Single-pass execution

```bash
tuitbot tick                                       # run all loops once
tuitbot tick --dry-run                             # preview without posting
tuitbot tick --loops discovery,content,analytics   # run specific loops only
tuitbot tick --ignore-schedule                     # skip active-hours check
tuitbot tick --require-approval                    # force approval mode for this tick
tuitbot tick --output json                         # structured JSON output
```

Designed for external schedulers (cron, systemd timers, launchd). Acquires a process lock to prevent concurrent ticks.

**Available loops:** `analytics`, `discovery`, `mentions`, `target`, `content`, `thread`

## Configuration Commands

### settings — View and edit configuration

```bash
tuitbot settings                   # interactive settings editor
tuitbot settings --show            # read-only config view
tuitbot settings --set KEY=VALUE   # set a value directly

# Jump to a specific category:
tuitbot settings voice             # brand voice & writing styles
tuitbot settings persona           # opinions, experiences, content pillars
tuitbot settings targets           # target accounts & competitor keywords
tuitbot settings business          # product description & keywords
tuitbot settings schedule          # timezone & active hours
tuitbot settings limits            # safety & rate limit settings
```

### settings enrich — Guided profile enrichment

```bash
tuitbot settings enrich
```

Walks you through three enrichment stages (Voice, Persona, Targeting) to improve content quality. Each stage is optional and can be completed at any time.

### Operating mode

```bash
tuitbot settings --set mode=composer     # AI-assisted writing, you control posting
tuitbot settings --set mode=autopilot    # full autonomous operation
```

Or via environment variable:

```bash
TUITBOT_MODE=composer tuitbot run
```

In Composer mode, autonomous loops are disabled. Discovery runs read-only. Approval mode is always on. See the [Composer Mode guide](composer-mode.md) for details.

## Operations Commands

### approve — Review queued posts

```bash
tuitbot approve                      # interactive review (one by one)
tuitbot approve --list               # list pending items
tuitbot approve --approve <ID>       # approve a specific item
tuitbot approve --reject <ID>        # reject a specific item
tuitbot approve --approve-all        # approve all pending items
```

### stats — Analytics snapshot

```bash
tuitbot stats                   # terminal display
tuitbot stats --output json     # structured JSON output
```

### backup — Database backup

```bash
tuitbot backup                             # backup to default location
tuitbot backup --output-dir /custom/path   # custom backup directory
tuitbot backup --list                      # list existing backups
tuitbot backup --prune 5                   # keep 5 most recent, delete rest
```

### restore — Restore from backup

```bash
tuitbot restore /path/to/backup.tar.gz                    # restore with confirmation
tuitbot restore /path/to/backup.tar.gz --force             # skip confirmation
tuitbot restore /path/to/backup.tar.gz --validate-only     # check without restoring
```

### update — Check for updates

```bash
tuitbot update                    # interactive update check + config upgrade
tuitbot update --check            # check only, don't install
tuitbot update --config-only      # upgrade config schema only
tuitbot update --non-interactive  # skip all prompts
```

## MCP Server

```bash
tuitbot mcp serve                          # Write profile (112 tools, default)
tuitbot mcp serve --profile admin          # Admin profile (139 tools — Ads, Compliance, Stream, universal request)
tuitbot mcp serve --profile api-readonly   # API read-only (45 tools — includes DM reads)
tuitbot mcp serve --profile readonly       # Read-only (14 tools)
tuitbot mcp manifest                       # emit tool manifest JSON (write)
tuitbot mcp manifest --profile admin       # emit manifest for a profile
```

See the [MCP Reference](mcp-reference.md) for tool details.

## Output Modes

Most read-only commands support `--output json` for machine-readable output:

```bash
tuitbot test --output json
tuitbot tick --output json
tuitbot stats --output json
tuitbot settings --show --output json
tuitbot approve --list --output json
```

## Environment Variables

Override any config value with `TUITBOT_` prefix and `__` (double underscore) as section separator:

```bash
export TUITBOT_X_API__CLIENT_ID=your_client_id
export TUITBOT_LLM__API_KEY=sk-your-key
export TUITBOT_LLM__PROVIDER=openai
export TUITBOT_MODE=composer
```

Precedence: CLI flags > environment variables > `config.toml` > built-in defaults.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Check failed, config error, or authentication error |
