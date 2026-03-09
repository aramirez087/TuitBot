# Session 02: Unified Entry And X Auth Bootstrap

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Build the unified onboarding entry path so users in every deployment mode can start with X sign-in instead of the long wizard.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/epic-charter.md
- docs/roadmap/x-profile-prefill-onboarding/target-flow.md
- dashboard/src/routes/+layout.svelte
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/stores/auth.ts
- dashboard/src/lib/stores/accounts.ts
- dashboard/src/lib/api/client.ts
- crates/tuitbot-server/src/auth/routes.rs
- crates/tuitbot-server/src/routes/x_auth.rs
- crates/tuitbot-server/src/lib.rs

Tasks
1. Introduce a shared onboarding entry state with Continue with X as the primary CTA across desktop and self-host, while keeping explicit fallback behavior only where X auth is unavailable.
2. Add the client and server plumbing for a short-lived onboarding session that can survive the X auth bootstrap without requiring the full config first.
3. Reuse or adapt the existing X auth flow so onboarding returns enough identity metadata to power later analysis without inventing a second auth stack.
4. Preserve deployment-specific requirements behind the shared bootstrap instead of forking the entire flow early.
5. Add focused tests for route gating, bootstrap success, user cancel, expired onboarding state, and deployment-aware fallback behavior.

Deliverables
- dashboard/src/lib/stores/onboarding-session.ts
- dashboard/src/routes/onboarding/+page.svelte
- crates/tuitbot-server/src/routes/onboarding.rs
- docs/roadmap/x-profile-prefill-onboarding/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- Users in every supported deployment mode can initiate onboarding with X auth and resume in a valid onboarding session.
- Desktop and self-host users are not regressed into a broken or separate first step unless the architecture truly requires it.
- The handoff records exact API contracts and edge cases for Session 03.
```
