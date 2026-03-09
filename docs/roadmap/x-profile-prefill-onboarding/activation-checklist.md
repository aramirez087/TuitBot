# Activation Checklist

## Overview

After completing the short onboarding wizard (which now allows skipping LLM and other optional steps), users land on the home page where an **Activation Checklist** shows exactly which capabilities are unlocked, which are deferred, and what the next best action is.

## Checklist Model

Checklist items are derived from the `CapabilityTier` system (Session 05). Each item maps to a tier threshold and links directly to the relevant Settings section — no mini-wizards or duplicated credential flows.

### Items

| Item | Completed when | Settings link | Desktop | Self-host |
|------|---------------|---------------|---------|-----------|
| Business profile | tier >= 1 | `/settings#business` | yes | yes |
| X credentials | tier >= 2 | `/settings#xapi` | yes | yes |
| LLM provider | tier >= 3 | `/settings#llm` | yes | yes |
| Posting access | tier >= 4 | `/settings#xapi` | yes | yes |
| Knowledge vault | always optional | `/settings#sources` | yes | yes |

### "What you can do now" by tier

| Tier | Label | Available actions |
|------|-------|-------------------|
| 1 | Profile Ready | View dashboard, edit settings, review profile |
| 2 | Exploration Ready | + Browse discovery, view targets, score content |
| 3 | Generation Ready | + Create AI drafts, compose replies |
| 4 | Posting Ready | + Schedule posts, enable autopilot |

## Deployment-Mode Filtering

The checklist renders the same items for both desktop and self-host modes. Cloud-only items (none currently) would be filtered by `deploymentMode`. This keeps the component future-compatible.

## Settings Deep-Link Strategy

Each checklist item links to `/settings#<section-id>`. The Settings page sections already have `id` attributes (`business`, `xapi`, `llm`, `sources`). The browser's native hash-based scrolling handles navigation.

## Dismissal Behavior

- Checklist is dismissible per-session via a local `$state` boolean.
- Dismissed state resets when the capability tier changes (detected via `$effect`).
- Auto-hides entirely at tier 4 (`posting_ready`).

## Integration Points

- **Home page** (`/`): Full checklist rendered above `DraftStudioShell`.
- **Content calendar** (`/content`): Compact banner with current tier and next action.
- Checklist state is purely derived from stores — no persistence, no API calls.

## Measurement Plan (Session 08)

- Track checklist item clicks (which setting section users navigate to first).
- Track tier transitions over time (how quickly users progress).
- Track dismissal rate vs. completion rate.
