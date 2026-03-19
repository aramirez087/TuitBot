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

## Branch Protection

**Status:** Enabled on `main` branch (as of 2026-03-18).

**Rule:** Branch protection enforces strict status checks via `required_status_checks.strict=true`. This means:
- All PRs must have passing CI checks before merge
- **Merged branches must be up-to-date with `main`** — prevents stale merges that cause silent conflicts
- Avoids the refactor/sprint-1 scenario where outdated PRs merged without rebasing

**To apply rule programmatically:**
```bash
cat > /tmp/protection.json << 'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": []
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF
gh api repos/aramirez087/TuitBot/branches/main/protection -X PUT --input /tmp/protection.json
```

**To verify rule is active:**
```bash
gh api repos/aramirez087/TuitBot/branches/main/protection | jq '.required_status_checks.strict'
# Output: true
```

**Test:** Attempt to merge a PR from a stale branch (not rebased onto latest main). GitHub will block it with message: *"Base branch is not up to date with target branch."*

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

**Note:** These commands are now automatically enforced in CI via the `release-verify` job (runs on every Rust change). The manual validation above is still required locally before opening a PR, but CI will catch any issues before merge.

**CI Validation Details:**
- `release-plz update --dry-run`: Detects invalid version bumps and dependency issues
- `cargo package --workspace`: Verifies all crates have complete metadata (description, license, keywords, etc.)
- Both gates must pass on every Rust change PR
- Fails immediately on missing/invalid metadata before reaching release.yml

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

## Automated Release Cadence

**Workflow:** `.github/workflows/release-plz-weekly.yml`

**Purpose:** Enforce a predictable weekly release schedule by automatically opening a `release-plz` PR every Monday at 09:00 UTC, regardless of whether changes exist.

**How it works:**
1. Cron trigger: `0 9 * * 1` (Mondays 09:00 UTC)
2. Runs: `release-plz release-pr` — reads `release-plz.toml`, bumps versions if changes exist
3. **Key behavior:**
   - If changes warrant a release, opens/updates a PR (title: `chore(release)`)
   - If no changes exist, exits cleanly (no empty PR created) — prevents spam
4. Workflow dispatch: Can be manually triggered via GitHub Actions UI for testing
5. Permissions: `contents: write` (bump versions), `pull-requests: write` (create PR)

**Why this pattern:**
- Predictable cadence (no surprise releases on random days)
- Changes are batched (weekly cutoff) — good for testing and announcements
- Automated versioning via `release-plz` (SemVer, changelog, tag, publish)
- No manual intervention needed — PR is ready-to-review-and-merge

**Related files:**
- `release-plz.toml` — defines versioning rules and changelog format
- `.github/workflows/release.yml` — runs on PR merge → builds, tests, publishes

## SBOM (Software Bill of Materials) Generation

**Workflow:** `.github/workflows/release.yml` (publish-assets job)

**Purpose:** Generate a machine-readable inventory of all Rust dependencies for supply chain compliance and vulnerability tracking.

**Tool & Format:**
- **Tool:** `cargo-cyclonedx` (preferred CycloneDX format)
- **Output:** `sbom.xml` (CycloneDX XML format per NTIA standards)
- **Rationale:** cargo-cyclonedx integrates directly with Cargo.lock, capturing the exact dependency tree at build time. CycloneDX XML is widely supported by compliance tools and vulnerability scanners (e.g., Dependabot, SPDX tools, SBOM databases).

**Implementation Details:**
1. **Step 1 (release job):** Install cargo-cyclonedx and generate sbom.xml from Cargo.lock
   - Runs after Rust toolchain setup
   - Non-blocking on error (`continue-on-error: true`) — tool failure does not block release
   - Output: `sbom-dist/sbom.xml`
2. **Step 2 (publish-assets job):** Download SBOM artifact and stage for GitHub Release upload
   - Renames to `tuitbot-{RELEASE_TAG}-sbom.xml` for clarity
   - Uploaded alongside binaries and checksums

**Error Handling:**
- If SBOM generation fails (tool not installed, Cargo.lock parsing issues, etc.), a warning is logged but the release proceeds
- This is intentional: release cadence should not be blocked by optional security metadata
- Failed SBOM generation is still visible in workflow logs for manual investigation

**Verification:**
- Generated SBOM can be validated with: `cyclonedx-cli validate --input-file tuitbot-{TAG}-sbom.xml`
- Tools like Syft, SPDX converters, and vulnerability scanners consume the SBOM directly from GitHub Releases

## Always Do First
- **Invoke the `frontend-design` skill** before writing any frontend code, every session, no exceptions.
- **Run tests before committing frontend changes**: `cd dashboard && npx vitest run` to catch test failures before they hit CI. If a component's rendering behavior changes, update its corresponding test file in `dashboard/tests/unit/`.
