# Epic Charter: Obsidian Ghostwriter Edge

## Mission

Give TuitBot a credible power-user edge over Typefully by turning the Obsidian vault into a **hook-first, provenance-tracked content engine** — where users select specific knowledge blocks from their notes, see pre-extracted tweetable hooks, and generate threads grounded in their own writing, all while maintaining strict privacy boundaries across Desktop, Self-host, and Cloud deployments.

## Competitive Positioning

| Capability | TuitBot (current) | TuitBot (Ghostwriter) | Typefully |
|---|---|---|---|
| Vault-backed generation | Chunk selection + RAG | Block-level send + hook-first threads | None |
| Provenance tracking | Immutable snapshot links | Enhanced with heading-anchor deep-links | None |
| Content seeds / hooks | Background extraction only | Exposed in composer as first-class entry point | AI suggestions (no vault grounding) |
| Obsidian integration | File-level deep-links | Section-level deep-links + selection handoff API | None |
| Privacy model | Account-scoped, no raw bodies in API | Deployment-aware content guards (Desktop/Self-host/Cloud) | Cloud-only, opaque |
| Thread drafting | Topic-based generation | Hook-first: pick a hook, expand to thread | Topic/outline-based |

Typefully competes on scheduling polish and analytics. TuitBot's edge is **vault grounding** — the ability to say "this thread came from *that* section of *this* note" with full provenance and a one-click deep-link back to the source. Ghostwriter sharpens that edge from note-level to block-level precision.

## Success Metrics

1. **Hook-to-thread conversion rate**: Percentage of hook selections that result in a published thread (target: >40% of hook selections reach approval queue or direct publish within 7 days of selection).
2. **Vault-to-compose usage**: Percentage of compose sessions that start from the vault panel (target: >25% of all compose sessions within 30 days of Ghostwriter launch).
3. **Citation attachment rate**: Percentage of published content that carries at least one vault provenance link (target: >60% for vault-originated content).
4. **Hook exposure engagement**: Click-through rate on pre-extracted hooks in the vault panel (target: >30% of expanded notes lead to a hook selection).

## Non-Goals

- **Full Obsidian plugin**: Designing or shipping an Obsidian community plugin is out of scope. The API contract for selection ingress is specified; plugin implementation is deferred.
- **Real-time sync**: No live sync between Obsidian and TuitBot. The existing file-watch + re-index pattern is sufficient.
- **Raw body exposure in Cloud mode**: Raw note bodies are never returned over HTTP in Cloud deployments. LLM processing happens server-side; only generated content is returned.
- **New parallel infrastructure**: All changes extend existing vault, compose, provenance, and draft-studio systems. No new databases, services, or sync engines.
- **Obsidian-specific UI in the dashboard**: The dashboard remains Obsidian-agnostic. Deep-links and selection handoff work through URI schemes and API contracts, not Obsidian-specific UI chrome.

## Constraints (from operator rules)

1. Extend existing vault, provenance, compose, draft-studio, and desktop flows before introducing new infrastructure.
2. Keep account-scoped behavior and privacy-safe read APIs; do not expose raw note bodies from server routes unless the privacy model explicitly approves it.
3. Treat Desktop, Self-host, and Cloud as different privacy envelopes; never market a path as local-first unless the runtime can guarantee it.
4. Prefer additive schema and API changes; preserve backward compatibility for current vault and compose flows.
5. Preserve approval queue, scheduled content, and existing automation behavior.
6. If an Obsidian-side capture path is needed, keep it minimal: command-driven selection handoff, no sync engine, no background daemon.
7. Record every material product or architecture decision under `docs/roadmap/obsidian-ghostwriter-edge/`.

## Stakeholders

- **Power users**: Obsidian-native knowledge workers who produce long-form notes and want to extract social content without context-switching.
- **Content creators**: Users who schedule threads and tweets from research notes and want provenance to verify originality.
- **Privacy-conscious users**: Self-hosters and desktop users who chose TuitBot specifically because their content stays local.

## Timeline Scope

Sessions 2-8 implement the Ghostwriter workflow in four phases (see `implementation-map.md`). This charter governs all sessions.
