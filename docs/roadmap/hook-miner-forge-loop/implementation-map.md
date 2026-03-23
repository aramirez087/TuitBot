# Implementation Map — Hook Miner + Forge Loop

**Date:** 2026-03-22
**Sessions:** 11 total (S01 = this audit, S02–S11 = implementation)
**Critical path:** S07 (thread normalization) blocks S08 (Forge sync engine)

---

## Track A: Hook Miner (Sessions 02–05)

### Session 02 — Hook Miner Product Spec

**Goal:** Define the angle/evidence taxonomy, UX copy, fallback contract, and API shape before writing code.

**Deliverables:**
- Angle taxonomy: `story`, `listicle`, `hot_take` (with definitions and examples)
- Evidence taxonomy: `contradiction`, `data_point`, `aha_moment` (with extraction rules)
- Fallback contract: when to show mined angles vs. generic hooks
- API contract for `POST /api/assist/angles`
- UX copy for angle cards (title, evidence snippet, source attribution)

**Repository anchors:**
- `dashboard/src/lib/components/composer/VaultSelectionReview.svelte`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/HookPicker.svelte`
- `dashboard/src/lib/components/composer/GraphSuggestionCards.svelte`
- `crates/tuitbot-server/src/routes/assist/hooks.rs`

**No code changes.** Spec session only.

---

### Session 03 — Hook Miner Extraction Engine

**Goal:** Implement `generate_mined_angles()` in `tuitbot-core` that takes accepted graph neighbors as evidence sources and returns 3 ranked angle cards.

**Deliverables:**
- `generate_mined_angles()` function in content generator
- Evidence extraction from graph neighbor snippets and chunk content
- Angle ranking by evidence strength
- Structured output parsing for angle cards
- Unit tests for extraction and ranking

**Repository anchors:**
- `crates/tuitbot-core/src/content/generator/mod.rs`
- `crates/tuitbot-core/src/content/generator/parser.rs`
- `crates/tuitbot-core/src/context/graph_expansion.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`

---

### Session 04 — Hook Miner Composer Integration

**Goal:** Wire angle cards into the dashboard compose flow, replacing the hook picker first step when evidence is available and falling back to generic hooks when it isn't.

**Deliverables:**
- `AngleCards.svelte` component (or integrated into VaultSelectionReview)
- Fallback logic: if `generate_mined_angles()` returns insufficient evidence → show HookPicker
- API endpoint `POST /api/assist/angles`
- VaultSelectionReview and FromVaultPanel wiring to pass accepted neighbors to angle generation

**Repository anchors:**
- `dashboard/src/lib/components/composer/VaultSelectionReview.svelte`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/HookPicker.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`

---

### Session 05 — Hook Miner Provenance & Draft Semantics

**Goal:** Ensure angle provenance (which evidence, from which neighbor, via which edge) flows through compose → approval → publish lifecycle.

**Deliverables:**
- Angle provenance attached to compose request
- Provenance includes evidence type, source neighbor node_id, edge relationship
- `copy_links_for()` handles angle-specific provenance fields
- Tests for provenance lifecycle

**Repository anchors:**
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-core/src/content/compose/mod.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`

---

## Track B: Forge (Sessions 06–09)

### Session 06 — Forge Data Contract

**Goal:** Define the exact YAML schema for analytics-enriched frontmatter entries and the thread performance contract.

**Deliverables:**
- Extended `LoopBackEntry` schema with analytics fields
- Thread-level frontmatter contract (root entry links to child entries)
- YAML examples for single tweet, thread root, thread child
- Note-level summary field spec (`tuitbot_summary:`)
- Update-path semantics (match by `tweet_id`, merge analytics fields)

**Repository anchors:**
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`

**No code changes.** Contract spec only.

---

### Session 07 — Thread Publish Normalization ⚠️ Critical Path

**Goal:** Fix the approval poster to post thread tweets as reply chains and create proper `thread`/`thread_tweets` records for Ghostwriter-composed threads.

**Deliverables:**
- Approval poster posts thread tweets sequentially with `in_reply_to_tweet_id`
- `thread` and `thread_tweets` records created for Ghostwriter-composed threads
- Provenance copied to `thread` entity type
- `thread_url` populated in loopback entries
- Tests for reply-chain posting and thread record creation

**Repository anchors:**
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-core/src/storage/threads.rs`
- `crates/tuitbot-core/src/content/compose/transforms.rs`
- `crates/tuitbot-core/src/storage/drafts.rs`
- `crates/tuitbot-server/src/routes/approval/handlers.rs`

**Risk:** This touches the posting hot path. Careful sequencing needed to avoid double-posting or lost tweets.

---

### Session 08 — Forge Sync Engine

**Goal:** Implement the periodic analytics enrichment loop that reads tweet performance, aggregates thread metrics, and writes enriched frontmatter back to source notes.

**Deliverables:**
- `forge_sync()` function in watchtower
- Thread-level aggregation: sum child tweet metrics into thread-level totals
- Extended `write_metadata_to_file()` with update path (match by `tweet_id`, merge)
- Note-level summary computation
- Idempotent sync (re-running produces same result)
- Tests for sync, aggregation, and frontmatter update

**Repository anchors:**
- `crates/tuitbot-core/src/automation/watchtower/analytics_loop.rs`
- `crates/tuitbot-core/src/storage/adapters/storage.rs`
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`
- `crates/tuitbot-core/src/storage/analytics/`
- `crates/tuitbot-core/src/storage/threads.rs`

**Blocked by:** S07 (thread normalization). Cannot aggregate thread metrics without linked child tweet IDs.

---

### Session 09 — Forge Settings & Prompt UX

**Goal:** Add opt-in toggle for Forge sync and first-sync prompt in the dashboard.

**Deliverables:**
- `analytics_sync_enabled` account setting (default off)
- Settings UI toggle in Content Sources section
- First-sync eligibility display (how many notes are eligible, what data will be written)
- Confirmation prompt before first sync

**Repository anchors:**
- `dashboard/src/lib/components/settings/ContentSourcesSection.svelte`
- `docs/roadmap/hook-miner-forge-loop/configuration.md` (if needed)

---

## Track C: Instrumentation & Release (Sessions 10–11)

### Session 10 — Instrumentation

**Goal:** Add typed analytics events for Hook Miner and Forge interactions.

**Deliverables:**
- Events: `angle_shown`, `angle_selected`, `angle_dismissed`, `fallback_to_generic`
- Events: `forge_sync_started`, `forge_sync_completed`, `forge_sync_failed`, `forge_sync_skipped`
- Events: `thread_chain_posted`, `thread_record_created`
- Frontend funnel tracking for angle → draft → approval → publish → forge sync
- Rust telemetry for sync timing and error rates

**Repository anchors:**
- `dashboard/src/lib/stores/analytics/backlinkFunnel.ts`
- `dashboard/src/lib/stores/analytics/funnel.ts`
- `crates/tuitbot-core/src/telemetry.rs` (or equivalent)

---

### Session 11 — Validation & Release Readiness

**Goal:** QA matrix, charter metric verification, and release assessment.

**Deliverables:**
- QA matrix covering all Hook Miner and Forge paths
- Charter metric measurement plan
- Release readiness checklist
- Final handoff with residual risks and post-launch monitoring plan

**Repository anchors:** All of the above.

---

## Dependency Graph

```
S01 (audit) ─── done
 │
 ├── S02 (HM spec) → S03 (HM engine) → S04 (HM composer) → S05 (HM provenance)
 │
 ├── S06 (Forge contract) → S07 (thread norm) → S08 (Forge sync) → S09 (Forge settings)
 │                              ⚠️ critical path
 │
 └── S10 (instrumentation) → S11 (validation)
```

Tracks A and B can proceed in parallel up to S07. S08 depends on S07. S10 depends on S05 and S09 being complete. S11 is the final gate.

---

## Thread and Analytics Normalization Gap

This is the single most important gap in the current codebase for this epic:

**Problem:** The approval poster (`approval_poster.rs`) routes `thread_tweet` action types to `post_tweet()` — standalone posting with no reply-to relationship. This means:
1. Thread tweets are posted as independent tweets, not a visible thread on X.
2. No `thread`/`thread_tweets` records are created for Ghostwriter-composed threads.
3. Provenance is not copied to `thread` entity type.
4. Analytics cannot aggregate thread child tweets because they aren't linked.
5. Forge cannot write thread-level performance to source notes.

**Fix (Session 07):** The approval poster must:
- Detect thread tweet groups (same compose session or thread_id).
- Post sequentially with `in_reply_to_tweet_id` linking each tweet to the previous.
- Create `thread` + `thread_tweets` records.
- Copy provenance to `thread` entity type.
- Populate `thread_url` in loopback entries.

**Why this is the critical path:** Every Forge feature that touches threads depends on this fix. Without it, thread analytics are impossible and Forge degrades to tweet-only.
