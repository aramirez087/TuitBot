# Session 01 Handoff — Passphrase Lifecycle UX

**Date:** 2026-03-01
**Status:** Complete
**Branch:** fix/uninstall (committed on existing branch)

---

## Completed

- [x] Root-cause analysis for all three reported failures with file:line references
- [x] Comparison with `fresh-install-auth-ux` charter — confirmed no regressions, only pre-existing gaps
- [x] Implementation charter with design decisions, session slices, acceptance scenarios, and security review
- [x] This handoff document

## Decisions Made

1. **Claimed-but-unconfigured path** gets recovery guidance via `ClaimStep` in `alreadyClaimed` mode — not a new claim flow. The existing passphrase cannot be shown (only the hash exists), and re-claiming would weaken security.

2. **`--reset-passphrase` becomes an early-exit fast path** in `main.rs` — after CLI parse and `db_dir` derivation, before DB init, API token, config, LLM, Watchtower, and port binding. Output is the bare passphrase on stdout, exit code 0.

3. **Login handler uses mtime-based disk refresh** — a `stat()` syscall on each login attempt (rate-limited to 5/min/IP). No filesystem watcher, no signal handling, no periodic timer.

## Key Root Causes

| Failure | Root Cause | File |
|---------|-----------|------|
| No passphrase after onboarding | `showClaimStep = !isTauri && !alreadyClaimed` excludes Secure step when claimed | `dashboard/src/routes/onboarding/+page.svelte:32` |
| Noisy `--reset-passphrase` | Reset handled at line 78, but startup continues through line 245 (port bind) | `crates/tuitbot-server/src/main.rs:78-245` |
| Stale in-memory auth | Login reads `state.passphrase_hash` (loaded once at startup), never re-reads disk | `crates/tuitbot-server/src/auth/routes.rs:117` |

## Open Issues

None blocking Session 02.

## Inputs for Session 02

**Read first:**
- `docs/roadmap/passphrase-lifecycle-ux/charter.md` — full context, design decisions, acceptance criteria

**Scope:** Frontend changes only (onboarding claimed path, ClaimStep variant). Do NOT modify backend code.

**Files to modify:**

| File | Change |
|------|--------|
| `dashboard/src/routes/onboarding/+page.svelte` | Always include 'Secure' step in web mode (change `showClaimStep` to `!isTauri`); pass `alreadyClaimed` to ClaimStep |
| `dashboard/src/lib/components/onboarding/ClaimStep.svelte` | Add `alreadyClaimed` prop; when true, show informational recovery guidance instead of passphrase generation UI |

**Quality gates:**
```bash
cd dashboard && npm run check
cd dashboard && npm run build
```

**Key constraints:**
- Do not touch the passphrase generation logic in ClaimStep when `alreadyClaimed` is false — that flow works correctly
- Do not modify backend auth routes or server startup
- Use Svelte 5 runes (`$props()`, `$state()`, `$derived()`) — no legacy syntax
- Use tabs for indentation in `.svelte` files
- Invoke the `frontend-design` skill before writing frontend code (per CLAUDE.md)

## Inputs for Session 03

**Read first:**
- `docs/roadmap/passphrase-lifecycle-ux/charter.md` — Design Decisions B and C

**Scope:** Backend changes only (reset command fast path, mtime-based login refresh).

**Files to modify:**

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/main.rs` | Early exit for `--reset-passphrase` after line 62 |
| `crates/tuitbot-server/src/state.rs` | Add `passphrase_hash_mtime: RwLock<Option<SystemTime>>` |
| `crates/tuitbot-server/src/auth/routes.rs` | Mtime check before passphrase verify in login |
| `crates/tuitbot-core/src/auth/passphrase.rs` | Add `passphrase_hash_mtime()` helper |
| `docs/lan-mode.md` | Update reset docs |

**Quality gates:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```
