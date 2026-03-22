# Session 06: Forge Data Contract And Frontmatter Schema

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Define the exact Forge frontmatter contract for publish history, analytics enrichment, and thread outcomes.

Repository anchors
- crates/tuitbot-core/src/automation/watchtower/loopback.rs
- crates/tuitbot-core/src/automation/approval_poster.rs
- crates/tuitbot-core/src/storage/provenance.rs
- docs/configuration.md
- dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte
- docs/roadmap/hook-miner-forge-loop/current-state-audit.md

Tasks
1. Keep the structured `tuitbot:` array as the canonical audit trail and define its additive per-entry fields.
2. Require every entry to support `tweet_id`, `url`, `published_at`, `type`, `status`, optional `thread_url`, optional `child_tweet_ids`, `impressions`, `likes`, `retweets`, `replies`, `engagement_rate`, `performance_score`, and `synced_at`.
3. Define the note-level summary fields exactly: `tuitbot_social_performance`, `tuitbot_best_post_impressions`, `tuitbot_best_post_url`, and `tuitbot_last_synced_at`.
4. Define idempotency and matching rules: same `tweet_id` updates the same entry; summary fields always reflect the highest-impression synced entry; thread entries use the root tweet id as `tweet_id`.
5. Define how a thread is represented as one note-level outcome with root tweet id, child tweet ids, and aggregated metrics.
6. Write concrete YAML examples for a single tweet note and a thread note.

Deliverables
- docs/roadmap/hook-miner-forge-loop/forge-frontmatter-contract.md
- docs/roadmap/hook-miner-forge-loop/forge-thread-contract.md
- docs/roadmap/hook-miner-forge-loop/session-06-handoff.md

Quality gates
- No code changes required unless a tiny clarifying doc fix is necessary.

Exit criteria
- The YAML shape is unambiguous.
- Matching, aggregation, and idempotency rules are concrete.
- The contract is compatible with the current `tuitbot` writeback path.
```
