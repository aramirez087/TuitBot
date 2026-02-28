# LAN Mode

Run the Tuitbot server on one machine (e.g., an OrangePi, Raspberry Pi, or NAS) and access the dashboard from any device on your local network.

## Quick Start

```bash
# On the server machine
cargo run -p tuitbot-server -- --host 0.0.0.0
```

On first start the server generates a **4-word passphrase** and prints it to the terminal:

```
  Web login passphrase: alpine cobra morning puzzle
  (save this — it won't be shown again)

  Dashboard: http://192.168.1.42:3001
```

Open that URL from any device on your network. You'll see a login screen — enter the passphrase and you're in.

## First-Time Setup (Browser)

When the server starts on `127.0.0.1` (the default) without an existing passphrase,
it **does not** generate one automatically. Instead, the browser-based onboarding
wizard handles passphrase creation:

1. Start the server: `cargo run -p tuitbot-server`
2. Open `http://localhost:3001` in your browser
3. Complete the onboarding wizard
4. On the final step, set your passphrase (a 4-word phrase is suggested)
5. Save the passphrase — you'll need it to log in again after your session expires

The passphrase is created during `POST /api/settings/init` and a session cookie
is set automatically, so you're logged in immediately after onboarding.

**LAN mode (`--host 0.0.0.0`)** still generates a passphrase at startup and
prints it to the terminal, preserving backward compatibility for CLI users.

## How It Works

Tuitbot has two authentication strategies that coexist:

| Mode | Who uses it | How it works |
|------|------------|--------------|
| **Bearer token** | Tauri desktop app, dev mode, API/MCP clients | Reads `~/.tuitbot/api_token` file, sends as `Authorization: Bearer` header |
| **Session cookie** | Web browsers over LAN | Enter passphrase once, server sets an `HttpOnly` cookie valid for 7 days |

When you open the dashboard in a browser without a bearer token on a fresh install, you're directed to the onboarding wizard. At the end of setup, you'll create a passphrase that protects future browser sessions. A session cookie is set automatically, so you're logged in immediately after onboarding. On subsequent visits, if your session has expired, you'll see a login screen where you enter the same passphrase. Sessions last 7 days.

## CLI Flags

```bash
cargo run -p tuitbot-server -- --help
```

| Flag | Default | Description |
|------|---------|-------------|
| `--host` | `127.0.0.1` | Bind address. Use `0.0.0.0` for LAN access |
| `--port` | `3001` | Port number |
| `--config` | `~/.tuitbot/config.toml` | Config file path |
| `--reset-passphrase` | — | Generate a new passphrase and print it |

## Passphrase Management

The passphrase is generated once and its bcrypt hash is stored in `~/.tuitbot/passphrase_hash`. The plaintext is only ever shown in the terminal at generation time.

**Forgot your passphrase?** Reset it:

```bash
cargo run -p tuitbot-server -- --reset-passphrase
```

This prints the new passphrase and invalidates the old one. Existing browser sessions continue working until they expire.

## Security Model

| Concern | Mitigation |
|---------|------------|
| Session theft via XSS | Cookie is `HttpOnly` — JavaScript cannot read it |
| Cross-site request forgery (CSRF) | Mutating requests (POST/PATCH/DELETE) require an `X-CSRF-Token` header that's returned at login |
| Brute-force passphrase guessing | Rate limited to 5 attempts per minute per IP |
| Database compromise | Sessions stored as SHA-256 hashes — a DB dump doesn't leak usable tokens |
| Passphrase exposure | Only printed to terminal once; hash file has `0600` permissions |
| Network sniffing | Use a reverse proxy with TLS for production (see below) |

## Recommended: TLS via Reverse Proxy

LAN mode transmits the passphrase and session cookie in plaintext over HTTP. On a trusted home network this is acceptable, but for anything more exposed, put Tuitbot behind a reverse proxy with TLS.

**Caddy example** (automatic HTTPS with Let's Encrypt):

```
tuitbot.local {
    reverse_proxy localhost:3001
}
```

**nginx example:**

```nginx
server {
    listen 443 ssl;
    server_name tuitbot.local;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location /api/ws {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Upgrading from a Previous Version

If you already have Tuitbot installed, the update is seamless — no manual steps required:

1. `git pull && cargo build` (or update via your package manager)
2. Start the server normally. On first start after the update:
   - The `sessions` table migration runs automatically
   - A passphrase is generated and printed to the terminal
   - `~/.tuitbot/passphrase_hash` is created with `0600` permissions
3. Everything else works exactly as before — Tauri, dev mode, and CLI are unaffected

If you missed the passphrase in the terminal output, reset it:

```bash
cargo run -p tuitbot-server -- --reset-passphrase
```

To start using LAN mode, just add `--host 0.0.0.0` to your server command.

## Tauri Desktop + LAN Mode

The Tauri desktop app always uses bearer token auth (reads `~/.tuitbot/api_token` directly). LAN mode doesn't affect it — the desktop app continues to work exactly as before, even when the server is bound to `0.0.0.0`.

## Troubleshooting

**"Connection refused" from another device**
- Verify the server is bound to `0.0.0.0`, not `127.0.0.1`
- Check firewall rules: port 3001 must be open for TCP

**"Invalid passphrase"**
- Passphrase is case-sensitive and space-separated (4 words if auto-generated)
- You can reset it from the terminal: `tuitbot-server --reset-passphrase`
- Or from Settings → LAN Access → Reset Passphrase (requires an active session)

**Session expired / redirected to login**
- Sessions last 7 days. Log in again with the same passphrase
- If the server was restarted, existing sessions remain valid (they're stored in the database)

**WebSocket not connecting in browser**
- The browser authenticates the WebSocket via the session cookie — no token needed
- Ensure the browser allows cookies for the server's origin
