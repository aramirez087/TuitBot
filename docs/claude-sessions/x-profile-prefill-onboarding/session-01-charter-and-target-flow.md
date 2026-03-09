# Session 01: Charter And Target Flow

Paste this into a new Claude Code session:

```md
Mission
Produce the implementation charter for a deployment-unified onboarding flow that starts with X sign-in, infers profile inputs, and keeps the cross-mode experience as similar as possible.

Repository anchors
- .claude/product-marketing-context.md
- docs/architecture.md
- docs/auth-matrix.md
- docs/getting-started.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/stores/onboarding.ts
- dashboard/src/lib/components/onboarding/BusinessStep.svelte
- crates/tuitbot-server/src/routes/settings.rs
- crates/tuitbot-server/src/routes/accounts.rs
- crates/tuitbot-server/src/auth/routes.rs

Tasks
1. Audit the current onboarding flow and enumerate every required step, field, and validation that creates friction today.
2. Define the target-state journey for desktop and self-hosted deployments from entry to X auth, analysis, editable prefill, any mode-specific finalization, and first value, while noting future cloud compatibility where useful.
3. Specify the architecture for onboarding session state, X identity bootstrap, inference, deployment-specific finalization, config materialization, and minimal necessary branching.
4. Define the inferred field contract for account type, name, description, audience, keywords, topics, tones, goals, cadence, confidence, and provenance.
5. Split the implementation into session-sized slices, identify the highest-risk dependencies, and document the test strategy with no placeholders.

Deliverables
- docs/roadmap/x-profile-prefill-onboarding/epic-charter.md
- docs/roadmap/x-profile-prefill-onboarding/target-flow.md
- docs/roadmap/x-profile-prefill-onboarding/inference-contract.md
- docs/roadmap/x-profile-prefill-onboarding/session-01-handoff.md

Quality gates
- No broad code changes are expected in this session; if you touch code, run the same checks required for later sessions.

Exit criteria
- The charter explains which onboarding steps are shared across all modes and exactly where the flow is allowed to diverge.
- The target flow is detailed enough that later sessions can implement without re-deciding core product behavior.
- The handoff names the exact files and decisions Session 02 must pick up.
```
