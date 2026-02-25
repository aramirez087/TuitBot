# PRDV1-07: Dashboard v1 (Sellable Product Surface)

## Goal

Close the UX/productization gaps so onboarding, observability, and export
workflows are ready for external customers.

## Already built (do not redo)

- Core pages already exist: Discovery, Drafts/Content, Approval, Settings, Costs, Activity, MCP.
- Onboarding flow already exists with X API + business + LLM setup.

## Gaps to implement

1. Onboarding completion for PRD flow:
   - connect X
   - choose language/brand profile
   - run quick validation test
   - land in composer with safe template
2. Discovery feed filters:
   - topic
   - score range
   - language
3. Export support:
   - CSV/JSON export for activity and approvals
4. Observability coherence:
   - single place showing LLM cost, X usage, errors, rate limits, policy outcomes
5. Audit usability:
   - better filtering/search for who approved what and when

## Primary code touchpoints

- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/activity.rs`
- `crates/tuitbot-server/src/routes/approval.rs`
- `crates/tuitbot-server/src/routes/mcp.rs`
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/*`
- `dashboard/src/routes/onboarding/+page.svelte`
- `dashboard/src/routes/(app)/discovery/+page.svelte`
- `dashboard/src/routes/(app)/activity/+page.svelte`
- `dashboard/src/routes/(app)/approval/+page.svelte`

## Implementation tasks

1. Add export endpoints.
   - `GET /api/activity/export?format=csv|json`
   - `GET /api/approval/export?format=csv|json`
2. Add frontend export actions.
   - Download buttons with selected filter context.
3. Improve onboarding workflow.
   - Add language + policy template setup step.
   - Add post-setup validation call.
4. Upgrade discovery filters.
   - add query params and UI controls for topic/language/score.
5. Create observability view composition.
   - aggregate data from costs + MCP telemetry + rate limits + runtime health.
6. Add audit metadata rendering.
   - reviewer, note, timestamp, action source.

## Acceptance criteria

- New user can go from onboarding to safe composer flow in one guided sequence.
- Exports produce usable CSV/JSON files for activity and approvals.
- Observability view shows cross-system health and governance indicators.
- Discovery filter UX supports topic/score/language combinations.

## Verification commands

```bash
cargo test -p tuitbot-server
cd dashboard && npm run check
```

## Out of scope

- Deep SRE behavior and circuit breakers (PRDV1-08).
- Multi-account tenant views (PRDV1-09).
