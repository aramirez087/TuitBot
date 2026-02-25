# PRDV1-05: Auth Pro (Diagnostics, Scope Checks, Safe Logging)

## Goal

Make authentication production-ready with actionable diagnostics, secure token
handling, and compatibility for legacy/media needs.

## Already built (do not redo)

- OAuth2 PKCE auth flow exists (`tuitbot auth`).
- Token refresh is implemented in `x_api/auth.rs`.
- Tokens stored on disk with restricted permissions.

## Gaps to implement

1. Actionable diagnostics in `tuitbot test`.
   - Missing scope detection (`tweet.write`, `like.write`, media-related scopes).
   - Read-only token detection and remediation guidance.
2. Scope-aware readiness checks.
   - Map missing scope to specific features/tools broken.
3. Verbose logging hygiene.
   - Guarantee no tokens/secrets in verbose output and error logs.
4. OAuth1a fallback for media/legacy endpoints (if required by deployment constraints).

## Primary code touchpoints

- `crates/tuitbot-cli/src/commands/test.rs`
- `crates/tuitbot-cli/src/commands/auth.rs`
- `crates/tuitbot-core/src/startup.rs`
- `crates/tuitbot-core/src/x_api/auth.rs`
- `crates/tuitbot-core/src/x_api/client.rs`
- `crates/tuitbot-core/src/config/mod.rs`
- `docs/configuration.md`
- `docs/operations.md`

## Implementation tasks

1. Extend token metadata checks.
   - Parse and compare granted scopes against required tool scopes.
2. Upgrade `tuitbot test` output.
   - Add auth diagnostics section:
     - granted scopes
     - missing scopes
     - expired/refresh status
     - explicit “how to fix” actions
3. Add “auth doctor” subcommand (or equivalent within `test`).
   - machine-readable JSON output for automation.
4. Add log redaction guardrails.
   - redact bearer tokens, client secrets, refresh tokens from logs/errors.
5. Add OAuth1a optional config + fallback path.
   - only for endpoints where OAuth2 is insufficient (media/legacy).
6. Add docs for auth matrix.
   - which endpoints/scopes required, and fallback behavior.

## Acceptance criteria

- `tuitbot test` clearly tells users what scope is missing and what to do next.
- No secrets appear in verbose logs.
- Media path works with OAuth2 and falls back when configured.
- Auth failures are typed and actionable.

## Verification commands

```bash
cargo test -p tuitbot-cli test
cargo test -p tuitbot-core x_api::auth
cargo test -p tuitbot-core startup
```

## Out of scope

- Policy routing logic (PRDV1-02).
- Bilingual QA workflows (PRDV1-06).
