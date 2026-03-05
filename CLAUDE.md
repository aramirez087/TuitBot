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

## Always Do First
- **Invoke the `frontend-design` skill** before writing any frontend code, every session, no exceptions.
