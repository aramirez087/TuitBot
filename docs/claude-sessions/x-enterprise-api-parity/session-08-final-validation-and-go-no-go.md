# Session 08: Final Validation And Go No-Go

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Mission: Perform full validation of the enterprise API parity initiative and publish a release-quality go/no-go decision.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-07-handoff.md`
- `docs/roadmap/x-enterprise-api-parity/charter.md`
- `docs/generated/mcp-manifest-write.json`
- `docs/generated/mcp-manifest-admin.json`
- `docs/generated/mcp-manifest-readonly.json`
- `docs/generated/mcp-manifest-api-readonly.json`
- `docs/generated/coverage-report.json`
- `scripts/check-mcp-manifests.sh`
- `scripts/run-conformance.sh`

Tasks:
1. Run mandatory Rust quality gates and capture pass/fail output summaries.
2. Run manifest sync verification and conformance harness; resolve any drift or failing checks.
3. Verify that DM, Ads/Campaign, and enterprise-admin tools are present in admin manifest and correctly excluded from non-admin profiles.
4. Validate docs consistency against generated manifests and coverage report.
5. Produce final go/no-go report with risks, mitigations, rollback plan, and explicit release recommendation.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/final-go-no-go-report.md`
- `docs/roadmap/x-enterprise-api-parity/session-08-handoff.md`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- All required checks pass with no unresolved blockers.
- Manifest/profile boundaries for enterprise tools are verified and documented.
- Final go/no-go report gives a single explicit recommendation with evidence.
```
