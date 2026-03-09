# Troubleshooting

## release-plz says no releases were created

Expected when the commit is not a merged release PR and no publishable delta exists.

## Build/publish asset jobs are skipped

These jobs run only when the workflow can resolve an asset-bearing release target:

- a `tuitbot-cli` release from `release-plz`
- a `tuitbot-server` release from `release-plz`
- or an explicit `release_tag` input on a manual run
- or a tagged release commit recoverable from the current merge commit during a rerun/backfill

## crates.io publish fails with email verification error

Verify email on crates.io profile for the token owner.

## Cargo lock issues in CI

Ensure `Cargo.lock` is tracked in git and up-to-date.

## X auth callback issues

Callback URI must exactly match:

`http://127.0.0.1:8080/callback`

## Debug commands

```bash
tuitbot test --output json
tuitbot settings --show --output json
tuitbot tick --dry-run --output json
```
