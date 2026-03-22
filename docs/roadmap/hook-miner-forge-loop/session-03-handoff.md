# Session 03 Handoff — Hook Miner Extraction Engine

**Date:** 2026-03-22
**Session:** 03 of 11
**Status:** Complete

---

## What Changed

Four new Rust source files and two documentation files. Three existing files modified to wire up the new modules and route.

| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/src/content/angles.rs` | Created | Domain types: `AngleType`, `EvidenceType`, `EvidenceItem`, `MinedAngle`, `AngleMiningOutput`, constants, confidence helper |
| `crates/tuitbot-core/src/content/evidence.rs` | Created | Evidence extraction engine: regex pre-filter, LLM extraction prompt, JSON parsing with fallback, validation (node ID check, truncation, dedup) |
| `crates/tuitbot-core/src/content/generator/angles.rs` | Created | Angle generation pipeline: full orchestration, angle generation prompt, response parser |
| `crates/tuitbot-server/src/routes/assist/angles.rs` | Created | `POST /api/assist/angles` handler with request/response DTOs, neighbor content fetching, citation building |
| `crates/tuitbot-core/src/content/mod.rs` | Modified | Added `pub mod angles;`, `pub mod evidence;`, and re-exports |
| `crates/tuitbot-core/src/content/generator/mod.rs` | Modified | Added `pub(crate) mod angles;` and `generate_mined_angles` delegation method on `ContentGenerator` |
| `crates/tuitbot-server/src/routes/assist/mod.rs` | Modified | Added `pub mod angles;` |
| `crates/tuitbot-server/src/lib.rs` | Modified | Registered `/assist/angles` route |
| `docs/roadmap/hook-miner-forge-loop/hook-miner-api-contract.md` | Created | API contract documentation |
| `docs/roadmap/hook-miner-forge-loop/session-03-handoff.md` | Created | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D11 | `angles.rs` and `evidence.rs` as siblings under `content/`, generator submodule `angles.rs` under `generator/` | Evidence extraction is distinct from generation. The generator submodule handles only the generation half. |
| D12 | Evidence extraction uses a single LLM call with all neighbors, not per-neighbor | Reduces latency. Cross-note contradictions require seeing all neighbors at once. |
| D13 | Angle parser follows `parse_hooks_response` pattern (prefix lines + `---` separators) | Proven robust. Same cognitive patterns. New prefixes: `ANGLE_TYPE:`, `SEED_TEXT:`, `RATIONALE:`, `EVIDENCE_IDS:`. |
| D14 | Endpoint named `/api/assist/angles` per contract | Contract (S02) specified this name. Task description used "hook-miner" as conceptual name. |
| D15 | `NeighborContent` lives in `evidence.rs` | Projection for the extraction pipeline. Graph expansion module shouldn't know about evidence concerns. |
| D16 | Truncate over-long citations instead of rejecting | Partial citations still provide value. 120-char limit is for UI display. |
| D17 | `temperature: 0.3` for extraction, `0.8` for generation | Extraction should be precise. Generation benefits from creativity. |
| D18 | Evidence JSON parsing has 3 fallback layers: direct parse → code block extraction → bracket finding | LLMs frequently wrap JSON in markdown code blocks or add preamble text. Robust parsing avoids false-negative empty evidence. |
| D19 | `accepted_neighbor_ids` is required (not optional) in the request | The mining pipeline depends on neighbor notes for evidence. Without neighbors, there is nothing to mine — returning a 400 is more informative than silently returning a fallback. |

---

## Test Coverage

| Module | Tests | Status |
|--------|-------|--------|
| `content::angles` | 9 tests: serde roundtrips, snake_case serialization, confidence assignment, output serialization (normal + fallback) | All pass |
| `content::evidence` | 13 tests: pre-filter (5 data patterns + empty), validation (4 rules), JSON parsing (3 formats) | All pass |
| `generator::angles` | 7 tests: parser (well-formatted, partial, empty), angle type parsing, evidence mapping | All pass |
| `routes::assist::angles` | 7 tests: request deserialization (2), response serialization (3), snippet truncation (2) | All pass |
| **Total new tests** | **36** | **All pass** |
| **Total workspace tests** | **567** | **All pass** |

---

## Quality Gates

```
cargo fmt --all && cargo fmt --all --check     ✅
RUSTFLAGS="-D warnings" cargo test --workspace  ✅ (567 passed)
cargo clippy --workspace -- -D warnings          ✅
```

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Evidence extraction LLM quality varies by vault density | Medium | Regex pre-filter catches obvious data points. Validation rejects invalid evidence. Fallback contract handles sparse results. |
| Two sequential LLM calls increase latency (~2-4s total) | Medium | Loading state in UI (Session 04). Token budgets are modest (500 + 800 output tokens). No retry loops in V1. |
| `hot_take` angles could produce inflammatory content | Medium | Business profile's voice/persona constrain tone. Prompt says "supported by evidence." User reviews all angles. |
| generator/mod.rs is 880 lines (over 500-line limit) | Low | Pre-existing violation. We added only ~15 lines (delegation method + mod declaration). Refactor is out of scope for this session. |

---

## Required Inputs for Session 04

Session 04 implements the frontend: angle card components, fallback states, and the mined-angle flow in the Ghostwriter Composer.

**Must read:**
- `docs/roadmap/hook-miner-forge-loop/hook-miner-ux.md` (UX spec from S02)
- `docs/roadmap/hook-miner-forge-loop/hook-miner-api-contract.md` (API contract from S03 — this session)
- `dashboard/src/routes/compose/` (existing Ghostwriter Composer)
- `dashboard/src/lib/stores/` (existing store patterns)
- `CLAUDE.md` frontend section (Svelte 5 runes, design tokens)

**Must create:**
- Angle card component with evidence badges
- Fallback state component ("NOT ENOUGH SIGNAL")
- API client function for `POST /api/assist/angles`
- Integration into the Composer flow (after neighbor acceptance step)

**Must preserve:**
- Existing hook generation flow (`/api/assist/hooks`) as fallback
- Current Ghostwriter visual language
- Tab indentation in Svelte files

---

## Architecture Summary

```
                    ┌─────────────────────┐
                    │ POST /assist/angles  │  (tuitbot-server)
                    │ routes/assist/       │
                    │ angles.rs            │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │ ContentGenerator     │  (tuitbot-core)
                    │ .generate_mined_    │
                    │  angles()           │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
    ┌─────────────┐  ┌──────────────┐  ┌────────────┐
    │ evidence.rs │  │ evidence.rs  │  │ angles.rs  │
    │ pre_filter  │  │ extract_     │  │ generate_  │
    │ _data_      │  │ evidence()   │  │ mined_     │
    │ points()    │  │ + validate   │  │ angles()   │
    │ (regex)     │  │ _evidence()  │  │ + parser   │
    └─────────────┘  └──────────────┘  └────────────┘
```
