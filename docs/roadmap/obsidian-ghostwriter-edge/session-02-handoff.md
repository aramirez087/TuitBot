# Session 02 Handoff: Obsidian Ghostwriter Entry Path

## What Changed

New Obsidian plugin package (`plugins/obsidian-tuitbot/`) and transport contract document. Zero changes to existing source code.

### Files Created

| File | Purpose |
|---|---|
| `plugins/obsidian-tuitbot/package.json` | NPM package with esbuild build pipeline |
| `plugins/obsidian-tuitbot/manifest.json` | Obsidian plugin manifest (desktop-only, minAppVersion 1.4.0) |
| `plugins/obsidian-tuitbot/tsconfig.json` | TypeScript config for bundler module resolution |
| `plugins/obsidian-tuitbot/src/context.ts` | Pure helper functions: `resolveHeadingPath()`, `extractBlock()`, payload types |
| `plugins/obsidian-tuitbot/src/main.ts` | Plugin entry point: commands, settings, token auth, HTTP transport |
| `plugins/obsidian-tuitbot/src/__tests__/payload.test.ts` | 15 unit tests for heading resolution and block extraction |
| `plugins/obsidian-tuitbot/main.js` | Compiled plugin bundle (esbuild output) |
| `plugins/obsidian-tuitbot/README.md` | Plugin documentation |
| `docs/roadmap/obsidian-ghostwriter-edge/obsidian-plugin-contract.md` | Transport contract with JSON schemas, examples, error catalog, backend checklist |
| `docs/roadmap/obsidian-ghostwriter-edge/session-02-handoff.md` | This file |

## Decisions Made

### Decision 1: HTTP POST to localhost API

**Chose**: `POST http://127.0.0.1:3001/api/vault/send-selection` with Bearer token auth.
**Rejected**: Clipboard bridge (fragile, no ack), file-system watcher (operator rules prohibit daemons), custom IPC (unnecessary new transport), obsidian:// URI (one-way), WebSocket (connection lifecycle overhead).
**Rationale**: TuitBot already has a full HTTP API on `127.0.0.1:3001`. Using it is zero new infrastructure. Aligns with Session 01 Decision 4.

### Decision 2: Auth via `~/.tuitbot/api_token` file

**Chose**: Read the token file the server already creates, cache in memory.
**Rejected**: New auth mechanism, OAuth, API key management UI.
**Rationale**: Matches how Tauri desktop app and CLI authenticate. Cross-platform path resolution via `HOME`/`USERPROFILE`.

### Decision 3: Plugin packaging follows Obsidian conventions

**Chose**: `esbuild` bundling to single `main.js`, Obsidian `manifest.json` schema, `Plugin` base class.
**Rejected**: Following OpenClaw plugin conventions (different runtime, different entry point pattern).
**Rationale**: Obsidian plugins run inside Obsidian's runtime, not Node.js. Must follow Obsidian community plugin standards.

### Decision 4: Pure logic separated into `context.ts`

**Chose**: Extract `resolveHeadingPath()`, `extractBlock()`, and type definitions into `context.ts` (no Obsidian imports).
**Rejected**: Keeping everything in `main.ts` (would prevent unit testing without Obsidian runtime mocks).
**Rationale**: The `obsidian` npm package is types-only with no proper module export. Separating pure logic allows Node.js test runner to import and test without Obsidian.

### Decision 5: Two commands, no UI beyond Notices

**Chose**: "Send selection to TuitBot" and "Send current block to TuitBot" as editor commands. No ribbon icon, no settings tab, no status bar.
**Rejected**: Settings tab UI, ribbon icon, sidebar panel.
**Rationale**: Operator constraint: "command-driven selection handoff, no sync engine, no background daemon." Minimal surface. Settings are JSON-editable in `data.json`.

## Quality Gates Passed

| Gate | Result |
|---|---|
| `npm --prefix plugins/obsidian-tuitbot install` | 21 packages, 0 errors |
| `npm --prefix plugins/obsidian-tuitbot run build` | `main.js` produced (8.7kb), 0 errors |
| `npm --prefix plugins/obsidian-tuitbot run test` | 15/15 tests pass |
| Contract completeness | JSON schemas, 3 examples, error catalog, trigger flow, backend checklist |

## Open Risks

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| 1 | **Server endpoint doesn't exist yet** | Expected | By design — plugin is the sender half. Will return connection error until Session 6 implements `POST /api/vault/send-selection`. Documented in README and contract. |
| 2 | **`requestUrl` may not reach localhost in some Obsidian configs** | Medium | `requestUrl` bypasses CORS. If blocked, fallback to Node.js `http` module (available in desktop Obsidian). |
| 3 | **`~/.tuitbot/api_token` path on Windows** | Medium | Uses `process.env.USERPROFILE` as fallback. Document the Windows path. |
| 4 | **`metadataCache` may be stale for recently edited notes** | Low | Cache is accurate enough for heading resolution. Selected text is the primary payload — heading context is supplementary. |
| 5 | **Plugin can't be integration-tested in CI** | Low | 15 unit tests cover pure logic. Build gate verifies type correctness against Obsidian types. Manual testing required for integration. |
| 6 | **`obsidian` types package version drift** | Low | Pinned to `^1.7.2`. `minAppVersion: 1.4.0` ensures heading URI support. |

## Session 3 Inputs

Session 3 (Hooks Panel Frontend) does NOT depend on Session 2. It consumes:
1. `ghostwriter-architecture.md` § "2. Hook-First Composer Panel" — UX flow and component spec.
2. `implementation-map.md` § "Session 3" — file change list.
3. Session 2's Hooks API (from the original Session 2 scope, not this plugin session).

The session that implements `POST /api/vault/send-selection` (Session 6 per implementation map) consumes:
1. `obsidian-plugin-contract.md` — full request/response schemas, error catalog, backend checklist.
2. `ghostwriter-architecture.md` § "6. Selection Ingress API" — table schema and TTL behavior.
3. `privacy-and-deployment-matrix.md` — Cloud-mode filtering for `GET /api/vault/selection/{session_id}`.

### Session 6 Exit Criteria (Selection Ingress)

- `POST /api/vault/send-selection` stores selection and returns `session_id` (tested).
- `GET /api/vault/selection/{session_id}` retrieves stored selection (tested).
- 30-minute TTL is enforced with hourly cleanup (tested).
- Rate limit: 10 requests/min/account (tested).
- Cloud mode: GET endpoint does not return raw `selected_text` (tested).
- WebSocket `SelectionReceived` event emitted to account's dashboard clients (tested).
- All existing tests continue to pass. Zero compilation warnings.
