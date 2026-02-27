# Session 02: Composer Data Model And API Contract

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.
Mission: Implement a high-fidelity data contract that enables faster, safer editing than Typefully.

Repository anchors:
- `dashboard/src/lib/api.ts`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `crates/tuitbot-server/tests/api_tests.rs`

Tasks:
1. Define a typed thread payload schema with stable block IDs, ordered tweet blocks, and per-block media references.
2. Update compose and draft endpoints to validate and persist the new contract without breaking tweet-only flows.
3. Add robust validation errors for malformed ordering, duplicate IDs, empty parts, and invalid media references.
4. Update dashboard API client types and adapters to use the new schema.
5. Add API tests for compatibility, optimistic reorder safety, and rejection paths.

Deliverables:
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `dashboard/src/lib/api.ts`
- `docs/roadmap/typefully-composer-ui-parity/session-02-api-contract.md`
- `docs/roadmap/typefully-composer-ui-parity/session-02-handoff.md`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria:
- Backend accepts ordered thread payloads with per-item media.
- Stable block IDs survive edit/reorder roundtrips.
- Legacy tweet compose flows still pass.
- Handoff lists UI integration points for Session 03.
```
