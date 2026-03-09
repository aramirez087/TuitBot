# Onboarding Funnel Metrics

## Funnel Stages

| # | Stage | Event | Description |
|---|-------|-------|-------------|
| 1 | Entry | `onboarding:started` | User begins the onboarding wizard |
| 2 | X Auth | `onboarding:x-auth-success` or `onboarding:scraper-selected` | X connection established or scraper chosen |
| 3 | Analysis | `onboarding:analysis-success` or `onboarding:analysis-skipped` | Profile analyzed or user continued manually |
| 4 | Profile Complete | `onboarding:step-entered` (step=Review) | Required profile fields filled, user reached review |
| 5 | Submission | `onboarding:submitted` | Init payload sent to server |
| 6 | Completion | `onboarding:completed` | Config created, user redirected to app |
| 7 | First Checklist View | `activation:checklist-viewed` | User sees the activation checklist on home |
| 8 | Tier 2 | `activation:tier-changed` (to=exploration_ready) | X credentials configured |
| 9 | Tier 3 | `activation:tier-changed` (to=generation_ready) | LLM provider configured |
| 10 | Tier 4 | `activation:tier-changed` (to=posting_ready) | Full posting access unlocked |

## Key Activation Metrics

| Metric | Formula | Target |
|--------|---------|--------|
| **Onboarding completion rate** | `onboarding:completed` / `onboarding:started` | >80% |
| **X auth rate** | `onboarding:x-auth-success` / `onboarding:started` | >60% |
| **Analysis utilization** | `onboarding:analysis-success` / `onboarding:x-auth-success` | >70% |
| **Profile edit rate** | unique users with `onboarding:profile-edited` / `onboarding:analysis-success` | informational |
| **Skip rate** | `onboarding:step-skipped` / `onboarding:started` | informational |
| **Time to posting_ready** | timestamp delta from `onboarding:completed` to `activation:tier-changed{to:posting_ready}` | <24h |
| **Checklist engagement** | unique users with `activation:checklist-item-clicked` / `activation:checklist-viewed` | >40% |

## Drop-Off Points to Watch

1. **Welcome -> X Access**: Users who don't click "Get Started" — may indicate unclear value prop
2. **X Access -> next step**: X Developer Portal setup is the highest-friction step
3. **Analysis -> Profile**: Analysis errors or sparse profiles may cause abandonment
4. **Review -> Submit**: Network errors or config validation failures
5. **Checklist -> Tier 2+**: Users who complete onboarding but never configure remaining capabilities

## Measurement Method

All events are logged to `console.info('[tuitbot:funnel]', ...)` with structured JSON payloads.

**How to access:**
- **Desktop (Tauri)**: Tauri logs in the app data directory
- **Browser DevTools**: Filter console by `[tuitbot:funnel]`
- **Self-hosted**: Server stdout if log level includes info

**Event format:**
```json
{
  "event": "onboarding:completed",
  "properties": { "tier": "generation_ready", "claimed": false },
  "timestamp": "2026-03-09T14:30:00.000Z"
}
```

## Future: Server-Side Event Sink

Not in scope for Session 8. When needed:
- `POST /api/telemetry/funnel` accepting batch event arrays
- SQLite table `funnel_events(id, event, properties_json, created_at)`
- Dashboard page showing funnel visualization
