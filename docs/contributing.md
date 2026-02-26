# Contributing

## Local development

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

## Contribution scope

- bug fixes
- tests
- documentation
- performance and reliability improvements

## PR standards

- include rationale and impact
- include tests for behavior changes
- keep commits focused
- follow Conventional Commit style when possible

## Documentation updates

Any behavior, config, or CLI change should update docs in the same PR.

## Generated artifacts

The files in `docs/generated/` are auto-generated from Rust source and must not
be edited by hand:

- `mcp-manifest-write.json` — write profile tool manifest (104 tools)
- `mcp-manifest-admin.json` — admin profile tool manifest (108 tools)
- `mcp-manifest-readonly.json` — readonly profile tool manifest (14 tools)
- `mcp-manifest-api-readonly.json` — api-readonly profile tool manifest (40 tools)

After changing MCP tools or profiles, regenerate and commit in the same PR:

```bash
bash scripts/generate-mcp-manifests.sh
```

CI runs `scripts/check-mcp-manifests.sh` and rejects PRs where committed
artifacts have drifted from source.
