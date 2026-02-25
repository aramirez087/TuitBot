# 11 — Cloud Hosted Tier (Plausible Model)

> **Goal:** Add a paid cloud-hosted option alongside the existing desktop app.
> Users who don't want to self-host pay $19-29/mo for a managed instance with
> always-on automation. This is the Plausible model: open-source core, paid cloud,
> self-hostable.

## Prerequisites

- Tasks 01-10 completed: desktop app is shipping, users are paying for it, you have
  product-market fit signal. Do NOT start this task until you have paying desktop
  customers confirming the product works.

## Context

The desktop app (Phase 1) validated the product. Now we're adding a cloud tier
where you run the infrastructure and users get:

- Always-on automation (no laptop-sleeping problem)
- Zero setup (no API key management for X — you provide the OAuth app)
- Web dashboard (same UI, no Tauri/install needed)
- Managed updates (always on latest version)

Users who want control keep self-hosting. Everyone else pays monthly.

The key architectural shift: `tuitbot-server` goes from single-user localhost
to multi-tenant cloud deployment. The core crate stays untouched.

## Three deployment modes

After this task, the product supports three ways to run — all using the same
codebase. The server and dashboard are always the same; only the deployment
context changes.

| Mode | How it runs | Dashboard access | Auth model | Who it's for |
|------|-------------|-----------------|------------|--------------|
| **Desktop app** (Tauri) | Tauri embeds the server + frontend in a native window | Native window on their machine | Local file token (task 02) | Users who want a native Mac/Windows app |
| **Self-hosted Docker** | `docker compose up` on a VPS | Browser → `https://tuitbot.myserver.com` | Single-user, local file token (no Stripe, no signup) | Technical users who want always-on + full control |
| **Cloud hosted** | You run the infra at `app.tuitbot.dev` | Browser → `https://app.tuitbot.dev` | Multi-tenant: email/password + Stripe billing | Everyone else — zero setup, monthly subscription |

### How Docker self-hosting works (the evangelist path)

The Docker image bundles **everything** — the Rust API server, the automation
runtime, AND the compiled Svelte dashboard as static files. There is nothing
else to install. The flow:

```
1. Get a $5/mo VPS (Hetzner, DigitalOcean, whatever)
2. git clone the repo (or docker pull tuitbot/tuitbot)
3. cp .env.example .env   # add your X API + LLM keys
4. docker compose up -d
5. Open browser → https://tuitbot.myserver.com
6. Walk through the onboarding wizard (same as task 10)
7. Done — automation runs 24/7, dashboard in the browser.
```

No Tauri install, no Node.js, no Rust toolchain. One container, one port.

The server serves the dashboard frontend as static files at the root path:

```rust
// In tuitbot-server, when TUITBOT_DASHBOARD_DIR is set:
let dashboard_dir = std::env::var("TUITBOT_DASHBOARD_DIR").ok();
if let Some(dir) = dashboard_dir {
    router = router.fallback_service(
        ServeDir::new(&dir)
            .fallback(ServeFile::new(format!("{dir}/index.html")))
    );
}
```

When `TUITBOT_DASHBOARD_DIR` is set (Docker/cloud), the server serves the frontend.
When it's not set (desktop app), Tauri serves the frontend natively. Same server
code, zero conditional logic in the routes.

**Self-host mode vs cloud mode:** The server takes a `--mode` flag:

- `tuitbot-server --mode self-host` (default) — single user, local file token
  auth from task 02, no Stripe, no signup page. Identical to the desktop
  experience but accessed via browser.
- `tuitbot-server --mode cloud` — multi-tenant, user accounts, Stripe billing,
  managed X API OAuth. This is what runs at `app.tuitbot.dev`.

Self-hosters never touch multi-tenant code. The mode flag gates which auth
middleware and route set is loaded.

## What to build

### 1. Multi-tenant architecture

The simplest multi-tenant model for tuitbot: **one SQLite database per user**.
No shared tables, no row-level isolation complexity. Each user gets their own
`{user_id}.db` file and their own config. This matches the desktop model exactly —
each "tenant" is identical to a self-hosted instance.

Create `crates/tuitbot-server/src/tenant.rs`:

```rust
pub struct TenantManager {
    data_dir: PathBuf,  // e.g., /data/tenants/
    tenants: DashMap<UserId, Arc<TenantState>>,
}

pub struct TenantState {
    pub db: DbPool,
    pub config: TuitbotConfig,
    pub runtime_handle: Option<RuntimeHandle>,
    pub event_tx: broadcast::Sender<WsEvent>,
}
```

- On user login: load or create their tenant state
- Each tenant has its own DB pool, config, and automation runtime
- Idle tenants (no WebSocket connections, no dashboard open) keep the runtime
  running but drop the DB pool after timeout (reopen on next request)

### 2. User authentication

Replace the local file token auth with real user accounts.

#### Option A: Self-built (simpler, fewer dependencies)
- `users` table in a shared management database (separate from per-tenant DBs)
- Email + password with argon2 hashing
- Session tokens (JWT or opaque tokens in Redis/SQLite)
- Email verification on signup
- Password reset flow

#### Option B: Auth provider (faster, more features)
- Use Clerk, Auth0, or Supabase Auth
- OAuth login (Google, GitHub) — lower friction for developers
- Handles email verification, password reset, MFA out of the box

**Recommendation:** Start with Option A (email + password + GitHub OAuth via a
small custom implementation). Avoid vendor lock-in early. Add more providers later.

Create `crates/tuitbot-server/src/auth/` with:
- `mod.rs` — middleware that extracts user from session token
- `password.rs` — argon2 hash/verify
- `session.rs` — session create/validate/revoke
- `oauth.rs` — GitHub OAuth flow (optional but high-value)

New routes:
- `POST /auth/signup` — create account (email, password)
- `POST /auth/login` — authenticate, return session token
- `POST /auth/logout` — revoke session
- `GET /auth/me` — current user info + subscription status
- `POST /auth/forgot-password` — send reset email
- `POST /auth/reset-password` — reset with token

### 3. Subscription billing (Stripe)

Integrate Stripe for subscription management.

#### Pricing tiers:
| Tier | Price | Limits |
|------|-------|--------|
| **Starter** | $19/mo | 5 replies/day, 3 tweets/day, 1 thread/week, 3 targets |
| **Growth** | $29/mo | 15 replies/day, 10 tweets/day, 3 threads/week, 10 targets |
| **Pro** | $49/mo | 30 replies/day, 20 tweets/day, 7 threads/week, unlimited targets |

Create `crates/tuitbot-server/src/billing.rs`:

- Stripe Checkout for new subscriptions
- Stripe Customer Portal for managing subscriptions
- Webhook handler (`POST /webhooks/stripe`) for:
  - `checkout.session.completed` — activate subscription
  - `invoice.paid` — extend subscription
  - `invoice.payment_failed` — grace period warning
  - `customer.subscription.deleted` — deactivate, stop automation

New routes:
- `POST /api/billing/checkout` — create Stripe Checkout session
- `GET /api/billing/portal` — redirect to Stripe Customer Portal
- `GET /api/billing/status` — current plan, usage, renewal date

Enforce tier limits in the automation runtime: override `max_replies_per_day` etc.
based on the user's plan. The tenant config is the source of truth — billing just
sets the ceilings.

### 4. X API OAuth (managed app)

For cloud users, you provide the X API OAuth app so they don't need developer
portal access. This is the biggest UX win over self-hosting.

- Register a single X API OAuth 2.0 app under your developer account
- Cloud users authenticate via OAuth flow (you store their refresh tokens)
- Each user's tokens are stored encrypted in their tenant DB
- Token refresh loop runs per-tenant (already exists in core)

Create `crates/tuitbot-server/src/x_oauth.rs`:

- `GET /api/x/connect` — start OAuth flow, redirect to X authorization URL
- `GET /api/x/callback` — handle OAuth callback, store tokens
- `DELETE /api/x/disconnect` — revoke tokens, disconnect account

**Security considerations:**
- Encrypt refresh tokens at rest (use `aes-gcm` with a server-level key)
- The server-level encryption key lives in environment variables, not in code
- Users can revoke access at any time from their X settings
- Rate limit the OAuth flow to prevent abuse

### 5. LLM provider handling

Two options for cloud users:

**Option A: BYOK (Bring Your Own Key)** — users enter their OpenAI/Anthropic key
in settings, stored encrypted per-tenant. You pay zero LLM costs.

**Option B: Managed LLM** — you proxy LLM calls through your own API key, cost
absorbed into subscription price. Simpler for users, but you eat the cost.

**Recommendation:** Start with BYOK. Add managed LLM as an add-on later ($5-10/mo
extra). This keeps margins healthy and avoids subsidizing heavy LLM users.

### 6. Dashboard as web app

The Svelte dashboard already works as a standalone web app (SvelteKit with static
adapter). For cloud hosting:

- Switch to `@sveltejs/adapter-node` for server-side rendering (better SEO for
  marketing pages, faster initial load)
- Add marketing pages: `/` (landing), `/pricing`, `/login`, `/signup`
- The authenticated dashboard lives at `/dashboard/*` (same routes as desktop)
- Add a layout guard that redirects unauthenticated users to `/login`

The desktop app (Tauri) continues using the static adapter. You now have two build
targets:

```bash
# Desktop (static SPA, served by Tauri)
cd dashboard && npm run build:desktop

# Cloud (Node.js server, SSR for marketing pages)
cd dashboard && npm run build:cloud
```

### 7. Automation runtime management

Each cloud user gets their own automation runtime instance. The tenant manager
handles lifecycle:

- **On subscription activation:** Create tenant state, start runtime
- **On dashboard visit:** Ensure runtime is running, connect WebSocket
- **On subscription cancellation:** Stop runtime, keep data for 30 days (grace period)
- **On account deletion:** Stop runtime, schedule data deletion

Resource management:
- Each runtime is lightweight (6 tokio tasks + a DB pool)
- A single server can handle ~500-1000 tenants depending on hardware
- Monitor per-tenant memory and CPU usage
- Add health checks: if a tenant's runtime crashes, auto-restart it

### 8. Docker deployment

Create `Dockerfile` and two compose files at the repo root. The Dockerfile is
shared — only the compose file and `--mode` flag differ.

```dockerfile
# Dockerfile — shared by cloud and self-host
FROM node:20-slim AS frontend
WORKDIR /app/dashboard
COPY dashboard/package*.json ./
RUN npm ci
COPY dashboard/ ./
RUN npm run build:cloud

FROM rust:1.75-slim AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/
RUN cargo build --release -p tuitbot-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/target/release/tuitbot-server /usr/local/bin/
COPY --from=frontend /app/dashboard/build /srv/dashboard
ENV TUITBOT_DASHBOARD_DIR=/srv/dashboard
ENV TUITBOT_DATA_DIR=/data
VOLUME /data
EXPOSE 3001
# Default to self-host mode — self-hosters don't need to think about modes
CMD ["tuitbot-server", "--mode", "self-host"]
```

#### Self-host compose (`docker-compose.yml`)

This is what self-hosters use. No Stripe, no multi-tenant, no managed OAuth.
Single user, their own API keys, browser dashboard.

```yaml
# docker-compose.yml — self-host (default)
services:
  tuitbot:
    image: tuitbot/tuitbot:latest  # or build: .
    ports: ["3001:3001"]
    volumes: ["tuitbot_data:/data"]
    environment:
      # Users provide their own keys — that's it
      - TUITBOT_X_API__CLIENT_ID=${X_API_CLIENT_ID}
      - TUITBOT_LLM__API_KEY=${LLM_API_KEY}
      - TUITBOT_LLM__PROVIDER=${LLM_PROVIDER:-openai}
    restart: unless-stopped

volumes:
  tuitbot_data:
```

Self-host setup is 3 steps:
1. Copy `.env.example` → `.env`, fill in X API + LLM keys
2. `docker compose up -d`
3. Open `http://server-ip:3001` → onboarding wizard

#### Cloud compose (`docker-compose.cloud.yml`)

This is what you use to deploy the managed service.

```yaml
# docker-compose.cloud.yml — your hosted deployment
services:
  tuitbot:
    image: tuitbot/tuitbot:latest
    command: ["tuitbot-server", "--mode", "cloud"]
    ports: ["3001:3001"]
    volumes: ["tuitbot_data:/data"]
    environment:
      - STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
      - STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}
      - X_API_CLIENT_ID=${X_API_CLIENT_ID}
      - X_API_CLIENT_SECRET=${X_API_CLIENT_SECRET}
      - ENCRYPTION_KEY=${ENCRYPTION_KEY}
      - SMTP_URL=${SMTP_URL}
    restart: unless-stopped

volumes:
  tuitbot_data:
```

Same image, different command. Cloud mode enables multi-tenant auth, Stripe
billing, managed X API OAuth, and the marketing pages.

### 9. Cloud deployment

Recommended: **Fly.io** or **Railway** for initial deployment.

- Single machine to start (scale later)
- Persistent volume for `/data` (tenant databases)
- Automatic TLS via the platform
- Deploy via `fly deploy` or `railway up`

Domain: `app.tuitbot.dev` (or similar)

### 10. Email transactional

You'll need to send emails for:
- Account verification
- Password reset
- Payment receipts (Stripe handles these)
- Weekly performance summaries (optional, high-value feature)
- Approval queue reminders ("You have 5 items waiting")

Use Resend, Postmark, or AWS SES. Create a small email module:
- `crates/tuitbot-server/src/email.rs`
- Template-based (plain text first, HTML later)
- Queue emails via a background tokio task

### 11. Marketing site

The landing page at `/` should sell the product. Key pages:

- **`/`** — hero, features, social proof, CTA
- **`/pricing`** — tier comparison table, FAQ
- **`/login`** and **`/signup`** — auth forms
- **`/docs`** — self-hosting guide, API reference

These are SvelteKit pages with SSR enabled for SEO. The authenticated dashboard
at `/dashboard/*` is SPA (client-side only).

## Migration path for existing desktop users

Desktop app users who want to switch to cloud:

1. Export: `tuitbot export --format json` (new CLI command) — dumps their config +
   historical data
2. Import: upload the JSON in cloud dashboard settings
3. Their SQLite data becomes the tenant DB, config maps to cloud settings
4. Disconnect local runtime, cloud takes over

## What NOT to build yet

- Team/agency features (multiple X accounts per subscription)
- API access for third-party integrations
- White-label / reseller program
- Mobile app
- Advanced analytics (compared to competitors, industry benchmarks)

## Acceptance criteria

### Self-host (Docker)
- [ ] `docker compose up` starts the server + dashboard in a single container
- [ ] Browser at `http://localhost:3001` shows the onboarding wizard on first run
- [ ] Self-host mode uses local file token auth (no signup/login page)
- [ ] Self-hosters provide their own X API + LLM keys via env vars or onboarding wizard
- [ ] Automation runs 24/7 inside the container, survives container restarts
- [ ] All dashboard pages work in the browser (analytics, activity, approval, calendar, targets, settings)
- [ ] Desktop app (Tauri) continues to work unchanged

### Cloud hosted
- [ ] Users can sign up with email + password at `app.tuitbot.dev`
- [ ] Stripe checkout creates a subscription and activates the account
- [ ] X API OAuth flow connects the user's X account (no developer portal needed)
- [ ] Automation runtime starts and runs 24/7 for paying users
- [ ] Per-user data isolation (each user has their own DB)
- [ ] Subscription cancellation stops automation and shows grace period notice
- [ ] Tier limits enforce reply/tweet/thread maximums
- [ ] Marketing pages (landing, pricing) render with SSR for SEO

### Shared
- [ ] Same Docker image used for both self-host and cloud (only `--mode` flag differs)
- [ ] Server serves the dashboard frontend as static files when `TUITBOT_DASHBOARD_DIR` is set
- [ ] Migration path works: desktop user can export and import into cloud

## Reference files

- `crates/tuitbot-server/src/` — everything from tasks 01-02
- `crates/tuitbot-core/src/automation/mod.rs` — runtime lifecycle
- `crates/tuitbot-core/src/config/mod.rs` — config structure
- `dashboard/` — Svelte frontend from tasks 03-09
- `docs/roadmap/10-packaging-and-distribution.md` — desktop packaging (stays as-is)
