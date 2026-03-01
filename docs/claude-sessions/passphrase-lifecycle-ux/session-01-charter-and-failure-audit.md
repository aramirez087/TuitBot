# Session 01: Charter and Failure Audit

Paste this into a new Claude Code session:

```md
Mission
Audit the passphrase lifecycle failures and write the implementation charter for fixing them.

Repository anchors
- docs/roadmap/fresh-install-auth-ux/charter.md
- dashboard/src/routes/+layout.svelte
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/components/onboarding/ClaimStep.svelte
- dashboard/src/routes/login/+page.svelte
- crates/tuitbot-server/src/main.rs
- crates/tuitbot-server/src/auth/routes.rs
- crates/tuitbot-server/src/routes/lan.rs
- crates/tuitbot-core/src/auth/passphrase.rs

Tasks
1. Reconstruct from code how the three reported failures happen today: no passphrase after onboarding, noisy `--reset-passphrase` output, and stale in-memory auth after an out-of-band reset.
2. Compare the current implementation with `docs/roadmap/fresh-install-auth-ux/charter.md` and note what is incomplete, regressed, or no longer matches reality.
3. Write a charter with the problem statement, exact failure modes, design decisions, implementation slices for Sessions 02-04, acceptance scenarios, and risks.
4. Create the handoff with explicit inputs for Session 02.

Deliverables
- docs/roadmap/passphrase-lifecycle-ux/charter.md
- docs/roadmap/passphrase-lifecycle-ux/session-01-handoff.md

Exit criteria
- The charter is detailed enough to execute the remaining sessions without reopening discovery.
- Each reported issue has a root-cause hypothesis tied to concrete file paths.
- Session 02 can start from the charter and handoff alone.
```
