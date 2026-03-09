# Session 05: Progressive Activation And Capability Gating

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Implement progressive activation so the short onboarding no longer blocks on every advanced credential, local setting, or posting prerequisite.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/session-04-handoff.md
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/stores/onboarding.ts
- dashboard/src/lib/components/onboarding/XApiStep.svelte
- dashboard/src/lib/components/onboarding/LlmStep.svelte
- dashboard/src/lib/components/onboarding/SourcesStep.svelte
- dashboard/src/lib/components/onboarding/ValidationStep.svelte

Tasks
1. Decide which capabilities are required for first-run versus safe to defer, including posting credentials, LLM setup, content sources, and self-host passphrase claim.
2. Refactor onboarding init and config state so a user can complete the short X-first setup and enter a safe starter mode without filling every advanced field immediately.
3. Add explicit capability states such as profile-ready, read-only exploration, generation-ready, and posting-ready, and make the transitions deterministic.
4. Keep missing prerequisites visible and recoverable instead of hiding them behind broken actions or the legacy wizard.
5. Add tests for partially configured starter state, later capability completion, and non-destructive fallback behavior.

Deliverables
- docs/roadmap/x-profile-prefill-onboarding/progressive-activation.md
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/stores/onboarding.ts
- dashboard/src/lib/components/onboarding/ValidationStep.svelte
- docs/roadmap/x-profile-prefill-onboarding/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- A user can finish the short onboarding without being forced to configure every advanced capability up front.
- Missing capabilities are represented as explicit, resumable state instead of hidden validation failures.
- The handoff gives Session 06 the exact checklist and first-run states to implement.
```
