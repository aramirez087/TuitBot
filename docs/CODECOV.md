# Codecov Integration for TuitBot

This document describes how to set up and verify Codecov integration for TuitBot's code coverage reporting.

## Overview

TuitBot uses Codecov to aggregate and track code coverage metrics from:
- **Rust** — cargo-tarpaulin (cobertura XML format)
- **Frontend** — Vitest (lcov format)

Workflows are defined in `.github/workflows/coverage.yml`.

## Prerequisites

- Codecov account at https://codecov.io
- Repository linked to Codecov
- GitHub repository with write access to secrets

## Setup Steps

### 1. Create Codecov Account & Link Repository

1. Go to https://codecov.io and sign in with GitHub
2. Authorize Codecov to access your repositories
3. Navigate to the TuitBot repository
4. Copy the repository token (if not auto-detected)

### 2. Add CODECOV_TOKEN Secret to GitHub

1. Go to your GitHub repository → **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Name: `CODECOV_TOKEN`
4. Value: Paste the token from Codecov
5. Click **Add secret**

The token is optional — Codecov can also detect public repositories automatically. However, adding the token ensures reliable uploads for pull requests and branch pushes.

### 3. Verify Workflow Configuration

Check `.github/workflows/coverage.yml`:

```yaml
- name: Upload Rust coverage to Codecov
  uses: codecov/codecov-action@v5
  with:
    files: coverage/cobertura.xml
    flags: rust
    fail_ci_if_error: false   # Non-blocking — CI passes even if upload fails
  env:
    CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}

- name: Upload frontend coverage to Codecov
  uses: codecov/codecov-action@v5
  with:
    files: dashboard/coverage/lcov.info
    flags: frontend
    fail_ci_if_error: false
  env:
    CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}
```

Both uploads are **non-blocking** (`fail_ci_if_error: false`) — CI won't fail if Codecov is unreachable or the token is missing.

### 4. Test the Integration

**Option A: Trigger via PR**
1. Create a feature branch
2. Make a small change (e.g., add a comment)
3. Open a Pull Request
4. Wait for coverage workflow to complete
5. Check PR for Codecov status check and coverage badge

**Option B: Manual Workflow Dispatch**
1. Go to **Actions** → **Coverage** workflow
2. Click **Run workflow** on the `main` branch
3. Monitor the job output for upload success

Expected output in logs:
```
✓ GitHub Actions to Codecov: SUCCESS
📋 Commit Status: ✓ success
```

### 5. Review Coverage Reports

Once uploaded, visit https://codecov.io/gh/aramirez087/TuitBot to:
- View overall coverage trends
- Compare coverage across commits and branches
- Review per-file coverage reports
- Set coverage targets and PR comments

## Configuration

### .codecov.yml

File: `.codecov.yml` defines coverage thresholds and PR comment behavior:

```yaml
coverage:
  status:
    project:
      default:
        # Don't fail PRs for <1% coverage drop
        threshold: 1%
    patch:
      default:
        # Require new code in PRs to have ≥60% coverage
        target: 60%

flags:
  rust:
    paths: crates/
    carryforward: true
  frontend:
    paths: dashboard/src/
    carryforward: true
```

- **project threshold**: Allows up to 1% drop in overall coverage
- **patch target**: New code must have ≥60% line coverage
- **flags**: Separate tracking for Rust and frontend with carryforward (preserves coverage from prior commits)

## Troubleshooting

### "No files found with the provided path"

Check that the correct directories exist after build:
- `coverage/cobertura.xml` — output from `cargo tarpaulin`
- `dashboard/coverage/lcov.info` — output from `npm run test:coverage`

Run locally:
```bash
# Rust coverage
cd /tmp/TuitBot && cargo tarpaulin --workspace --out xml --output-dir coverage/
ls coverage/cobertura.xml

# Frontend coverage
cd /tmp/TuitBot/dashboard && npm run test:coverage
ls coverage/lcov.info
```

### "HTTP 401 Unauthorized"

The CODECOV_TOKEN secret is missing or invalid:
1. Verify the secret exists in GitHub repo settings
2. Check token value in Codecov dashboard
3. Ensure secret name is **exactly** `CODECOV_TOKEN`

### "Coverage appears to be 0%"

Check workflow logs for errors in tarpaulin or vitest commands. Verify:
- `cargo tarpaulin` installed and in PATH
- Rust tests pass (no `--deny unsound` or other failures)
- Vitest runs successfully locally: `npm run test:coverage`

### PR shows "codecov/project" as failing but CI is green

This is expected — PR check can fail if coverage drops >1% threshold, but CI (`fail_ci_if_error: false`) won't block the merge. Review the coverage comment in the PR and make a judgment call.

## References

- Codecov documentation: https://docs.codecov.com
- codecov/codecov-action: https://github.com/codecov/codecov-action
- cargo-tarpaulin docs: https://github.com/xd009642/tarpaulin
- Vitest coverage: https://vitest.dev/guide/coverage
