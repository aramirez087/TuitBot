/**
 * Keyboard shortcut utilities — stateless helpers for matching and displaying shortcuts.
 * No global registry; each component checks its own shortcuts via matchEvent().
 */

export interface ShortcutDef {
	combo: string;
	label: string;
	category: 'Mode' | 'Compose' | 'AI' | 'Thread';
	when?: 'thread' | 'tweet' | 'always';
}

let _isMac: boolean | null = null;

export function isMac(): boolean {
	if (_isMac === null) {
		_isMac =
			typeof navigator !== 'undefined' &&
			/Mac|iPod|iPhone|iPad/.test(navigator.platform ?? '');
	}
	return _isMac;
}

/**
 * Check whether a KeyboardEvent matches a shortcut combo string.
 * Combo format: modifiers joined by `+`, ending with a key name.
 *   - `cmd` → metaKey on Mac, ctrlKey elsewhere
 *   - `shift` → shiftKey
 *   - `alt` → altKey
 *   - Key names are compared lowercase against event.key.toLowerCase()
 * Examples: "cmd+shift+f", "alt+arrowup", "cmd+k", "tab", "shift+tab"
 */
export function matchEvent(e: KeyboardEvent, combo: string): boolean {
	const parts = combo.toLowerCase().split('+');
	const key = parts[parts.length - 1];
	const mods = new Set(parts.slice(0, -1));

	const wantCmd = mods.has('cmd');
	const wantShift = mods.has('shift');
	const wantAlt = mods.has('alt');

	const cmdPressed = isMac() ? e.metaKey : e.ctrlKey;

	if (wantCmd !== cmdPressed) return false;
	if (wantShift !== e.shiftKey) return false;
	if (wantAlt !== e.altKey) return false;

	// Ensure the non-cmd modifier key isn't pressed unexpectedly
	if (isMac() && e.ctrlKey) return false;
	if (!isMac() && e.metaKey) return false;

	return e.key.toLowerCase() === key;
}

/**
 * Format a combo string for display using platform-appropriate symbols.
 */
export function formatCombo(combo: string): string {
	const parts = combo.toLowerCase().split('+');
	const mac = isMac();
	const symbols: string[] = [];

	for (const part of parts.slice(0, -1)) {
		switch (part) {
			case 'cmd':
				symbols.push(mac ? '⌘' : 'Ctrl');
				break;
			case 'shift':
				symbols.push(mac ? '⇧' : 'Shift');
				break;
			case 'alt':
				symbols.push(mac ? '⌥' : 'Alt');
				break;
		}
	}

	const key = parts[parts.length - 1];
	switch (key) {
		case 'arrowup':
			symbols.push('↑');
			break;
		case 'arrowdown':
			symbols.push('↓');
			break;
		case 'enter':
			symbols.push(mac ? '↩' : 'Enter');
			break;
		case 'escape':
			symbols.push('Esc');
			break;
		case 'tab':
			symbols.push('Tab');
			break;
		default:
			symbols.push(key.toUpperCase());
			break;
	}

	return mac ? symbols.join('') : symbols.join('+');
}

/** Complete shortcut catalog for CommandPalette display. */
export const SHORTCUT_CATALOG: ShortcutDef[] = [
	{ combo: 'cmd+enter', label: 'Submit / Post', category: 'Compose', when: 'always' },
	{ combo: 'cmd+shift+f', label: 'Toggle focus mode', category: 'Mode', when: 'always' },
	{ combo: 'cmd+k', label: 'Open command palette', category: 'Mode', when: 'always' },
	{ combo: 'cmd+j', label: 'AI improve selection', category: 'AI', when: 'always' },
	{ combo: 'escape', label: 'Close modal / palette', category: 'Mode', when: 'always' },
	{ combo: 'cmd+shift+n', label: 'Switch to tweet mode', category: 'Mode', when: 'always' },
	{ combo: 'cmd+shift+t', label: 'Switch to thread mode', category: 'Mode', when: 'always' },
	{ combo: 'alt+arrowup', label: 'Move card up', category: 'Thread', when: 'thread' },
	{ combo: 'alt+arrowdown', label: 'Move card down', category: 'Thread', when: 'thread' },
	{ combo: 'cmd+d', label: 'Duplicate card', category: 'Thread', when: 'thread' },
	{ combo: 'cmd+shift+s', label: 'Split at cursor', category: 'Thread', when: 'thread' },
	{ combo: 'cmd+shift+m', label: 'Merge with next', category: 'Thread', when: 'thread' },
	{ combo: 'tab', label: 'Next card', category: 'Thread', when: 'thread' },
	{ combo: 'shift+tab', label: 'Previous card', category: 'Thread', when: 'thread' }
];
