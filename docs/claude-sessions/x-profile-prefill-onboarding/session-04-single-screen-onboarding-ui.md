# Session 04: Single Screen Onboarding UI

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Replace the onboarding wizard with a single-screen editable prefill experience that feels fast, confident, and obviously tied to the user's X account across deployment modes.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/target-flow.md
- docs/roadmap/x-profile-prefill-onboarding/session-03-handoff.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/stores/onboarding.ts
- dashboard/src/lib/stores/onboarding-session.ts
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- dashboard/src/lib/components/onboarding/BusinessStep.svelte
- dashboard/src/lib/components/onboarding/ReviewStep.svelte

Tasks
1. Build the shared onboarding screen with analysis-progress, editable inferred fields, explicit account identity, and a clear primary CTA into the next step.
2. Surface inferred values for name, description, audience, keywords, topics, tones, goals, and cadence with sensible controls and confidence or provenance affordances.
3. Keep manual editing first-class and add resilient states for no-analysis, low-confidence, auth-loss, and retry.
4. Use the same UI skeleton across desktop and self-host, and branch only in the post-submit routing or extra fields that are truly mode-specific.
5. Add component-level or route-level validation that ensures the single-screen form still produces a coherent payload in every mode.

Deliverables
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/components/onboarding/ProfileAnalysisState.svelte
- dashboard/src/lib/components/onboarding/PrefillProfileForm.svelte
- dashboard/src/lib/stores/onboarding.ts
- docs/roadmap/x-profile-prefill-onboarding/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- Users can review and edit the inferred setup in one screen without stepping through the old wizard.
- The UI makes it obvious which fields came from X analysis and which still need user judgment.
- The handoff defines the exact payload and navigation behavior for the progressive activation work in Session 05.
```
