# Keyboard Shortcut Cheatsheet

Quick reference for all keyboard shortcuts available in the Tuitbot Compose Modal.

## How Shortcuts Work

- **Mac:** `Cmd` is shown as `⌘`, `Alt` as `⌥`, `Shift` as `⇧`
- **Windows/Linux:** `Cmd` maps to `Ctrl`; `Alt` and `Shift` are unchanged
- Shortcuts are active only while the Compose Modal is open
- Thread-specific shortcuts only fire when the compose mode is set to **Thread**
- The Tauri desktop app has no shortcut conflicts; in-browser usage may conflict with some browser defaults (e.g., `Ctrl+D` for bookmarks — the shortcut will still fire because the modal captures the event first)

---

## Compose — Always Available

| Action | Mac | Windows / Linux | Description |
|--------|-----|-----------------|-------------|
| Submit / Post | `⌘↩` | `Ctrl+Enter` | Submit the current content (posts immediately or schedules if a time is selected) |
| Open command palette | `⌘K` | `Ctrl+K` | Open fuzzy-search over all compose actions |
| Toggle focus mode | `⌘⇧F` | `Ctrl+Shift+F` | Full-viewport distraction-free editing; all other shortcuts remain active |
| AI improve selection | `⌘J` | `Ctrl+J` | Run AI Improve on selected text (or full tweet if no selection) |
| Switch to tweet mode | `⌘⇧N` | `Ctrl+Shift+N` | Switch compose mode to single tweet |
| Switch to thread mode | `⌘⇧T` | `Ctrl+Shift+T` | Switch compose mode to thread |
| Close | `Esc` | `Esc` | Closes in cascade order: command palette → from-notes panel → focus mode → modal |

## Thread Operations — Thread Mode Only

| Action | Mac | Windows / Linux | Description |
|--------|-----|-----------------|-------------|
| Move card up | `⌥↑` | `Alt+↑` | Move the focused card one position up in the thread |
| Move card down | `⌥↓` | `Alt+↓` | Move the focused card one position down in the thread |
| Duplicate card | `⌘D` | `Ctrl+D` | Copy the current card (text + media) as a new card below |
| Split at cursor | `⌘⇧S` | `Ctrl+Shift+S` | Split the current card into two at the cursor position |
| Merge with next | `⌘⇧M` | `Ctrl+Shift+M` | Combine the current card with the card below it |
| Next card | `Tab` | `Tab` | Move focus to the next thread card |
| Previous card | `⇧Tab` | `Shift+Tab` | Move focus to the previous thread card |

---

## Command Palette Actions (⌘K / Ctrl+K)

The command palette provides fuzzy search over all compose actions. Each action belongs to a category and some display a shortcut hint inline.

### Mode

| Action | Shortcut Hint |
|--------|---------------|
| Toggle focus mode | `⌘⇧F` / `Ctrl+Shift+F` |
| Switch to Tweet | `⌘⇧N` / `Ctrl+Shift+N` |
| Switch to Thread | `⌘⇧T` / `Ctrl+Shift+T` |

### Compose

| Action | Shortcut Hint |
|--------|---------------|
| Submit / Post now | `⌘↩` / `Ctrl+Enter` |
| Attach media | — |

### AI

| Action | Shortcut Hint |
|--------|---------------|
| AI Improve | `⌘J` / `Ctrl+J` |
| Generate from notes | — |

### Thread (visible only in thread mode)

| Action | Shortcut Hint |
|--------|---------------|
| Add tweet card | — |
| Duplicate card | `⌘D` / `Ctrl+D` |
| Split at cursor | `⌘⇧S` / `Ctrl+Shift+S` |
| Merge with next | `⌘⇧M` / `Ctrl+Shift+M` |
| Move card up | `⌥↑` / `Alt+↑` |
| Move card down | `⌥↓` / `Alt+↓` |

---

## Palette Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move selection through results |
| `Enter` | Execute the selected action |
| `Esc` | Close the palette (returns to modal) |
| Type any text | Filter actions by label or category |

---

## Notes

- **Escape cascade:** Pressing Escape follows a priority order — it first closes the command palette (if open), then the from-notes panel, then exits focus mode, and finally closes the modal. Each press handles one level.
- **Focus mode:** All shortcuts work identically inside focus mode. The expanded viewport does not change any key bindings.
- **Thread mode gating:** Thread-specific shortcuts (`Alt+↑/↓`, `⌘D`, `⌘⇧S`, `⌘⇧M`, `Tab`, `Shift+Tab`) are only active when the compose mode is set to Thread. They do nothing in Tweet mode.
- **Source of truth:** Shortcut definitions live in `dashboard/src/lib/utils/shortcuts.ts` (`SHORTCUT_CATALOG`). Command palette actions live in `dashboard/src/lib/components/CommandPalette.svelte` (`allActions`).
