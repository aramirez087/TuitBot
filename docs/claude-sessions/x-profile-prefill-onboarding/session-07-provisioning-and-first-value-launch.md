# Session 07: Provisioning And First Value Launch

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission
Turn the short onboarding into a live, safe Tuitbot starter state that lands the user in the fastest credible first-value experience.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/session-06-handoff.md
- crates/tuitbot-server/src/routes/settings.rs
- crates/tuitbot-server/src/routes/accounts.rs
- crates/tuitbot-server/src/routes/onboarding.rs
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/lib/stores/accounts.ts

Tasks
1. Materialize the approved onboarding data into config or account state for each deployment mode, reusing the shared onboarding payload and starter-state contract.
2. Preserve safe defaults such as approval mode and deployment-aware source behavior while avoiding extra setup steps that delay first value.
3. Ensure the connected X identity becomes the active account context with synced display name, username, and avatar where available.
4. Route the user to a strong first-value destination with no dead ends, and make recovery idempotent for repeated onboarding completion callbacks.
5. Add tests for successful provisioning, partial failure recovery, duplicate completion events, and account-context correctness across desktop and self-host.

Deliverables
- crates/tuitbot-server/src/routes/onboarding.rs
- crates/tuitbot-server/tests/onboarding_provisioning.rs
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/lib/stores/accounts.ts
- docs/roadmap/x-profile-prefill-onboarding/session-07-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- A newly onboarded user lands in a configured Tuitbot starter state without repeating the onboarding form.
- Desktop and self-host users can complete the same onboarding front half and finalize without being forced through a separate legacy wizard.
- Provisioning is idempotent and safe under retries or duplicated completion callbacks.
- The handoff identifies the instrumentation and polish work left for Session 08.
```
