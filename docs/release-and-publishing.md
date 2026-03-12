# Release and Publishing

## CI release model

Workflow: `.github/workflows/release.yml`

1. Every push to `main` updates the release PR via `release-plz release-pr`.
   Dashboard-only changes under `dashboard/**` are fingerprinted into
   `crates/tuitbot-server/dashboard-release-marker.txt` during this job so
   they also trigger a `tuitbot-server` release PR.
2. A manual run of `.github/workflows/release.yml` on `main` publishes crates and creates release tags.
3. Manual runs also accept an optional `release_tag` input so you can backfill assets for an existing `tuitbot-cli-v...` or `tuitbot-server-v...` tag.
4. If no `release_tag` input is provided, a rerun on the release PR merge commit can still recover the tagged release from the merge parents and backfill missing assets.

## Tags

- `tuitbot-core-vX.Y.Z`
- `tuitbot-mcp-vX.Y.Z`
- `tuitbot-cli-vX.Y.Z`

## GitHub releases and binary assets

Releases that include `tuitbot-cli` or `tuitbot-server` publish GitHub release assets:

| Binary | Platforms |
|--------|-----------|
| `tuitbot` (CLI) | linux x86_64, macOS Intel, macOS Apple Silicon, windows x86_64 |
| `tuitbot-server` | linux x86_64, macOS Intel, macOS Apple Silicon, windows x86_64 |

When both packages are released together, assets are attached to the CLI release tag. When only `tuitbot-server` is released, the workflow creates or reuses the server release tag for asset uploads.

Each asset-publishing release also includes a `SHA256SUMS` checksum manifest covering all archives.

`tuitbot update` uses the CLI-tagged assets to self-update the CLI and, if `tuitbot-server` is found on `PATH`, updates it from the best release that ships a server binary for the current platform.

## Required repository secrets

- `CARGO_REGISTRY_TOKEN`: crates.io API token for publish.
- `RELEASE_PLZ_TOKEN` (optional): PAT for broader automation trigger behavior.

## First-time crates.io requirements

The crates.io account bound to `CARGO_REGISTRY_TOKEN` must have a verified email.

Profile settings:

- <https://crates.io/settings/profile>

## Manual verification

```bash
release-plz update --config release-plz.toml --allow-dirty
cargo check --workspace
```
