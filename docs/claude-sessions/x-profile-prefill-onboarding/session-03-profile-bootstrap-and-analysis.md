# Session 03: Profile Bootstrap And Analysis

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Build the profile-analysis pipeline that reads the connected X account and returns normalized onboarding suggestions with provenance.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/inference-contract.md
- docs/roadmap/x-profile-prefill-onboarding/session-02-handoff.md
- crates/tuitbot-server/src/routes/onboarding.rs
- crates/tuitbot-server/src/routes/accounts.rs
- crates/tuitbot-server/src/routes/x_auth.rs
- crates/tuitbot-core/src/context/author.rs
- crates/tuitbot-core/src/context/topics.rs
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- dashboard/src/lib/stores/onboarding.ts

Tasks
1. Add an onboarding analysis endpoint that returns raw profile identity, recent-post sample metadata, inferred suggestions, confidence, and provenance fields.
2. Reuse the existing profile sync and X client machinery where possible, and fetch enough authored content to infer audience, topics, tones, goals, and posting cadence.
3. Implement deterministic heuristics first and use LLM enrichment only where it materially improves the output; document sparse-account fallbacks.
4. Persist only the minimum data needed for onboarding continuity, and avoid silently storing unnecessary raw profile content.
5. Add tests for rich-profile, sparse-profile, no-post, auth-failure, and low-confidence cases.

Deliverables
- crates/tuitbot-server/src/routes/onboarding.rs
- crates/tuitbot-server/tests/onboarding_analysis.rs
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- docs/roadmap/x-profile-prefill-onboarding/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- The onboarding analysis API can prefill the required profile fields and explain where each suggestion came from.
- Low-data accounts degrade gracefully instead of blocking the flow.
- The handoff tells Session 04 exactly how to consume the API in the UI.
```
