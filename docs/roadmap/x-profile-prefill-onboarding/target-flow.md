# Target Onboarding Flow

## Overview

The target flow replaces the current 8-9 step wizard with a 10-step flow that front-loads X sign-in and LLM setup, uses profile inference to prefill business fields, and presents all inferred values on a single editable review screen.

Step count increases by 1 (analysis step added, but Profile step becomes faster), while perceived effort drops significantly because 6 manual fields become editable prefills.

## Step-by-Step Journey

### Step 0: Welcome

**Identical across:** Desktop, SelfHost

**What the user sees:**
- Tuitbot logo and brand
- Updated headline: "Connect your X account to get started"
- Subtext: "Tuitbot analyzes your profile and tweets to set up your content strategy automatically."
- "Get Started" button

**Validation:** None (always advanceable)

**Changes from current:** Copy update only — mention X sign-in and automatic setup.

---

### Step 1: X Access + Auth

**Identical across:** Desktop, SelfHost

**What the user sees:**

Phase A — Credential Entry (same as current XApiStep):
- Client ID input field
- Client Secret input field (optional)
- Scraper mode toggle ("No API key? Use read-only mode")
- If scraper mode selected: skip to Step 3 (LLM)

Phase B — Account Connection (new):
- After entering Client ID, a "Connect your X account" button appears
- Button opens OAuth PKCE flow in a new browser tab/window
- While waiting: spinner + "Waiting for authorization..."
- On success: shows connected state — avatar, @username, display name
- On failure: error message with retry option

**OAuth implementation:**
- Frontend calls `POST /api/onboarding/x-auth/start` (new endpoint, no account ID needed — uses a temporary pre-account state keyed by Client ID)
- OAuth callback handled by existing callback infrastructure
- Frontend polls `GET /api/onboarding/x-auth/status/{state}` until complete
- On success, OAuth tokens are stored in a temporary location (not yet associated with an account — account doesn't exist until config init)

**Validation:** Client ID required (or scraper mode). If API mode, OAuth must complete successfully before advancing.

**Changes from current:** OAuth sign-in added as Phase B. Two-phase step: credentials then connection.

---

### Step 2: LLM Setup

**Identical across:** Desktop, SelfHost

**What the user sees:**
- Same as current LlmStep: provider dropdown, API key input, model selector, base URL (optional for Ollama)
- New helper text at top: "Tuitbot needs an AI model to analyze your profile and generate content."

**Validation:** Same as current — API key + model required (except Ollama which only needs model).

**Changes from current:** Moved from step 3 to step 2. Copy updated to mention profile analysis.

---

### Step 3: Profile Analysis (new)

**Identical across:** Desktop, SelfHost (skipped for scraper mode)

**What the user sees:**
- Animated loading state with progress steps:
  1. "Fetching your X profile..." (checkmark when done)
  2. "Reading your recent tweets..." (checkmark when done)
  3. "Analyzing your content strategy..." (checkmark when done)
- Each step transitions after its server-side operation completes
- Total expected time: 3-8 seconds depending on LLM latency

**Server behavior:**
- Frontend calls `POST /api/onboarding/analyze-profile` with:
  - `client_id` (to construct X API client)
  - LLM config (provider, api_key, model, base_url)
  - OAuth state reference (to retrieve tokens)
- Server performs:
  1. Build `XApiHttpClient` with OAuth access token
  2. Call `get_me()` with expanded `user.fields` (description, location, url, entities)
  3. Call `get_user_tweets()` for last 15 tweets
  4. Run `profile_inference::infer()` with User + tweets + LLM provider
  5. Return `InferredProfile` JSON

**Error handling:**
- X API failure: "Could not fetch your profile. You can enter your details manually." → advance to manual BusinessStep
- LLM failure: "AI analysis unavailable. You can enter your details manually." → advance to manual BusinessStep
- Partial failure (e.g., tweets failed but profile succeeded): Run inference with available data, flag affected fields as low confidence

**Validation:** Auto-advances on completion (or on fallback to manual)

---

### Step 4: Editable Profile Review (replaces BusinessStep)

**Identical across:** Desktop, SelfHost

**What the user sees (with inference):**

Header: "Review Your Profile"
Subtext: "We analyzed your X account and suggested these settings. Edit anything that doesn't look right."

- **Account Type** toggle (Individual / Business) — inferred, editable
- **Name** — inferred from X display name or bio, editable, shows provenance tag
- **Description** — inferred from X bio, editable, shows provenance tag
- **Website** — pulled from X profile URL, editable, shows provenance tag
- **Target Audience** — inferred from bio + tweets, editable, shows confidence badge + provenance
- **Discovery Keywords** — inferred from bio + tweet topics, editable tag input, shows confidence badge + provenance
- **Content Topics** — inferred from tweet themes, editable tag input, shows confidence badge + provenance

Each field has:
- The inferred value pre-filled in the input
- A small confidence indicator: green dot (high), yellow dot (medium), orange dot (low)
- A provenance label: "from bio", "from tweets", "from bio + tweets", "from profile URL"
- Full edit capability — user can change any value

An "Accept All" visual confirmation at the top that all fields have values.

**What the user sees (scraper mode / fallback):**
- Same as current BusinessStep — all fields empty, manual entry required
- No confidence badges or provenance tags

**Validation:** Same as current BusinessStep — product_name, product_description, target_audience, product_keywords, industry_topics all required.

---

### Step 5: Language & Brand

**Identical across:** Desktop, SelfHost

**What the user sees:**
- Language selector (default: English)
- Brand voice selector (default: balanced)
- If inference ran: brand_voice may be pre-selected based on tweet tone analysis (e.g., "casual" if tweets are informal)

**Validation:** Always valid (defaults available)

**Changes from current:** Possible pre-selection of brand_voice from inference.

---

### Step 6: Content Sources (Vault)

**Identical across:** Desktop, SelfHost (capability-gated defaults)

**What the user sees:** Same as current SourcesStep. Optional step.

**Validation:** Always valid (optional)

**Changes from current:** None.

---

### Step 7: Validation

**Identical across:** Desktop, SelfHost

**What the user sees:** Same as current ValidationStep — LLM connection test.

**Validation:** Always advanceable (validation is informational)

**Changes from current:** None.

---

### Step 8: Review & Launch

**Identical across:** Desktop, SelfHost

**What the user sees:** Same as current ReviewStep — summary of all configured values, approval_mode toggle.

**Validation:** Always valid

**Changes from current:** Shows inferred-then-edited profile values. If inference ran, a small note: "Profile auto-configured from your X account."

---

### Step 9: Secure (web-only)

**Present in:** SelfHost only (gated by `!isTauri`)

**What the user sees:** Same as current ClaimStep — passphrase generation, save confirmation.

**Validation:** Passphrase >= 8 chars and save acknowledged

**Changes from current:** None.

---

## Scraper Mode Flow

When the user selects "No API key" (scraper mode) in Step 1:

- Step 1 Phase B (OAuth): Skipped
- Step 2 (LLM): Same
- Step 3 (Analysis): Skipped entirely
- Step 4 (Profile Review): Falls back to current manual BusinessStep — all fields empty
- Steps 5-9: Same as API mode

This ensures no regression for scraper mode users.

## Data Flow Diagram

```
Step 1 (X Access)
  ├─ Client ID → stored in onboarding state
  ├─ OAuth PKCE flow → temporary token storage
  └─ Connected user info (avatar, username) → UI confirmation

Step 2 (LLM)
  └─ LLM config → stored in onboarding state

Step 3 (Analysis)
  ├─ Input: OAuth tokens + LLM config
  ├─ Server: get_me() → User with bio fields
  ├─ Server: get_user_tweets() → 15 recent tweets
  ├─ Server: profile_inference::infer() → InferredProfile
  └─ Output: InferredProfile → stored in onboarding state

Step 4 (Review)
  ├─ Input: InferredProfile from state (or empty if scraper/fallback)
  ├─ User edits any fields
  └─ Output: Final business profile values → onboarding state

Steps 5-9
  └─ Standard flow using finalized onboarding state

Submit (end of last step)
  └─ POST /api/settings/init with all onboarding state → config.toml created
```

## Onboarding State Additions

The `OnboardingData` interface in `dashboard/src/lib/stores/onboarding.ts` needs these additions:

```typescript
// X Auth state (new)
x_connected: boolean;
x_username: string;
x_display_name: string;
x_avatar_url: string;
oauth_state: string;  // for polling auth status

// Inference state (new)
inference_status: 'idle' | 'running' | 'complete' | 'failed' | 'skipped';
inferred_profile: InferredProfile | null;
```

The existing fields (product_name, product_description, etc.) continue to hold the final edited values, whether they came from inference or manual entry.

## Temporary Token Storage

OAuth tokens obtained during onboarding are stored temporarily because:
- No account exists yet (accounts are created after `POST /api/settings/init`)
- The existing `x_auth.rs` flow requires an account ID

Solution: Store tokens in a temporary path keyed by the OAuth state parameter. After `POST /api/settings/init` creates the config and the default account, migrate tokens to the account's standard token path. This is analogous to the existing `migrate_default_credentials` pattern in `accounts.rs`.

## Progress Bar Labels

Current: Welcome, X Access, Profile, LLM, Language, Vault, Validate, Review, [Secure]

Target: Welcome, X Access, LLM, Analyze, Profile, Language, Vault, Validate, Review, [Secure]

The "Analyze" step appears in the progress bar but auto-advances on completion. For scraper mode, it is hidden (same as Secure is hidden for Desktop).
