# CLAUDE.md

## Commands

```bash
# Build & Run
cargo build                              # debug
cargo build --release                    # release → target/release/tuitbot
cargo run -p tuitbot-server              # API server (127.0.0.1:3001)
cargo run -p tuitbot-server -- --host 0.0.0.0  # LAN mode (all interfaces)

# Test
cargo test                               # all tests
cargo test -p tuitbot-core scoring       # specific module
cargo test -p tuitbot-core -- --test-threads=1  # serial (env-var tests)

# Lint & Format
cargo fmt --all                          # auto-format
cargo clippy --workspace -- -D warnings  # lint

# Frontend
cd dashboard && npm run dev              # dev server (localhost:5173)
cd dashboard && npm run build            # production build
cd dashboard && npm run check            # type-check (svelte-check)

# Desktop
cd dashboard && npm run tauri dev        # dev mode
cd dashboard && npm run tauri build      # production → DMG/MSI/AppImage
```

## Mandatory CI Checklist

Run before handing off **any** Rust change — fix all failures:

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

- Always run `cargo fmt --all` — never assume hand-formatted code is correct.
- All warnings are release blockers. Do not finish until this passes.

## Cross-Platform Testing
CI runs on **macOS, Linux, and Windows**. All tests must pass on all three:
- **Never hardcode Unix paths** like `/tmp` in tests. Use `std::env::temp_dir()` for a directory that exists, or `tempfile::tempdir()` for a throwaway dir.
- **Never assume `/` path separators.** Use `std::path::PathBuf` or `Path::join()`.
- **Never assume Unix line endings.** Use `.trim()` or `.lines()` when comparing text output.

## Constraints

### File Size Limits
- **Rust:** Max 500 lines per `.rs` file. Exceed → convert to module directory (`foo.rs` → `foo/mod.rs` + submodules). Tests in `tests.rs` submodule when >100 lines.
- **Svelte:** Max 400 lines per `+page.svelte`. Extract into sibling `*Section.svelte` files.

### Dependency Rule
`tuitbot-core` has three layers: Toolkit (`core/toolkit/`) ← Workflow (`core/workflow/`) ← Autopilot (`core/automation/`). No upward or skip-level imports.

### Error Handling
- `thiserror` in `tuitbot-core` (typed enums per domain).
- `anyhow` in binary crates (`tuitbot-cli`, `tuitbot-server`).

### Server Boundary
`tuitbot-server` owns zero business logic — only routing, serialization, WebSocket fan-out, auth. All domain logic lives in `tuitbot-core`.

### Frontend
Svelte 5 (runes: `$props()`, `$state()`, `$derived()`, `$effect()`), TypeScript strict, TailwindCSS, SPA mode (no SSR).

### Crates.io Release Discipline
When changing crates under `crates/`:
- Keep `Cargo.lock` committed. Every local `path` dep must include explicit `version`.
- Maintain package metadata: `description`, `license`, `repository`, `homepage`, `documentation`, `keywords`.
- Do not change `release-plz.toml` tag conventions without approval.

Validate before handoff:
```bash
release-plz update --config release-plz.toml --allow-dirty
cargo package --workspace --allow-dirty
```

## Coverage Thresholds

**Rust Coverage (Cargo Tarpaulin):**

*Core Crates (Task 4.1):*
- Minimum: **75% lines** (enforced by `cargo tarpaulin --fail-under 75`)
- Scope: All workspace crates except `tuitbot-mcp` and `tuitbot-cli`
- CI fails if coverage drops below 75%
- Measurement: `cargo tarpaulin --workspace --exclude tuitbot-mcp --exclude tuitbot-cli --out Xml --fail-under 75`

*tuitbot-mcp Tools (Task 4.3):*
- Minimum: **60% lines** (enforced by `cargo tarpaulin -p tuitbot-mcp --fail-under 60`)
- Scope: MCP tool handlers (manifest, admin, write, workflow, etc.)
- CI fails if coverage drops below 60%
- Separate gate due to tool complexity (many small handlers, request validation, etc.)
- Measurement: `cargo tarpaulin -p tuitbot-mcp --out Xml --fail-under 60`

**Frontend Coverage (Vitest):**
- Global minimum: **70% lines** (enforced in `dashboard/vitest.config.ts`)
- Per-file minimums for core stores: **75% lines**
  - `src/lib/stores/approval.ts`
  - `src/lib/stores/analytics.ts`
  - `src/lib/stores/settings.ts`
  - `src/lib/stores/targets.ts`
- CI fails if either global or per-file threshold drops
- Measurement: `npm run test:coverage:ci` in dashboard/

**CI Enforcement:**
- Coverage workflow (`.github/workflows/coverage.yml`) runs on every push + PR
- Both Rust and Frontend gates must pass for merge
- Codecov reports uploaded to tracking history

## Always Do First
- **Invoke the `frontend-design` skill** before writing any frontend code, every session, no exceptions.
- **Run tests before committing frontend changes**: `cd dashboard && npx vitest run` to catch test failures before they hit CI. If a component's rendering behavior changes, update its corresponding test file in `dashboard/tests/unit/`.
