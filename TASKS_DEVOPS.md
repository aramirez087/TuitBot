# DevOps/CI Board — TuitBot Pipeline Optimization & Automation

**Board Lead Inbox:** Release pipeline efficiency, dependency automation, and tooling  
**Audit Date:** 2026-03-13  
**Total Effort:** ~2–3 weeks  
**Success Metric:** 20+ min release cycle reduction; automated dependency updates; coverage reporting

---

## Task 4.1: Unify Dashboard Build Steps

**Scope:** `.github/workflows/release.yml`  
**Current Problem:** Dashboard builds 3+ separate times in release workflow  
**Status:** NOT STARTED  
**Assigned to:** [DevOps Lead]  
**Sprint:** 1 (HIGH ROI, quick win)

### Current Workflow Analysis
- **Lines 46–50:** Dashboard build in `frontend` job
- **Lines 103–107:** Dashboard build in crate publish step
- **Lines 168–172:** Dashboard build in binary build step
- **Result:** Same build output generated 3× per release (~5–10 min wasted)

### Solution: Artifact Caching

**Plan A: Cache SvelteKit Output (Preferred)**
- [ ] Build dashboard once in standalone job (`dashboard-build`)
- [ ] Cache output: `dashboard/.svelte-kit` and `dashboard/build/`
- [ ] Subsequent jobs restore cache (no rebuild)
- [ ] Expected savings: 15–20 min per release

**Implementation:**
```yaml
# .github/workflows/release.yml
jobs:
  dashboard-build:
    name: Build Dashboard
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - uses: actions/cache@v4
        with:
          path: dashboard/node_modules
          key: dashboard-${{ hashFiles('dashboard/package-lock.json') }}
      - run: cd dashboard && npm ci && npm run build
      - uses: actions/upload-artifact@v4
        with:
          name: dashboard-build
          path: |
            dashboard/build/
            dashboard/.svelte-kit/

  publish-crates:
    needs: dashboard-build
    # ... rest of publish job
    steps:
      # ... checkout
      - uses: actions/download-artifact@v4
        with:
          name: dashboard-build
          path: dashboard/
      # ... no need to rebuild, artifact is ready
```

### Definition of Done
- [ ] Dashboard builds only once per release
- [ ] Artifacts cached and reused across jobs
- [ ] Release workflow execution time reduced by ≥20%
- [ ] No manual steps added to release process

### Effort Estimate
- 2–3 days (workflow rewrite, validation)

### Risk Mitigation
- Test on dev/staging first (branch + PR)
- Keep old workflow commented out as fallback
- Monitor first 2–3 releases for issues

---

## Task 4.2: Integrate Code Coverage Reporting

**Status:** NOT STARTED  
**Assigned to:** [DevOps Lead]  
**Sprint:** 2

### Backend Coverage (Rust)

**Requirements:**
- [ ] Install `cargo-tarpaulin` in CI
- [ ] Add coverage job to `ci.yml`:
  ```bash
  cargo tarpaulin --workspace --out Xml --output-dir coverage
  ```
- [ ] Upload to codecov.io (optional) or generate badge locally
- [ ] Set minimum coverage threshold (e.g., 75% for core modules)
- [ ] Fail CI if coverage drops below threshold

**Implementation:**
```yaml
# .github/workflows/ci.yml
coverage:
  name: Code Coverage (Rust)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions-rust-lang/setup-rust-toolchain@v1
    - run: cargo install cargo-tarpaulin
    - run: cargo tarpaulin --workspace --out Xml
    - uses: codecov/codecov-action@v4
      with:
        files: ./cobertura.xml
        flags: rust
```

### Frontend Coverage (JavaScript/TypeScript)

**Requirements:**
- [ ] Vitest configured with coverage in `dashboard/`
- [ ] Add coverage job to `ci.yml`:
  ```bash
  cd dashboard && npm ci && npm run test:coverage
  ```
- [ ] Upload to codecov.io or generate badge
- [ ] Set threshold (e.g., 70% for dashboard)

**Implementation:**
```yaml
# .github/workflows/ci.yml
coverage-frontend:
  name: Code Coverage (Frontend)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
    - run: cd dashboard && npm ci && npm run test:coverage
    - uses: codecov/codecov-action@v4
      with:
        files: ./dashboard/coverage/cobertura-coverage.xml
        flags: frontend
```

### Badge in README
- [ ] Add coverage badge to README.md:
  ```markdown
  [![Coverage](https://codecov.io/gh/aramirez087/TuitBot/branch/main/graph/badge.svg)](https://codecov.io/gh/aramirez087/TuitBot)
  ```

### Definition of Done
- [ ] Coverage reports generated in every CI run
- [ ] Badge visible in README and on codecov.io
- [ ] CI fails if coverage drops below threshold
- [ ] Developers can download coverage reports from CI artifacts

### Effort Estimate
- 2–3 days (setup + validation)

---

## Task 4.3: Performance Baseline Tracking

**Status:** NOT STARTED  
**Assigned to:** [DevOps Lead]  
**Sprint:** 2–3

### Backend Performance Benchmarks

**Requirements:**
- [ ] Set up `cargo-criterion` for benchmark suite
- [ ] Create benchmarks for critical paths:
  - Content generation (LLM call latency)
  - Tweet scoring (discovery engine)
  - Approval queue filtering
  - Database queries (analytics aggregation)

**Implementation:**
```rust
// benches/generation_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_content_generation(c: &mut Criterion) {
  c.bench_function("generate_tweet_200_chars", |b| {
    b.iter(|| {
      generate_tweet(black_box(&test_account), black_box(&test_context))
    })
  });
}

criterion_group!(benches, bench_content_generation);
criterion_main!(benches);
```

- [ ] Add to CI: `cargo criterion --bench '*' --output-format bencher`
- [ ] Track trends over time (commit-to-commit)
- [ ] Alert if performance regresses by >10%

### Frontend Performance Metrics

**Requirements:**
- [ ] Measure SvelteKit build time
- [ ] Track dashboard bundle size (JS, CSS)
- [ ] Monitor component render time (Lighthouse)
- [ ] Alert if bundle size grows >5% per release

**Implementation in CI:**
```bash
# Measure build time
time npm run build

# Check bundle size
npm run build && du -sh dashboard/build/
```

### Definition of Done
- [ ] Benchmarks run in CI and produce reports
- [ ] Trends tracked over time
- [ ] Regressions detected early (before release)
- [ ] Developers can run benchmarks locally

### Effort Estimate
- 3–5 days (setup + baseline establishment)

---

## Task 4.4: Automated Dependency Updates

**Status:** NOT STARTED  
**Assigned to:** [DevOps Lead]  
**Sprint:** 2

### Dependabot Integration

**Requirements:**
- [ ] Enable Dependabot in GitHub (Settings → Code security → Dependabot alerts)
- [ ] Create `.github/dependabot.yml`:
  ```yaml
  version: 2
  updates:
    # Rust dependencies
    - package-ecosystem: "cargo"
      directory: "/"
      schedule:
        interval: "weekly"
        day: "monday"
      allow:
        - dependency-type: "direct"
      reviewers:
          - "aramirez087"
      labels:
          - "dependencies"

    # Node dependencies
    - package-ecosystem: "npm"
      directory: "/dashboard"
      schedule:
        interval: "weekly"
        day: "monday"
      allow:
        - dependency-type: "direct"
      reviewers:
          - "aramirez087"
      labels:
          - "dependencies"
  ```

- [ ] Dependabot automatically creates PRs for updates
- [ ] Each PR runs full CI (tests, linting, security audit)
- [ ] Team reviews and merges (or auto-merge if tests pass)

### Configuration

- [ ] Set `auto-merge` for:
  - Patch versions (1.0.0 → 1.0.1) if CI passes
  - Dependencies with no known breaking changes
  
- [ ] Require manual review for:
  - Major version updates (1.0.0 → 2.0.0)
  - Dependencies with breaking changes

### Definition of Done
- [ ] Dependabot PRs flowing in (typically 1–3 per week)
- [ ] CI runs on each PR
- [ ] Process documented (how to review/merge)

### Effort Estimate
- 1–2 days (setup + process definition)

---

## Task 4.5: Security & Compliance Automation

**Status:** NOT STARTED  
**Assigned to:** [DevOps Lead]  
**Sprint:** 2

### Enhancements to Existing Audit

**Current State:**
- `cargo audit` runs in CI, fails on vulnerabilities ✓
- No automated dependency updates yet

**Enhancements:**
- [ ] `cargo audit --deny warnings` — block on advisory warnings (not just errors)
- [ ] `npm audit` in dashboard (add to CI)
- [ ] License compliance check (`cargo-license` for Rust, `npm-license-checker` for Node)
- [ ] SBOM generation (software bill of materials) for releases

**Implementation:**
```yaml
# .github/workflows/ci.yml
security-audit:
  name: Security & Compliance Audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    # Rust audit
    - uses: actions-rust-lang/setup-rust-toolchain@v1
    - run: cargo install cargo-audit
    - run: cargo audit --deny warnings
    
    # Node audit
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
    - run: cd dashboard && npm audit --audit-level=high
    
    # License compliance
    - run: cargo install cargo-license
    - run: cargo license --all-features > /tmp/rust-licenses.txt
    - run: cd dashboard && npx npm-license-checker --csv > /tmp/node-licenses.txt
```

### Definition of Done
- [ ] All audits pass in CI
- [ ] Vulnerabilities blocked before merge
- [ ] License report included in releases

### Effort Estimate
- 2–3 days

---

## Task 4.6: GPG Signing for Release Artifacts

**Status:** NOT STARTED (LOW priority, nice-to-have)  
**Assigned to:** [DevOps Lead]  
**Sprint:** 3

### Requirements
- [ ] Generate GPG key (or use existing)
- [ ] Store private key in GitHub Secrets (`GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`)
- [ ] Sign release artifacts in `release.yml`:
  ```bash
  gpg --import $GPG_PRIVATE_KEY
  gpg --detach-sign --armor target/release/tuitbot
  ```
- [ ] Upload `.asc` signature files alongside binaries
- [ ] Document signature verification in README:
  ```bash
  curl -O https://github.com/.../releases/download/tuitbot-cli-v0.1.22/tuitbot-cli-v0.1.22.asc
  gpg --verify tuitbot-cli-v0.1.22.asc tuitbot-cli
  ```

### Definition of Done
- [ ] Releases include GPG signatures
- [ ] Verification instructions in README
- [ ] Users can verify binary authenticity

### Effort Estimate
- 2–3 days (setup + documentation)

---

## Task 4.7: Desktop App Build Consolidation

**Status:** NOT STARTED (LOW priority, consolidation)  
**Assigned to:** [DevOps Lead]  
**Sprint:** 3

### Current State
- Desktop build (`build-desktop.yml`) is separate from main release

### Goal
- [ ] Consolidate desktop build into `release.yml`
- [ ] Same binaries, same versioning, same release notes
- [ ] Easier to manage (single workflow)

### Implementation
- Merge `build-desktop.yml` into `release.yml` as conditional job:
  ```yaml
  # .github/workflows/release.yml
  build-desktop:
    if: startsWith(github.ref, 'refs/tags/tuitbot-cli-v')
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - run: cd dashboard && npm ci && npm run tauri build
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/dmg/*.dmg
            src-tauri/target/release/bundle/msi/*.msi
  ```

### Definition of Done
- [ ] Desktop app builds in main release workflow
- [ ] Same release notes for all artifacts
- [ ] CI time unchanged (desktop builds in parallel)

### Effort Estimate
- 2–3 days

---

## Summary & Success Criteria

| Task | Est. Days | Priority | Sprint | Owner | ROI |
|------|-----------|----------|--------|-------|-----|
| 4.1 Dashboard Cache | 2–3 | HIGH | 1 | DevOps | ⭐⭐⭐⭐⭐ 20 min/release |
| 4.2 Coverage Reports | 2–3 | MEDIUM | 2 | DevOps | ⭐⭐⭐ Observability |
| 4.3 Performance Bench | 3–5 | MEDIUM | 2–3 | DevOps | ⭐⭐⭐ Regression detection |
| 4.4 Dependabot | 1–2 | MEDIUM | 2 | DevOps | ⭐⭐⭐ Automation |
| 4.5 Security Audit | 2–3 | MEDIUM | 2 | DevOps | ⭐⭐ Compliance |
| 4.6 GPG Signing | 2–3 | LOW | 3 | DevOps | ⭐ Supply chain |
| 4.7 Desktop Consolidation | 2–3 | LOW | 3 | DevOps | ⭐ Maintainability |

**Total Estimated Effort:** ~2–3 weeks  
**Expected Outcomes:**
- Release cycle reduced by ≥20 minutes
- Automated dependency updates flowing in
- Coverage reporting in CI (observability)
- Performance regressions detected early
- Supply chain security (GPG signing)
- Simplified release workflow (desktop consolidation)

---

## Release Workflow Timeline (Post-Optimization)

**Before (Current):**
1. Create release PR (manual, ~2 min)
2. Wait for CI (test, build, lint) — ~10–15 min
3. Merge release PR
4. Wait for release job:
   - Build Rust binaries (3 platforms) — ~20 min
   - **Build dashboard 3 times** — ~15 min (wasted)
   - Publish crates — ~3 min
   - Create GitHub release — ~1 min
5. **Total: ~35–40 min**

**After (Optimized):**
1. Create release PR (manual, ~2 min)
2. Wait for CI (test, build, lint) — ~10–15 min
3. Merge release PR
4. Wait for release job:
   - **Build dashboard once** (cached) — ~3 min
   - Build Rust binaries (3 platforms, in parallel) — ~20 min
   - Publish crates — ~3 min
   - Create GitHub release — ~1 min
5. **Total: ~20–25 min** (savings: ~15–20 min)

---

## Deployment Process Documentation

Create `DEPLOYMENT.md` with:
- Release checklist
- Hotfix process
- Rollback strategy
- Monitoring post-release (error rates, CI health)
- Troubleshooting common issues
- Desktop app signing process
- Dependency update approval workflow

