# Shortcut Regression Matrix

Covers every keyboard shortcut across all composer surfaces. Each cell describes the expected behavior for that shortcut in the given context.

**Legend**:
- **Fires** — shortcut is handled and produces the described action
- **Blocked** — shortcut is intercepted but intentionally suppressed (no action)
- **Propagates** — shortcut is not handled at this layer; browser default applies
- **N/A** — context cannot exist (e.g., palette shortcuts don't apply outside palette)

## Workspace-Level Shortcuts

Handled by `ComposeWorkspace.handleKeydown` on `svelte:window`.

| Shortcut | Tweet (Embedded) | Tweet (Modal) | Thread (Embedded) | Thread (Modal) | Preview Open | Palette Open |
|---|---|---|---|---|---|---|
| `Cmd+Enter` | Fires: Submit tweet | Fires: Submit tweet | Propagates to ThreadFlowLane card handler (split/add) | Propagates to ThreadFlowLane card handler (split/add) | Blocked | N/A |
| `Cmd+Shift+Enter` | Fires: Submit | Fires: Submit | Fires: Submit thread | Fires: Submit thread | Blocked | N/A |
| `Cmd+K` | Fires: Open palette | Fires: Open palette | Fires: Open palette | Fires: Open palette | Blocked | N/A |
| `Cmd+J` | Fires: AI improve + undo snapshot | Fires: AI improve + undo snapshot | Fires: AI improve focused block + undo snapshot | Fires: AI improve focused block + undo snapshot | Blocked | N/A |
| `Cmd+Shift+F` | Blocked (already full-page) | Fires: Toggle focus mode | Blocked (already full-page) | Fires: Toggle focus mode | Blocked | N/A |
| `Cmd+Shift+N` | Fires: Switch to tweet mode | Fires: Switch to tweet mode | Fires: Switch to tweet mode | Fires: Switch to tweet mode | Blocked | N/A |
| `Cmd+Shift+T` | Fires: Switch to thread mode | Fires: Switch to thread mode | Fires: Switch to thread mode | Fires: Switch to thread mode | Blocked | N/A |
| `Cmd+I` | Fires: Toggle inspector | Fires: Toggle inspector | Fires: Toggle inspector | Fires: Toggle inspector | Blocked | N/A |
| `Cmd+Shift+P` | Fires: Open preview overlay | Fires: Open preview overlay | Fires: Open preview overlay | Fires: Open preview overlay | Fires: Close preview overlay | N/A |
| `Escape` | Fires: Close layers (notes > inspector > focus) | Fires: Close layers or modal | Fires: Close layers (notes > inspector > focus) | Fires: Close layers or modal | Fires: Close preview | Fires: Close palette |

## Thread Card–Level Shortcuts

Handled by `ThreadFlowLane.handleCardKeydown` when a thread card textarea is focused. Only active in thread mode.

| Shortcut | Thread (Embedded) | Thread (Modal) | Notes |
|---|---|---|---|
| `Cmd+Enter` | Fires: Split at cursor or add block after (if cursor at end / empty block) | Same | `e.stopPropagation()` prevents workspace handler from firing |
| `Backspace` at pos 0 | Fires: Merge with previous block (if >2 blocks) | Same | Only at cursor position 0 with no selection |
| `Tab` | Fires: Focus next block | Same | `e.preventDefault()` blocks browser tab behavior |
| `Shift+Tab` | Fires: Focus previous block | Same | |
| `Alt+ArrowUp` | Fires: Move block up | Same | |
| `Alt+ArrowDown` | Fires: Move block down | Same | |
| `Cmd+D` | Fires: Duplicate block | Same | `e.preventDefault()` overrides browser bookmark dialog |
| `Cmd+Shift+S` | Fires: Split block at cursor | Same | Secondary split shortcut |
| `Cmd+Shift+M` | Fires: Merge with next block | Same | |

## Command Palette Shortcuts

When `CommandPalette` is open, `ComposeWorkspace.handleKeydown` returns early (`if (paletteOpen) return`). All shortcuts are handled by the palette's own `handleKeydown`.

| Shortcut | Behavior |
|---|---|
| `Escape` | Close palette |
| `ArrowDown` | Select next item |
| `ArrowUp` | Select previous item |
| `Enter` | Execute selected action |
| All other shortcuts | Propagate (no workspace interception) |

## Preview Overlay Shortcuts

When `previewMode` is true, only two shortcuts pass through the workspace handler. All others return early.

| Shortcut | Behavior |
|---|---|
| `Escape` | Close preview |
| `Cmd+Shift+P` | Toggle preview (close) |
| All other shortcuts | Blocked (early return) |

## Layout-Level Shortcuts

Handled by `+layout.svelte` on `window.addEventListener('keydown')`. These fire regardless of composer state.

| Shortcut | Behavior | Conflict with Composer |
|---|---|---|
| `Cmd+1..9` | Navigate to route | None — composer doesn't use bare digit shortcuts |
| `Cmd+,` | Go to settings | None |
| `Cmd+N` | Focus compose / navigate to `/` | None — `Cmd+Shift+N` (with shift) is separate |

## Shortcut Hints Consistency

All visible shortcut references must match the implemented behavior.

| Surface | Shortcut | Label | Matches Implementation |
|---|---|---|---|
| Tips tray (tweet mode) | `Cmd+Enter` | Publish | Yes — submits in tweet mode |
| Tips tray (thread mode) | `Cmd+Enter` | Split / add post | Yes — handled by ThreadFlowLane |
| Tips tray (both) | `Cmd+K` | Command palette | Yes |
| Tips tray (both) | `Cmd+J` | AI improve | Yes |
| Command palette | `Cmd+J` | AI improve (selection or post) | Yes — improves selection if any, full post otherwise |
| Command palette | `Cmd+Enter` (thread) | Split / add post | Yes |
| Command palette | `Cmd+Shift+Enter` (thread) / `Cmd+Enter` (tweet) | Publish | Yes — mode-aware shortcut display |
| HomeComposerHeader tooltip | `⌘J` | AI improve selection or post | Yes |
| HomeComposerHeader tooltip | `⌘⇧P` | Open/Close preview | Yes |
| SHORTCUT_CATALOG | `cmd+enter` (tweet) | Publish | Yes |
| SHORTCUT_CATALOG | `cmd+enter` (thread) | Split / add post below | Yes |
| SHORTCUT_CATALOG | `cmd+j` | AI improve (selection or full post) | Yes |

## Safety: Destructive Shortcut Audit

| Shortcut | Before Session 4 | After Session 4 | Risk Level |
|---|---|---|---|
| `Cmd+J` (no selection, tweet mode) | Replaced entire tweet text — **no undo** | Replaces entire tweet text — **undo snapshot + 10s banner** | Safe |
| `Cmd+J` (with selection, tweet mode) | Replaced selection — **no undo** | Replaces selection — **undo snapshot + 10s banner** | Safe |
| `Cmd+J` (thread mode, no selection) | Replaced entire focused block — **no undo** | Replaces entire focused block — **undo snapshot + 10s banner** | Safe |
| `Cmd+J` (empty text) | No-op (early return) | No-op (early return) | Safe |
| AI generate (palette) | Replaced all content — **no undo** | Unchanged (palette action, not a keyboard shortcut) | Acknowledged — out of scope for this session |
| From notes generation | Replaced all content — undo available | Unchanged — undo available | Safe |
