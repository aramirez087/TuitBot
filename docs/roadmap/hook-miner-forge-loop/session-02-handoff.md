# Session 02 Handoff — Hook Miner UX & Contract Spec

**Date:** 2026-03-22
**Session:** 02 of 11
**Status:** Complete

---

## What Changed

Three documentation files created under `docs/roadmap/hook-miner-forge-loop/`. Zero code changes.

| File | Purpose |
|------|---------|
| `hook-miner-ux.md` | Full UX spec: angle cards, evidence badges, fallback states, loading states, recovery states, copy, non-goals, accessibility |
| `hook-miner-contract.md` | API contract: Rust enums, core types, `POST /api/assist/angles` endpoint, TypeScript types, fallback logic, provenance extension, extraction pipeline |
| `session-02-handoff.md` | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D6 | Angle taxonomy: `Story`, `Listicle`, `HotTake` (3 enums) | Maps to the three dominant content archetypes on X (narrative, list, opinion). Evidence-grounded rather than stylistic. Three forces ranking by evidence strength, not combinatorial explosion. |
| D7 | Evidence taxonomy: `Contradiction`, `DataPoint`, `AhaMoment` (3 enums) | Maps to the most common reasons content resonates (conflict, specificity, surprise). Extraction-oriented — tells the LLM what to look for. |
| D8 | Fallback threshold: `MIN_EVIDENCE_QUALITY = 0.3`, `MIN_EVIDENCE_COUNT = 2` | Below 2 evidence items, angles lack differentiation. Below 0.3 quality, evidence is noise. Constants live in `tuitbot-core`, not server layer. |
| D9 | Related-note influence: evidence -> angle -> hook -> draft (no auto-insertion) | Auto-insertion would break voice consistency and surprise users. Neighbor content is RAG context for draft generation, not pasted text. |
| D10 | UX copy locked: "MINED ANGLES", "NOT ENOUGH SIGNAL", "Mine again", "Use this angle", etc. | Concrete copy prevents implementation-time bike-shedding. Fallback messaging is informative and non-alarming. |

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Evidence extraction LLM quality varies by vault density and note quality | Medium | Regex pre-filter catches obvious data points. Validation rejects invalid evidence. Fallback contract handles sparse results gracefully. |
| `hot_take` angles could produce inflammatory content | Medium | Business profile's `voice` and `persona` constrain tone. Prompt explicitly says "supported by evidence" not "be controversial." User reviews all angles before selection. |
| 3 angles may feel limited compared to 5 hooks | Low | 3 evidence-backed options are more useful than 5 generic options. Fallback to generic hooks is one click away. |
| Token budget (4300 max) may truncate large neighbor contexts | Low | Neighbors included in descending score order. Most useful content comes first. Budget is generous for typical 1-3 accepted neighbors. |
| Two sequential LLM calls (extraction + generation) increase latency | Medium | Loading shimmer and "Mining angles from your notes..." label set expectations. Timeout at 15s with recovery actions. Could be parallelized in future but V1 keeps it sequential for simplicity. |

---

## Required Inputs for Session 03

Session 03 implements the Rust backend: angle types, evidence extraction, and `POST /api/assist/angles`.

**Must read:**
- `docs/roadmap/hook-miner-forge-loop/hook-miner-contract.md` (this session's output — the implementation spec)
- `crates/tuitbot-core/src/content/generator/mod.rs` (existing `generate_hooks` to understand the pattern)
- `crates/tuitbot-core/src/context/graph_expansion.rs` (neighbor fetching)
- `crates/tuitbot-core/src/context/retrieval.rs` (RAG context and `VaultCitation`)
- `crates/tuitbot-server/src/routes/assist/hooks.rs` (existing endpoint to parallel)
- `crates/tuitbot-server/src/routes/assist/mod.rs` (route registration)

**Must create:**
- `crates/tuitbot-core/src/content/angles.rs` — `AngleType`, `EvidenceType`, `EvidenceItem`, `MinedAngle`, `AngleMiningOutput` types
- `crates/tuitbot-core/src/content/evidence.rs` — Evidence extraction engine (regex pre-filter + LLM extraction + validation)
- `crates/tuitbot-core/src/content/generator/angles.rs` — `generate_mined_angles()` function
- `crates/tuitbot-server/src/routes/assist/angles.rs` — `POST /api/assist/angles` handler with request/response DTOs

**Must test:**
- Enum serialization roundtrips (`AngleType`, `EvidenceType`)
- Evidence extraction validation (reject invalid node_ids, truncate long citations, deduplicate)
- Fallback logic (no neighbors, insufficient evidence, low quality)
- API endpoint deserialization (minimal and full request payloads)
- API endpoint response serialization (with and without fallback_reason)

---

## Verification Checklist

- [x] All 7 session tasks addressed:
  - [T1] Replace generic 5 hooks with 3 mined angles — UX spec flow overview + contract routing logic
  - [T2] Angle taxonomy enums — `AngleType { Story, Listicle, HotTake }` with definitions
  - [T3] Evidence taxonomy enums — `EvidenceType { Contradiction, DataPoint, AhaMoment }` with extraction rules
  - [T4] Angle card UX — full card anatomy, ASCII mockup, layout spec
  - [T5] Weak-signal fallback — threshold constants, fallback states, recovery actions, copy
  - [T6] Related-note influence — D9 decision, no auto-insertion, evidence pipeline defined
  - [T7] Copy and non-goals — D10 copy table, 10 non-goals listed
- [x] Exit criteria met:
  - UX spec is concrete enough to build without extra product decisions
  - Fallback path is explicit and non-destructive (state preserved across both actions)
  - Related-note influence defined without surprising draft mutation
- [x] No code changes — documentation only
- [x] All copy is concrete, no placeholders or TBDs
- [x] API contract includes Rust types, DTOs, TypeScript types, error codes
- [x] Backward compatibility guarantees documented for every existing component
