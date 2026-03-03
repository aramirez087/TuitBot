# Session 01: X Access Charter

Paste this into a new Claude Code session:

```md
Continuity
- Start from the current repository state and read files directly.

Mission
Define the product, config, and safety contract for a no-X-API-key local/LAN mode that lives in the existing X access settings surface.

Repository anchors
- `README.md`
- `config.example.toml`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `dashboard/src/routes/(app)/settings/XApiSection.svelte`
- `dashboard/src/lib/components/onboarding/XApiStep.svelte`
- `crates/tuitbot-mcp/src/provider/scraper.rs`

Tasks
1. Audit the current meaning of `x_api.provider_backend` and `x_api.scraper_allow_mutations`, including where they already work and where they are hidden.
2. Write `docs/roadmap/no-x-api-local-mode/charter.md` with the problem statement, user promise, non-goals, and explicit deployment boundaries: desktop and LAN/self-hosted allowed, cloud disallowed.
3. Write `docs/roadmap/no-x-api-local-mode/x-access-contract.md` that locks in the v1 contract: `provider_backend = "x_api"` stays the paid path, `provider_backend = "scraper"` becomes the no-key path, client ID and client secret are optional in scraper mode, and writes must fail closed or queue with actionable guidance when transport confidence is insufficient.
4. Decide whether the product runtime should reuse `provider_backend` directly or only wrap it with UI copy, and document the decision without adding duplicate config knobs.
5. Capture a concrete implementation split for Sessions 02 through 04, including the key files each session must touch.
6. End with a handoff.

Deliverables
- `docs/roadmap/no-x-api-local-mode/charter.md`
- `docs/roadmap/no-x-api-local-mode/x-access-contract.md`
- `docs/roadmap/no-x-api-local-mode/session-01-handoff.md`

Quality gates
- No runtime feature work in this session; if you touch production code, explain why in the handoff.

Exit criteria
- The charter defines user-visible behavior for the no-key mode.
- The contract names the exact config fields and safe degraded write behavior.
- The handoff gives Session 02 enough context to implement without reopening scope.
```
