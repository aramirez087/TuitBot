# PRD v1 Execution Pack (Gap-Only Plan)

## Why this exists

This roadmap translates the Spanish PRD into executable, Claude-friendly work
units while avoiding rework on features already shipped in this repo.

Use this as the entrypoint. Execute tasks in order (`PRDV1-01` to `PRDV1-09`).
Each task file is scoped so it can be completed in a focused implementation
pass.

## Baseline from current code (already built)

- MCP server exists with many tools and initial policy gate.
- Composer mode + approval queue + drafts + discovery + calendar already exist.
- Dashboard already has Discovery, Drafts, Approval, Settings, Costs, MCP, Activity.
- OAuth2 PKCE + refresh exists.
- Media upload exists in core/server flow (dashboard path).
- MCP telemetry table and MCP governance endpoints already exist.

## Major gaps vs PRD (what this roadmap covers)

- MCP response envelope is not strict across all tools.
- Error taxonomy is incomplete (`rate_limit_reset`, `policy_decision` fields missing).
- Policy engine is still v1-level (limited rules, no template profiles).
- Approval queue lacks reviewer identity, notes, risk context, and batch safeguards.
- Missing direct X capability: retweet/unretweet, delete, first-class thread post, MCP media tool.
- Auth diagnostics are basic; no actionable scope diagnostics in `tuitbot test`.
- Bilingual + brand QA gates are not implemented as first-class artifacts.
- Exports (CSV/JSON) and explicit observability UX are incomplete.
- SRE-lite features (circuit-breaker autopause, backup/recovery flow, migration preflight) are missing.
- Agency/multi-account foundation is not implemented.

## Execution order

1. [PRDV1-01-mcp-contract-and-error-taxonomy.md](./PRDV1-01-mcp-contract-and-error-taxonomy.md)
2. [PRDV1-02-policy-engine-v2.md](./PRDV1-02-policy-engine-v2.md)
3. [PRDV1-03-approval-queue-professionalization.md](./PRDV1-03-approval-queue-professionalization.md)
4. [PRDV1-04-tool-coverage-engage-media-threads.md](./PRDV1-04-tool-coverage-engage-media-threads.md)
5. [PRDV1-05-auth-pro-and-token-diagnostics.md](./PRDV1-05-auth-pro-and-token-diagnostics.md)
6. [PRDV1-06-bilingual-brand-voice-qa-gates.md](./PRDV1-06-bilingual-brand-voice-qa-gates.md)
7. [PRDV1-07-dashboard-sellable-v1.md](./PRDV1-07-dashboard-sellable-v1.md)
8. [PRDV1-08-sre-lite-and-operability.md](./PRDV1-08-sre-lite-and-operability.md)
9. [PRDV1-09-agency-multi-account-foundation.md](./PRDV1-09-agency-multi-account-foundation.md)

## Definition of done for the full PRD v1

- Safe default produces `0` accidental posts in composer/safe mode.
- MCP tools consistently return strict envelope schema.
- All high-risk mutations are policy-gated and auditable.
- Bilingual + brand QA report is emitted on every draft before approval.
- Dashboard supports onboarding, approvals, observability, and export workflows.
- Runbooks and operational commands support 24/7 self-host operation.

## Guardrails for implementation agents

- Build only what is listed as gap in each task file.
- Do not remove existing routes/pages unless task explicitly requests it.
- Keep config backward-compatible when introducing new policy/QA fields.
- Add migrations for all schema changes.
- Every task must include unit/integration test coverage and docs update.
