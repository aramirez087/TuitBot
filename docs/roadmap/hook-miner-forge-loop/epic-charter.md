# Epic Charter — Hook Miner + Forge Loop

**Date:** 2026-03-22
**Epic owner:** Session-automated (Rust + product engineer)
**Status:** Chartered

---

## Mission

Turn TuitBot's Ghostwriter from "pick a generic hook style" to "choose an evidence-mined angle with closed-loop analytics" by:

1. **Hook Miner** — replacing the first compose step (5 generic hook styles) with 3 evidence-backed angle cards sourced from the vault graph, so every draft starts from a concrete insight rather than a stylistic template.

2. **Forge** — extending the existing `tuitbot:` frontmatter loopback into an opt-in analytics enrichment layer that writes performance data (impressions, likes, engagement rate) back to the source notes that produced the content.

Together these make the Ghostwriter loop feel closed: notes produce angles, angles produce tweets, tweet performance flows back to notes.

---

## Success Metrics

| Metric | Target | Measurement Point |
|--------|--------|-------------------|
| Angle-to-draft conversion | ≥ 50% | Mined angles that produce a draft (not abandoned) |
| Fallback-to-generic rate | ≤ 30% | Sessions where graph evidence is too sparse and generic hooks are shown instead |
| Forge sync success rate | ≥ 95% | Eligible notes that update frontmatter without error |
| Thread outcome completeness | 100% | Ghostwriter-composed threads that persist root + child IDs in `threads`/`thread_tweets` |

---

## Non-Goals

These are explicitly out of scope for this epic:

- **Multi-hop graph expansion.** Stay at 1-hop per the Backlink Synthesizer decision. Multi-hop adds complexity without proven user value.
- **Real-time analytics streaming.** Forge syncs on a periodic schedule (poll-based), not via live WebSocket updates.
- **Engagement prediction or ML scoring in Hook Miner.** Evidence is displayed to the user as-is. No predictive models, no "this angle will perform better" claims.
- **New parallel metadata system.** Forge extends the existing `tuitbot:` frontmatter contract. No sidecar database, no separate sync daemon, no new Obsidian plugin commands.
- **Obsidian plugin redesign.** The selection ingress pipeline is complete and stable. No new plugin commands for this epic.
- **Thread composition UX overhaul.** Thread normalization fixes the publish path (approval poster posts reply chains, creates thread records). It does not redesign how users compose threads in the dashboard.
- **Historical backfill.** Forge writes analytics to notes going forward. No batch process to backfill performance data for previously published tweets.

---

## Product Stance

### Hook Miner is evidence-first UX

The user sees why an angle was suggested: the source note, the graph relationship, and the specific evidence (a data point, a contradiction, an aha moment). Every angle card shows its provenance. If the evidence is weak or the graph is sparse, the system falls back gracefully to the existing generic hook picker — no hidden degradation, no fake confidence.

### Forge is an extension, not a parallel system

Forge writes to the same `tuitbot:` frontmatter array that the loopback already uses. It adds analytics fields to existing entries and optionally adds a note-level summary. The user sees performance data next to their source notes in Obsidian without installing anything new or learning a new system.

### Thread normalization is a transparent fix

The approval poster currently posts thread tweets as standalone tweets. This is a bug, not a feature. Fixing it to post reply chains and create proper `thread`/`thread_tweets` records is a correctness fix that unblocks Forge's thread analytics. No user-facing toggle, no opt-in — threads should just work.

---

## Rollout Stance

| Component | Rollout Strategy | Default State |
|-----------|-----------------|---------------|
| Hook Miner | Additive to HookPicker; shown when graph evidence meets quality threshold | On (with automatic fallback to generic) |
| Forge | Opt-in via `analytics_sync_enabled` account setting | Off |
| Thread normalization | Transparent fix in approval poster | Always on (correctness fix) |

### Deployment modes

- **Desktop (Tauri):** Full Hook Miner + Forge. Local filesystem loopback fully functional.
- **Self-host:** Full Hook Miner + Forge. Loopback depends on vault source being `local_fs`.
- **Cloud:** Hook Miner works (graph data available server-side). Forge loopback disabled (no filesystem access). Analytics still tracked in DB — just not written to notes.

---

## Constraints Inherited from Prior Epics

1. **Account scoping** — all queries, storage, and loopback are scoped to `account_id`. No cross-account data leakage.
2. **Privacy envelopes** — cloud mode never exposes `selected_text` in GET responses. Forge analytics do not include raw tweet content in frontmatter.
3. **Raw-text limits** — selection text capped at 10k chars. Hook Miner input bounded by the same limit.
4. **Server boundary** — `tuitbot-server` owns zero business logic. All angle mining, evidence extraction, and analytics aggregation live in `tuitbot-core`.
5. **Dependency rule** — Toolkit ← Workflow ← Autopilot. Hook Miner extraction goes in Toolkit or Workflow. Forge sync goes in Autopilot (watchtower).

---

## Key Design Decisions

### D1: Hook Miner extends `generate_hooks`, not replaces it

The existing `POST /api/assist/hooks` endpoint and `generate_hooks()` function remain as the fallback path. A new `POST /api/assist/angles` endpoint calls `generate_mined_angles()` which uses accepted graph neighbors as evidence. When evidence quality is below threshold, the frontend falls back to the existing hook picker flow.

### D2: Forge reuses `LoopBackEntry` with additional fields

The `LoopBackEntry` struct gains optional analytics fields: `impressions`, `likes`, `retweets`, `replies`, `engagement_rate`, `performance_score`, `synced_at`. The `write_metadata_to_file()` function gains an update path (match by `tweet_id`, merge fields) alongside its existing append path.

### D3: Thread normalization is a prerequisite for Forge

The approval poster must post thread tweets as a reply chain (each tweet replies to the previous) and create `thread`/`thread_tweets` records for Ghostwriter-composed threads. Without this, Forge cannot aggregate thread metrics or trace thread performance back to source notes.

### D4: Analytics needs thread-level aggregation

A new aggregation function computes thread-level metrics by summing child tweet performance. This is computed on-demand (not stored in a separate table) to avoid schema proliferation.

### D5: Backlink Synthesizer work is complete and not re-scoped

Graph expansion, neighbor scoring, suggestion cards, synthesis toggle, and provenance edge fields are all shipped. This epic builds on top of that work without modifying it.
