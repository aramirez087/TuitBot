# Session 02: Onboarding and Recovery UX

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Implement the frontend and onboarding changes that make the web passphrase visible, explicit, and recoverable for first-run users.

Repository anchors
- docs/roadmap/passphrase-lifecycle-ux/charter.md
- dashboard/src/routes/+layout.svelte
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/components/onboarding/ClaimStep.svelte
- dashboard/src/routes/login/+page.svelte

Tasks
1. Implement the chartered web-flow fix so a user cannot finish first-run onboarding without seeing a clear passphrase handoff or a clear recovery path.
2. Fix the unconfigured-but-claimed path so it no longer looks like onboarding completed without a passphrase and instead sends the user to the correct state with explicit guidance.
3. Align login and onboarding copy with the supported reset flows while keeping Tauri and bearer-token behavior unchanged.
4. Record any manual verification steps that remain uncovered by automated checks in the handoff.

Deliverables
- dashboard/src/routes/+layout.svelte
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/components/onboarding/ClaimStep.svelte
- dashboard/src/routes/login/+page.svelte
- docs/roadmap/passphrase-lifecycle-ux/session-02-handoff.md

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Fresh web onboarding exposes the passphrase or clear recovery guidance before redirecting into app content.
- Unconfigured claimed instances no longer strand the user in an ambiguous onboarding or login state.
- Tauri flows still reach onboarding or dashboard without a passphrase claim step.
```
