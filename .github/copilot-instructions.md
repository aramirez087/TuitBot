# Copilot Code Review Instructions

## Architecture

This is a Rust + Svelte 5 workspace with four crates:

| Crate | Role |
|---|---|
| `tuitbot-core` | All business logic, data models, typed errors |
| `tuitbot-cli` | Thin CLI binary wrapping core |
| `tuitbot-server` | HTTP/WebSocket routing only — **zero business logic** |
| `tuitbot-mcp` | Model Context Protocol integration |

### Dependency layers (strict, no violations)

```
Toolkit (core/toolkit/) ← Workflow (core/workflow/) ← Autopilot (core/automation/)
```

- No upward imports (workflow must not import automation, toolkit must not import workflow)
- No skip-level imports (automation must not import toolkit directly)
- Flag any `use crate::toolkit` in automation or `use crate::automation` in workflow/toolkit

### Server boundary

`tuitbot-server` must contain only routing, serialization, WebSocket fan-out, and auth. Any data transformation, validation beyond request parsing, or domain logic belongs in `tuitbot-core`. Flag violations.

## Rust conventions

### Error handling
- `tuitbot-core`: `thiserror` with typed enums per domain (e.g., `ConfigError`, `XApiError`, `ToolkitError`)
- Binary crates (`tuitbot-cli`, `tuitbot-server`): `anyhow` with `.context()` for wrapping
- Wrapped errors must use `#[source]` for chain traversal
- `#[allow(clippy::...)]` requires a comment explaining why

### File size
- Max **500 lines** per `.rs` file — convert to module directory if exceeded (`foo.rs` → `foo/mod.rs` + submodules)
- Test modules over 100 lines should be extracted to a `tests.rs` submodule

### Axum routes
- Literal routes (`/approval/stats`) must appear before parameterized routes (`/approval/{id}`)

### SQLx
- Dynamic `WHERE IN` clauses must use parameterized placeholders bound in a loop — never string interpolation

### Linting standard
- All warnings are errors: `RUSTFLAGS="-D warnings"` and `clippy -- -D warnings`
- Workspace-level `clippy::pedantic = "warn"` is enabled

## Frontend conventions (Svelte 5)

### Runes only — no legacy patterns
```svelte
<!-- ✓ Correct -->
let { title, onaction } = $props();
let count = $state(0);
let doubled = $derived(count * 2);

<!-- ✗ Reject these -->
export let title;          // old Svelte 4
$: doubled = count * 2;   // old reactive
<svelte:component this={X} />  // use $derived + <Icon /> instead
```

### File size
- Max **400 lines** per `+page.svelte` — extract into sibling `*Section.svelte` components

### Styling
- Use design tokens: `var(--color-surface)`, `var(--color-accent)`, `var(--color-border)`, etc.
- No hardcoded hex colors — always reference `app.css` tokens
- TypeScript strict mode (`lang="ts"`, no `any` types)
- Indentation: **tabs** (not spaces) in all dashboard files

### Checks
- `npm run check` (svelte-check) must pass — there is no `lint` script
- `npm run build` must succeed

## Release discipline (crates/)

- `Cargo.lock` must be committed
- Every `path = "../..."` dependency must include an explicit `version` field
- Crate metadata required: `description`, `license`, `repository`, `homepage`, `documentation`, `keywords`

## Common issues to flag

| Pattern | Problem | Suggestion |
|---|---|---|
| Business logic in server route handler | Layering violation | Move to `tuitbot-core` |
| `export let` in `.svelte` file | Legacy Svelte 4 | Use `$props()` |
| Hardcoded color hex in component | Breaks theming | Use `var(--color-*)` token |
| `#[allow(...)]` without comment | Hidden tech debt | Add justification comment |
| SQL with string interpolation | Injection risk | Use parameterized placeholders |
| File exceeding size limit | Maintainability | Refactor into module/components |
| Missing `#[source]` on error variant wrapping another error | Broken error chain | Add `#[source]` attribute |
| `<svelte:component>` usage | Deprecated in Svelte 5 | Use capitalized `$derived` variable |
| `@const` inside a `<div>` | Invalid Svelte syntax | Only valid in `{#if}`, `{#each}`, `{:else}`, `{#snippet}` blocks |
| Literal route after parameterized route in Axum | Route shadowing | Reorder: literals first |
