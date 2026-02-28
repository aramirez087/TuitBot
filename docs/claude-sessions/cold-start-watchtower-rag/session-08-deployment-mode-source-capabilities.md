# Session 08: Deployment Mode Source Capabilities

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/session-07-handoff.md, docs/roadmap/BACKLOG-cloud-hosted-tier.md, docs/architecture.md, docs/configuration.md, crates/tuitbot-core/src/config/types.rs, crates/tuitbot-server/src/routes/runtime.rs, and crates/tuitbot-server/src/routes/settings.rs.

Mission
Refactor the source model so local folder access is a deployment capability, not a universal option, while preserving backward compatibility for desktop and self-host users.

Repository anchors
- docs/roadmap/BACKLOG-cloud-hosted-tier.md
- docs/architecture.md
- docs/configuration.md
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/api.ts

Tasks
1. Define a concrete capability matrix for desktop app, self-hosted browser, and cloud-hosted mode covering local_folder, manual_local_path, google_drive, and future remote providers.
2. Extend config and API contracts so source settings can be validated against deployment capabilities instead of assuming every source is legal everywhere.
3. Add a server-exposed capability payload, preferably via the existing runtime status surface or a nearby settings surface, that the UI can consume without platform guessing.
4. Preserve existing local-folder configs by treating them as valid in desktop and self-host modes, with a clear rejection path in cloud mode.
5. Document the corrected source model and the migration away from the universal folder-picker assumption.

Deliverables
- docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md
- docs/architecture.md
- docs/configuration.md
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/api.ts
- docs/roadmap/cold-start-watchtower-rag/session-08-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings

Exit criteria
- The backend exposes a capability model the frontend can trust.
- Cloud mode can reject local-only source types without breaking desktop or self-host flows.
- The migration rules for pre-existing local configs are written down.
```
