# Codecov Setup

One-time setup to enable coverage uploads from CI to codecov.io.

## 1. Create a Codecov account

Go to <https://app.codecov.io> and sign in with your GitHub account.
Add the `aramirez087/TuitBot` repository.

## 2. Get the upload token

In Codecov → your repo → **Settings → General → Repository Upload Token**.
Copy the token (looks like a UUID).

## 3. Add the token as a GitHub secret

1. Go to **GitHub → TuitBot repo → Settings → Secrets and variables → Actions**
2. Click **New repository secret**
3. Name: `CODECOV_TOKEN`
4. Value: paste the Codecov upload token
5. Click **Add secret**

## 4. That's it

The `coverage.yml` workflow reads `${{ secrets.CODECOV_TOKEN }}` automatically.
On the next push/PR, coverage reports will be uploaded and the README badges
will update.

## Thresholds

| Layer    | Enforcement              | Target |
|----------|--------------------------|--------|
| Rust     | `--fail-under 75` in CI  | ≥75%   |
| Frontend | Vitest per-file thresholds | ≥70–85% (per store) |
| PR patch | codecov.io status check  | ≥60%   |

Thresholds are defined in:
- `.codecov.yml` — Codecov project/patch targets
- `.github/workflows/coverage.yml` — Rust tarpaulin `--fail-under`
- `dashboard/vitest.config.ts` — per-file Vitest thresholds

## Downloading coverage reports

Every CI run uploads two artifacts:
- **`rust-coverage-report`** — `cobertura.xml` (30-day retention)
- **`frontend-coverage-report`** — `lcov.info` + `coverage-final.json` (30-day retention)

Download from: **GitHub → Actions → (workflow run) → Artifacts**
