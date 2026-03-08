# Rail Interactions — Keyboard, Focus, and Quick Actions

## Focus Model

The rail uses the **roving tabindex** pattern (WAI-ARIA listbox). One item has `tabindex="0"` (the focused item); all others have `tabindex="-1"`. The rail container has `role="listbox"` and each item `role="option"`.

### Zone Transitions

| From | To | Key |
|------|-----|-----|
| Rail | Composer | `Tab` (natural focus order) |
| Composer | Rail | `Escape` |
| Rail | Tabs | Not keyboard-trapped; `1`-`4` switch tabs directly |

`Escape` from the composer zone calls `railComponent.focus()` on the shell level, which delegates to the focused rail item's native `focus()`.

## Keyboard Shortcut Map

All rail shortcuts are active only when a rail item has focus (keydown on the listbox, not window).

| Key | Action | Context |
|-----|--------|---------|
| `ArrowDown` | Move focus to next draft | Any tab |
| `ArrowUp` | Move focus to previous draft | Any tab |
| `Enter` | Select focused draft (open in composer) | Any tab |
| `N` | Create new draft | Any tab |
| `Delete` / `Backspace` | Archive focused draft (undo toast shown) | Active, Scheduled tabs |
| `D` | Duplicate focused draft | Any tab |
| `R` | Restore focused draft | Archive tab only |
| `1` | Switch to Drafts tab | Any tab |
| `2` | Switch to Scheduled tab | Any tab |
| `3` | Switch to Posted tab | Any tab |
| `4` | Switch to Archive tab | Any tab |

Modifier keys (`Cmd`, `Ctrl`, `Alt`) are checked — bare letter shortcuts only fire without modifiers to avoid conflicts with browser/OS shortcuts.

## Quick Actions

Each `DraftRailItem` displays action buttons on hover and keyboard focus:

- **Active/Scheduled tabs**: Duplicate (Copy icon) + Archive (Archive icon)
- **Archive tab**: Restore (RotateCcw icon)

Action buttons use `stopPropagation` to prevent the click from also selecting the draft.

### Visibility Rules

- `opacity: 0` by default
- `opacity: 1` when the parent item is hovered (`:hover`) or focused (`.focused` class)
- Actions are always keyboard-accessible via the rail shortcuts regardless of visibility

## Archive Undo Flow

1. User archives a draft (via keyboard `Delete`/`Backspace` or click)
2. Draft is immediately archived via `studio.archiveDraft(id)`
3. An undo toast appears at the bottom of the rail: "Draft archived — Undo"
4. Toast auto-dismisses after 5 seconds
5. Clicking "Undo" calls `studio.restoreDraft(id)` and clears the toast
6. Switching tabs also clears the toast (the archive is committed)

The toast uses `role="status"` and `aria-live="polite"` for screen reader announcement.

## Command Palette Integration

The command palette (`Cmd+K`) shows a "DraftStudio" category when in draft studio mode. Actions:

| Action | ID | Shortcut |
|--------|----|----------|
| New draft | `ds-new-draft` | `N` |
| Duplicate current draft | `ds-duplicate` | `D` |
| Archive current draft | `ds-archive` | `Backspace` |
| Jump to rail | `ds-jump-rail` | `Escape` |

These actions are injected via the `extraPaletteActions` prop on `ComposeWorkspace`, which forwards them to `CommandPalette`. Unknown action IDs are dispatched to the shell via the `ondraftaction` callback.

## Focus Management After Archive

When a draft is archived, `focusedIndex` is clamped to `Math.min(focusedIndex, drafts.length - 1)`. If the list becomes empty, the "New Draft" button receives focus.

## Accessibility

- `role="listbox"` on the rail list container
- `role="option"` and `aria-selected` on each item
- `role="tablist"` and `role="tab"` with `aria-selected` on tab buttons
- Focus ring via `box-shadow: inset 0 0 0 1.5px var(--color-accent)` on the `.focused` class
- Undo toast with `role="status"` and `aria-live="polite"`
- Action button groups with `role="group"` and `aria-label="Draft actions"`
