# Session 08: Measurement Copy And Fallbacks

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Mission
Instrument the new onboarding funnel and harden the copy and fallback states so the experience is measurable, trustworthy, and resilient.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/session-07-handoff.md
- .claude/product-marketing-context.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/lib/api/client.ts
- docs/getting-started.md

Tasks
1. Add or document the analytics events needed to measure entry, X auth, analysis success, edits, onboarding completion, checklist progress, capability unlocks, and first-value completion.
2. Harden the UI and server behavior for auth loss, sparse profiles, low-confidence inference, deferred-capability friction, and unsupported deployment modes.
3. Run a copy sweep across onboarding and first-run activation surfaces so the language stays direct, practical, and consistent with the product context.
4. Produce an experiment backlog for future optimization without leaving unresolved placeholders in the implemented flow.
5. Update onboarding-facing docs to match the new experience where appropriate.

Deliverables
- docs/roadmap/x-profile-prefill-onboarding/funnel-metrics.md
- docs/roadmap/x-profile-prefill-onboarding/experiment-backlog.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- docs/roadmap/x-profile-prefill-onboarding/session-08-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- The new funnel can be measured end to end and the docs name the key activation metrics.
- Failure states are explicit and do not dump users back into confusion or data loss.
- The handoff gives Session 09 a clear validation matrix and remaining risk list.
```
