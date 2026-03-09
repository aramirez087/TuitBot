# Progressive Activation Design

## Overview

Progressive activation allows users to complete a shortened onboarding flow and start using Tuitbot immediately, without configuring every advanced capability upfront. Missing capabilities are surfaced as explicit, resumable states rather than hidden validation failures.

## Capability Tiers

| Tier | Name | Required | Unlocks |
|------|------|----------|---------|
| 0 | `unconfigured` | Nothing | Redirect to onboarding |
| 1 | `profile_ready` | Business profile (name, description, keywords) | Dashboard access, view settings |
| 2 | `exploration_ready` | Tier 1 + X credentials (client_id or scraper mode) | Discovery, search, scoring |
| 3 | `generation_ready` | Tier 2 + LLM config (provider + API key for cloud, or ollama) | Draft generation, reply composition |
| 4 | `posting_ready` | Tier 3 + valid posting tokens (OAuth or scraper session) | Scheduled posting, autopilot |

### Tier Computation

Tiers are **computed, never stored**. The `compute_tier(config, can_post)` function in `tuitbot-core::config::capability` derives the tier from the current config and runtime state every time it's called. This eliminates stale-state bugs.

### Server Exposure

Both endpoints return `capability_tier`:
- `GET /api/settings/status` — passes `can_post=false` (no runtime context)
- `GET /api/runtime/status` — passes actual `can_post` value

## Onboarding Changes

### Shortened Flow

The onboarding wizard now requires only:
- **X Access** — mode selection + optional OAuth
- **Profile** — name, description, keywords, topics, audience

All other steps are optional:
- **LLM** — skippable; skipping also skips Analyze
- **Language/Brand** — skippable; defaults to English/balanced
- **Vault** — skippable; no content source configured
- **Validate** — handles missing LLM gracefully

### Skip to Finish

After the Profile step, a "Skip optional steps" button appears alongside "Next". Clicking it jumps directly to Review. Skipped steps show with dashed borders in the progress bar.

### Server-Side Validation

`init_settings` now uses `validate_minimum()` instead of `validate()`:
- `validate_minimum()` checks only business profile fields, structural requirements (db_path), and content source deployment compatibility
- Skips LLM API key requirements and X API client_id requirements
- Full `validate()` is still used by `PATCH /api/settings` and `POST /api/settings/validate`

## Frontend Stores

### `capability.ts`
Derived stores from runtime status:
- `capabilityTier` — current tier string
- `canExplore` — tier >= 2
- `canGenerate` — tier >= 3
- `canPublish` — tier >= 4

### `runtime.ts`
Extended to parse and expose `capability_tier` from both runtime and config status responses.

### `onboarding.ts`
Added `isMinimalComplete()` helper that checks only the profile fields needed for tier 1.

## UI States

### ReviewStep
Shows a tier indicator card:
- **Amber** when capabilities are deferred — lists what's missing
- **Green** when all capabilities are configured
- Each section shows "Configured" or "Set up later" badges

### ValidationStep
- When LLM is configured: runs connection test as before
- When LLM is not configured: shows info card explaining generation will be available after LLM setup in Settings

## Measurement Plan

Tier transitions should be tracked at:
1. **Onboarding submit** — log initial tier in the init response
2. **Settings PATCH** — detect tier changes via before/after comparison
3. **Runtime status polls** — frontend can observe tier upgrades

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Partial configs crash at runtime | `validate_minimum()` still validates structural fields. Runtime code handles missing LLM gracefully. |
| Users forget to configure LLM | ReviewStep clearly shows deferred items. Dashboard can show tier-based prompts. |
| Existing validate() path breaks | `validate()` is unchanged. `validate_minimum()` is purely additive. |
