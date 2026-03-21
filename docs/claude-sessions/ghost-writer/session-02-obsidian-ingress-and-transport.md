# Session 02: Obsidian Ingress And Transport

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/epic-charter.md`, `docs/roadmap/obsidian-ghostwriter-edge/ghostwriter-architecture.md`, `docs/roadmap/obsidian-ghostwriter-edge/privacy-and-deployment-matrix.md`, `docs/roadmap/obsidian-ghostwriter-edge/implementation-map.md`, and `docs/roadmap/obsidian-ghostwriter-edge/session-01-handoff.md`.

Mission
Create the Obsidian-side Ghostwriter entry path so a user can send the active selection or block context into TuitBot through one explicit, documented transport.

Repository anchors
- `plugins/openclaw-tuitbot/package.json`
- `plugins/openclaw-tuitbot/src/index.ts`
- `dashboard/src-tauri/src/lib.rs`
- `dashboard/src/lib/utils/obsidianUri.ts`
- `README.md`
- `plugins/obsidian-tuitbot/package.json`
- `plugins/obsidian-tuitbot/manifest.json`
- `plugins/obsidian-tuitbot/tsconfig.json`
- `plugins/obsidian-tuitbot/src/main.ts`

Tasks
1. Create the new `plugins/obsidian-tuitbot/` package, build setup, and Obsidian manifest using the repository's existing plugin conventions as reference.
2. Implement commands that capture the active editor selection and block context, including vault path, note path, heading hints, selected text, and editor position data.
3. Implement exactly one transport into the local TuitBot runtime, matching Session 01's architecture, and document why the rejected alternatives were not chosen.
4. Write the transport contract and payload examples for the backend session to consume next.

Deliverables
- `plugins/obsidian-tuitbot/package.json`
- `plugins/obsidian-tuitbot/manifest.json`
- `plugins/obsidian-tuitbot/tsconfig.json`
- `plugins/obsidian-tuitbot/src/main.ts`
- `plugins/obsidian-tuitbot/README.md`
- `docs/roadmap/obsidian-ghostwriter-edge/obsidian-plugin-contract.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-02-handoff.md`

Quality gates
- `npm --prefix plugins/obsidian-tuitbot install`
- `npm --prefix plugins/obsidian-tuitbot run build`
- The plugin contract contains concrete payload examples and trigger flows with no placeholders.

Exit criteria
- The Obsidian plugin can emit a real Ghostwriter payload through the chosen transport.
- Session 03 can implement the receiving side without reopening plugin UX or packaging decisions.
```
