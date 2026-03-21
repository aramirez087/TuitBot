# TuitBot Ghostwriter — Obsidian Plugin

Send selections and blocks from your Obsidian vault to TuitBot for content generation with full provenance tracking. The plugin captures editor context (heading hierarchy, note title, frontmatter tags) and sends it to your local TuitBot server via a single HTTP POST.

## Prerequisites

- **TuitBot Desktop** running on `http://127.0.0.1:3001` (or a self-hosted instance).
- TuitBot must have been started at least once to generate the API token at `~/.tuitbot/api_token`.
- Obsidian **v1.4.0+** (desktop only — the plugin requires localhost network access).

## Installation

1. Copy the `obsidian-tuitbot` folder into your vault's `.obsidian/plugins/` directory:
   ```
   cp -r plugins/obsidian-tuitbot /path/to/vault/.obsidian/plugins/tuitbot
   ```
2. In Obsidian, go to **Settings → Community Plugins → Installed Plugins** and enable **TuitBot Ghostwriter**.

## Commands

| Command | Trigger | Behavior |
|---|---|---|
| **Send selection to TuitBot** | Select text → Command Palette | Sends the selected text with heading context, note title, and frontmatter tags. |
| **Send current block to TuitBot** | Place cursor in paragraph → Command Palette | Sends the contiguous non-blank lines around the cursor (the "block"). |

Both commands POST to `POST /api/vault/send-selection` with a JSON payload containing the selection, its position, and surrounding context.

## Configuration

Settings are stored in `.obsidian/plugins/tuitbot/data.json`. Edit manually or through the Obsidian plugin settings interface (when available):

```json
{
  "serverUrl": "http://127.0.0.1:3001",
  "apiTokenPath": "~/.tuitbot/api_token"
}
```

| Setting | Default | Description |
|---|---|---|
| `serverUrl` | `http://127.0.0.1:3001` | TuitBot server URL. Change for self-hosted instances. |
| `apiTokenPath` | `~/.tuitbot/api_token` | Path to the API token file. On Windows, `~` resolves to `%USERPROFILE%`. |

## Transport Contract

The full request/response schema, payload examples, error catalog, and backend implementation checklist are documented in [`docs/roadmap/obsidian-ghostwriter-edge/obsidian-plugin-contract.md`](../../docs/roadmap/obsidian-ghostwriter-edge/obsidian-plugin-contract.md).

## Development

```bash
npm install          # install dependencies
npm run build        # bundle to main.js (esbuild)
npm run dev          # watch mode
npm run test         # run unit tests
```

The build produces a single `main.js` file that Obsidian loads directly.

## Current Status

The plugin is the **sender** half of the Ghostwriter selection flow. The server-side `POST /api/vault/send-selection` endpoint does not exist yet — it will be implemented in a later session. Until then, sending a selection will show a connection error notice. The transport contract document bridges to the receiver implementation.
