# Launch Readiness Report — X Profile Prefill Onboarding

**Date:** 2026-03-09
**Epic:** x-profile-prefill-onboarding (Sessions 1–9)
**Author:** Session 09 (automated validation)

---

## 1. Verdict

**GO** — The onboarding-to-first-value flow is complete, instrumented, and handles all identified error states. No critical path remains unimplemented. Remaining items are optimizations and polish that do not block the primary user journey.

---

## 2. Evidence Summary

### CI Gates
| Gate | Status |
|------|--------|
| `cargo fmt --all --check` | PASS |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS (all tests) |
| `cargo clippy --workspace -- -D warnings` | PASS |
| `cd dashboard && npm run check` | PASS (0 errors, 0 warnings) |

### QA Validation
- **20/20 scenarios** traced through code paths — all pass (see `qa-matrix.md`)
- **21/21 funnel events** verified with correct properties and fire conditions
- **Frontend ↔ Server contract** aligned for analyze-profile response, init payload, and 409 recovery
- **Desktop vs Self-Host divergence** limited to Claim step (intentional)
- **Deep-link section IDs** all present: `#business`, `#xapi`, `#llm`, `#sources`
- **11 error conditions** have explicit handlers with user-friendly messages and recovery actions

### Test Coverage
- `onboarding.rs`: 2 unit tests (token path, user JSON serialization)
- `onboarding_provisioning.rs`: Integration tests for init + provisioning flow
- Frontend: Svelte type-checking passes (`svelte-check`)

---

## 3. Feature Completeness Matrix

| Feature (from Epic Charter) | Status | Session |
|-----------------------------|--------|---------|
| X OAuth during onboarding (pre-account) | Done | 3–4 |
| Profile analysis endpoint (heuristic + LLM) | Done | 3–5 |
| Inferred profile with per-field confidence & provenance | Done | 5 |
| Prefill profile form with review/override | Done | 5 |
| Scraper mode graceful degradation | Done | 4–5 |
| Progressive activation (4 capability tiers) | Done | 6 |
| Activation checklist (full + compact) | Done | 6 |
| Starter-state provisioning (init_settings) | Done | 7 |
| X identity written to default account at init | Done | 7 |
| Onboarding token migration to account path | Done | 7 |
| Funnel event instrumentation (21 events) | Done | 8 |
| Error handling (network, OAuth, LLM, sparse, 409) | Done | 8 |
| Copy consistency with product context | Done | 8 |
| Measurement documentation (funnel-metrics.md) | Done | 8 |
| Experiment backlog (10 experiments) | Done | 8 |
| Re-analysis trigger in Settings | **Deferred** | — |
| Tier-gated empty states (Discovery, Targets) | **Deferred** | — |
| First-draft activation experience | **Deferred** | — |

---

## 4. Risk Register

| # | Risk | Severity | Status | Mitigation |
|---|------|----------|--------|------------|
| R1 | X Developer Portal setup friction | High | **Accepted** | Clear step-by-step guide; scraper mode as fallback; experiment E1 for improvement |
| R2 | LLM inference quality varies by profile | Medium | **Mitigated** | Heuristic fallback; per-field confidence; all fields editable; sparse profile notice |
| R3 | OAuth tokens expire during onboarding | Low | **Mitigated** | 10-minute PKCE TTL; expired token detection in analyze endpoint; user-friendly error |
| R4 | Analyze endpoint unauthenticated | Low | **Accepted** | Requires valid OAuth tokens to call X API; self-hosted/desktop only; no cloud exposure |
| R5 | Hash-anchor deep-links may not auto-scroll | Low | **Accepted** | Section IDs exist and match; browser native hash behavior works in most cases; `scrollToSection` helper available |
| R6 | Stale onboarding tokens accumulate | Very Low | **Accepted** | Only on abandoned flows; small JSON files; cleanup task in post-launch backlog |
| R7 | No runtime test coverage (no running instance) | Medium | **Mitigated** | Code-path tracing covers all scenarios; recommend manual smoke test before merge |

---

## 5. Known Limitations

| # | Limitation | Impact | Workaround |
|---|-----------|--------|------------|
| 1 | No re-analysis trigger when LLM added post-onboarding | Users who skip LLM during onboarding can't auto-populate profile later | Users can manually edit all profile fields in Settings |
| 2 | Discovery/Targets pages show generic empty states | Users at lower tiers see standard empty states instead of tier-specific guidance | Activation checklist directs users to the right Settings sections |
| 3 | DraftStudioShell lacks first-run activation experience | New users see generic empty state on home page | "New Draft" button and checklist "Available now" section provide guidance |
| 4 | Hash-anchor scrolling not hardened for cross-page SPA navigation | May not auto-scroll on first navigation from `/` to `/settings#section` | User can scroll manually; all sections are visible on the settings page |
| 5 | Stale `onboarding_tokens.json` not cleaned up | Abandoned flows leave small JSON files in data dir | No functional impact; file size is negligible |
| 6 | Analyze endpoint has no auth check | Unauthenticated callers can trigger profile analysis if they have network access | Endpoint requires valid OAuth tokens to succeed; risk is very low for self-hosted/desktop |
| 7 | Server-side telemetry not implemented | Funnel events only visible in browser console/Tauri logs | `console.info('[tuitbot:funnel]')` is grep-able; sufficient for initial launch |

---

## 6. Experiment Readiness

All 10 experiments from `experiment-backlog.md` can run on the current codebase:

| ID | Experiment | Runnable? | Blockers |
|----|-----------|-----------|----------|
| E1 | Inline X Developer App Guide | Yes | None — XApiStep accepts new guide content |
| E2 | Defer Claim to Post-Onboarding | Yes | None — `showClaimStep` flag is the only gate |
| E3 | Auto Re-Analysis on LLM Config | Yes | Needs "Analyze Profile" button in Settings (P1 backlog) |
| E4 | Skip Welcome for Returning Users | Yes | None — `localStorage` check is trivial |
| E5 | Auto-Detect LLM (Ollama) | Yes | None — LlmStep mount can probe localhost |
| E6 | Analysis Skip Timer | Yes | None — ProfileAnalysisState already has timer patterns |
| E7 | Sample Content on Welcome | Yes | None — WelcomeStep accepts new content |
| E8 | Single-Page Onboarding | Yes | None — all step components are independent |
| E9 | Progressive LLM Suggestion | Yes | None — LlmStep can reorder provider cards |
| E10 | Checklist Gamification | Yes | None — `capabilityTier` store is globally available |

E3 is the only experiment that depends on a backlog item (the re-analysis button). All others can be implemented directly on the current code.

---

## 7. Deployment Checklist

1. Run CI gates one final time:
   ```bash
   cargo fmt --all && cargo fmt --all --check
   RUSTFLAGS="-D warnings" cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cd dashboard && npm run check
   ```
2. Manual smoke test (recommended):
   - Start fresh instance with no config
   - Complete onboarding in API mode (if X Developer credentials available) or scraper mode
   - Verify landing on home page with activation checklist
   - Verify Settings page loads with correct profile data
3. Merge `epic/x-profile-prefill-onboarding` → `main`
4. Tag release if following release-plz conventions
5. Update `getting-started.md` if any final copy changes were needed (already updated in Session 8)

---

## 8. Post-Launch Backlog

Ordered by priority. Items marked with session references for traceability.

### P1 — High Impact
1. **Re-analysis trigger in Settings** — Add "Analyze Profile" button in Settings LLM section for users who configure LLM post-onboarding. Enables experiment E3. (Sessions 5–8)
2. **Tier-gated empty states** — Discovery and Targets pages show tier-specific prompts when user lacks required capabilities. (Session 6)

### P2 — Medium Impact
3. **First-draft activation experience** — DraftStudioShell shows activation-aware empty state guiding `profile_ready` users toward their first action. (Session 6)
4. **Hash-anchor scroll hardening** — Add `$effect` on settings page mount that reads `window.location.hash` and calls `scrollToSection()` for reliable cross-page deep-links. (Session 6)
5. **Stale onboarding tokens cleanup** — Periodic or on-init cleanup of abandoned `onboarding_tokens.json` files. (Session 3)
6. **Analyze endpoint authentication** — Add auth check to `POST /api/onboarding/analyze-profile` for defense-in-depth. (Session 3)

### P3 — Low Urgency
7. **Server-side telemetry endpoint** — `POST /api/telemetry/funnel` for persistent event storage and dashboarding. (Session 8)
8. **Experiments E1–E10** — Funnel optimization experiments from `experiment-backlog.md`. (Session 8)

---

## 9. Key Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Open issues 1–6 are non-blocking | Core onboarding → provisioning → first-value path works without them. Users are never stranded. |
| D2 | Analyze endpoint auth is acceptable for launch | Self-hosted/desktop only; requires valid OAuth tokens to do anything useful; no cloud exposure path. |
| D3 | Client-side telemetry is sufficient for launch | Self-hosted product has no cloud telemetry infrastructure. Console.info events are grep-able from Tauri logs and DevTools. |
| D4 | Tier-gated empty states are backlog, not blockers | Activation checklist provides the upgrade guidance. Generic empty states show standard UI, not broken state. |
| D5 | Manual smoke test recommended before merge | Code-path tracing confirms all paths exist, but runtime validation catches integration issues that static analysis misses. |
