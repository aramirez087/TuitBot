# Session 04 Prompt: CI Doc Sync and Drift Guards

## Use this as your Claude Code prompt

You are implementing Session 04 of the "Best X MCP for Full and Read-Only Teams" epic in `/Users/aramirez/Code/ReplyGuy`.

Goal:
- Make docs and shipped MCP behavior impossible to drift.

Hard requirements:
- CI validates tool lists/counts per profile against committed docs artifacts.
- Fail fast on mismatch.
- Process must be maintainable for future profile/tool additions.

Scope for this session:
1. Add a docs artifact path for generated profile manifests, for example:
   - `docs/generated/mcp-manifest-full.json`
   - `docs/generated/mcp-manifest-readonly.json`
   - `docs/generated/mcp-manifest-api-readonly.json`
2. Add generation/verification script(s), preferably in `scripts/`:
   - generate manifests from CLI
   - compare against committed artifacts
   - validate tool counts and profile names
3. Update CI workflow (`.github/workflows/ci.yml`) to run:
   - manifest generation check
   - docs sync check
4. Update docs build flow if needed so `mkdocs build --strict` also reflects generated content checks.
5. Add contributor instructions in docs for how to refresh generated artifacts.

Implementation guidance:
- Prefer deterministic JSON formatting and stable key ordering.
- Keep script output actionable (print exact mismatch and refresh command).
- Docs and generated artifacts should use canonical profile names only.

Validation commands:
- `cargo run -p tuitbot-cli -- mcp manifest --profile full`
- `cargo run -p tuitbot-cli -- mcp manifest --profile readonly`
- `cargo run -p tuitbot-cli -- mcp manifest --profile api-readonly`
- `bash scripts/<your-sync-check-script>.sh`
- `mkdocs build --strict`

Deliverables:
- CI gate in place for manifest/doc drift.
- Generated artifacts committed.
- Clear refresh instructions for maintainers.
