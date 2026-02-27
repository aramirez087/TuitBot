# Tuitbot Documentation

Tuitbot is an autonomous X growth assistant that operates across three distinct architectures:

- **Desktop App**: A beautiful Tauri+Svelte visual dashboard for editing and approving content visually.
- **Docker Self-Hosted**: 24/7 web-based dashboard driven natively through a persistent remote container.
- **headless CLI**: The foundational CLI, suitable for cron execution (`tuitbot tick`) or background daemons (`tuitbot run`).

This documentation is the source of truth for setup, operations, release, and publishing.

## Documentation Map

- [Getting Started](getting-started.md): install, initialize, authenticate, and run.
- [LAN Mode](lan-mode.md): access the dashboard from any device on your network.
- [Configuration](configuration.md): configuration model and production guidance.
- [Composer Mode](composer-mode.md): AI-assisted writing, drafts, and discovery feed.
- [CLI Reference](cli-reference.md): command-by-command usage.
- [MCP Reference](mcp-reference.md): AI-agent integration and tools.
- [Architecture](architecture.md): crate boundaries and runtime design.
- [Operations](operations.md): deployment and day-2 runbook.
- [Release and Publishing](release-and-publishing.md): release-plz flow and crates.io.
- [Troubleshooting](troubleshooting.md): known failure modes and fixes.
- [Contributing](contributing.md): development workflow and PR expectations.

## Versioning Policy

- GitHub release tags for CLI binaries: `tuitbot-cli-vX.Y.Z`
- Crate tags for registry releases:
  - `tuitbot-core-vX.Y.Z`
  - `tuitbot-mcp-vX.Y.Z`
  - `tuitbot-cli-vX.Y.Z`

## Support

- GitHub issues: <https://github.com/aramirez087/TuitBot/issues>
- Security-sensitive reports should not be disclosed publicly before triage.
