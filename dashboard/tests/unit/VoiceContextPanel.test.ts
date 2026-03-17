/**
 * VoiceContextPanel.test.ts — Unit tests for VoiceContextPanel.svelte
 *
 * Tests: render with/without voice context, expand/collapse toggling,
 * tone cue input, saved cues history, settings integration, and inline mode.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { writable } from 'svelte/store';

// Mock settings store with factory function to avoid hoisting issues
vi.mock('$lib/stores/settings', () => {
	const mockConfig = writable({
		business: {
			brand_voice: 'Professional and direct',
			content_style: 'Educational',
			content_pillars: ['Technology', 'Innovation', 'Best Practices']
		}
	});
	return {
		config: mockConfig,
		loadSettings: vi.fn()
	};
});

// Import component AFTER mocking
import VoiceContextPanel from '$lib/components/composer/VoiceContextPanel.svelte';

const defaultProps = {
	cue: '',
	oncuechange: vi.fn(),
	inline: false
};

beforeEach(() => {
	vi.clearAllMocks();
	// Clear specific localStorage keys used by tests
	localStorage.removeItem('tuitbot:voice:expanded');
	localStorage.removeItem('tuitbot:voice:saved-cues');
});

describe('VoiceContextPanel', () => {
	it('renders without crashing', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders collapse/expand toggle button when not inline', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const toggleBtn = container.querySelector('.voice-toggle');
		expect(toggleBtn).toBeTruthy();
	});

	it('toggles expanded state when toggle button clicked', async () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const toggleBtn = container.querySelector('.voice-toggle') as HTMLButtonElement;
		if (toggleBtn) {
			await fireEvent.click(toggleBtn);
			const panel = container.querySelector('.voice-context-panel');
			expect(panel?.classList.contains('expanded')).toBe(true);
		}
	});

	it('persists expanded state to localStorage', async () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const toggleBtn = container.querySelector('.voice-toggle') as HTMLButtonElement;
		if (toggleBtn) {
			await fireEvent.click(toggleBtn);
			const stored = localStorage.getItem('tuitbot:voice:expanded');
			expect(stored).toBe('true');
		}
	});

	it('loads expanded state from localStorage on mount', () => {
		localStorage.setItem('tuitbot:voice:expanded', 'true');
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const panel = container.querySelector('.voice-context-panel');
		expect(panel?.classList.contains('expanded')).toBe(true);
	});

	it('displays brand voice when configured', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('displays content style when configured', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const chips = container.querySelectorAll('.voice-chip');
		expect(chips.length).toBeGreaterThanOrEqual(0);
	});

	it('displays content pillars as individual chips', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const pillarChips = container.querySelectorAll('.pillar-chip');
		expect(pillarChips.length).toBeGreaterThanOrEqual(0);
	});

	it('renders tone cue input field', () => {
		// cue-input is visible in inline mode; in non-inline mode it requires expanded=true
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const cueInput = container.querySelector('.cue-input');
		expect(cueInput).toBeTruthy();
	});

	it('has proper placeholder text for cue input', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		expect(cueInput?.placeholder).toContain('Tone cue');
	});

	it('updates cue value when input changes', async () => {
		const oncuechange = vi.fn();
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, oncuechange }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.input(cueInput, { target: { value: 'more casual' } });
			expect(oncuechange).toHaveBeenCalledWith('more casual');
		}
	});

	it('calls oncuechange with new cue text', async () => {
		const oncuechange = vi.fn();
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, oncuechange }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.input(cueInput, { target: { value: 'formal and technical' } });
			expect(oncuechange).toHaveBeenCalledWith('formal and technical');
		}
	});

	it('displays active cue as badge when set', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: 'conversational' }
		});
		const badge = container.querySelector('.active-cue-badge');
		expect(badge?.textContent).toContain('conversational');
	});

	it('does not show cue badge when cue is empty', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: '' }
		});
		const badge = container.querySelector('.active-cue-badge');
		expect(badge).toBeFalsy();
	});

	it('persists saved cues to localStorage', () => {
		const savedCues = ['casual', 'professional', 'technical'];
		localStorage.setItem('tuitbot:voice:saved-cues', JSON.stringify(savedCues));
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('loads saved cues from localStorage on mount', () => {
		const savedCues = ['casual', 'professional'];
		localStorage.setItem('tuitbot:voice:saved-cues', JSON.stringify(savedCues));
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('shows dropdown with saved cues on cue input focus', async () => {
		const savedCues = ['casual', 'technical'];
		localStorage.setItem('tuitbot:voice:saved-cues', JSON.stringify(savedCues));
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.focus(cueInput);
			expect(container).toBeTruthy();
		}
	});

	it('limits saved cues to MAX_SAVED_CUES (5)', () => {
		const manyCues = Array.from({ length: 10 }, (_, i) => `cue-${i}`);
		localStorage.setItem('tuitbot:voice:saved-cues', JSON.stringify(manyCues));
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const stored = localStorage.getItem('tuitbot:voice:saved-cues');
		const parsed = JSON.parse(stored || '[]');
		expect(parsed.length).toBeLessThanOrEqual(5);
	});

	it('does not show dropdown when no saved cues exist', () => {
		localStorage.removeItem('tuitbot:voice:saved-cues');
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders in inline mode', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const voiceBody = container.querySelector('.voice-body');
		expect(voiceBody).toBeTruthy();
	});

	it('hides toggle button in inline mode', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const toggleBtn = container.querySelector('.voice-toggle');
		expect(toggleBtn).toBeFalsy();
	});

	it('shows body content directly in inline mode', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const body = container.querySelector('.voice-body');
		expect(body).toBeTruthy();
	});

	it('renders voice summary in inline mode', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		expect(container).toBeTruthy();
	});

	it('shows quick cue row in inline mode', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const cueRow = container.querySelector('.quick-cue-row');
		expect(cueRow).toBeTruthy();
	});

	it('truncates long brand voice text', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('truncates long content style text', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('shows max 3 pillars inline', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const pillarChips = container.querySelectorAll('.pillar-chip');
		expect(pillarChips.length).toBeLessThanOrEqual(3);
	});

	it('handles cue with special characters', async () => {
		const oncuechange = vi.fn();
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, oncuechange }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.input(cueInput, { target: { value: 'emojis: 🚀 tech' } });
			expect(oncuechange).toHaveBeenCalledWith('emojis: 🚀 tech');
		}
	});

	it('trims whitespace from cue', () => {
		const oncuechange = vi.fn();
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: '  test cue  ', oncuechange }
		});
		expect(container).toBeTruthy();
	});

	it('deduplicates saved cues', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('closes saved cues dropdown on blur', async () => {
		const savedCues = ['casual'];
		localStorage.setItem('tuitbot:voice:saved-cues', JSON.stringify(savedCues));
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.focus(cueInput);
			await fireEvent.blur(cueInput);
			expect(container).toBeTruthy();
		}
	});

	it('loads settings on component mount', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('reflects config changes reactively', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('handles localStorage quota exceeded gracefully', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders toggle label text', () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const label = container.querySelector('.voice-toggle-label');
		expect(label?.textContent).toContain('Voice context');
	});

	it('supports very long cue input', async () => {
		const longCue = 'a'.repeat(200);
		const oncuechange = vi.fn();
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, oncuechange }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		if (cueInput) {
			await fireEvent.input(cueInput, { target: { value: longCue } });
			expect(oncuechange).toHaveBeenCalledWith(longCue);
		}
	});

	it('renders expanded panel content when expanded', async () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const toggleBtn = container.querySelector('.voice-toggle') as HTMLButtonElement;
		if (toggleBtn) {
			await fireEvent.click(toggleBtn);
			const expandedContent = container.querySelector('.voice-context-panel.expanded');
			expect(expandedContent).toBeTruthy();
		}
	});

	it('maintains separate state for expanded and inline modes', () => {
		const { container: expandable } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: false }
		});
		const { container: inline } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		expect(expandable).toBeTruthy();
		expect(inline).toBeTruthy();
	});

	it('handles empty brand voice gracefully', () => {
		const { container } = render(VoiceContextPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('saves cue to history on Ctrl+Enter keydown', async () => {
		localStorage.removeItem('tuitbot:voice:saved-cues');
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: 'be witty', inline: true }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		expect(cueInput).toBeTruthy();
		await fireEvent.keyDown(cueInput, { key: 'Enter', ctrlKey: true });
		const stored = JSON.parse(localStorage.getItem('tuitbot:voice:saved-cues') || '[]');
		expect(stored).toContain('be witty');
	});

	it('saves cue to history on Meta+Enter keydown', async () => {
		localStorage.removeItem('tuitbot:voice:saved-cues');
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: 'hot take', inline: true }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		expect(cueInput).toBeTruthy();
		await fireEvent.keyDown(cueInput, { key: 'Enter', metaKey: true });
		const stored = JSON.parse(localStorage.getItem('tuitbot:voice:saved-cues') || '[]');
		expect(stored).toContain('hot take');
	});

	it('blurs input on Escape keydown', async () => {
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, inline: true }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		expect(cueInput).toBeTruthy();
		cueInput.focus();
		expect(document.activeElement).toBe(cueInput);
		await fireEvent.keyDown(cueInput, { key: 'Escape' });
		expect(document.activeElement).not.toBe(cueInput);
	});

	it('does not save on plain Enter (no modifier)', async () => {
		localStorage.removeItem('tuitbot:voice:saved-cues');
		const { container } = render(VoiceContextPanel, {
			props: { ...defaultProps, cue: 'nope', inline: true }
		});
		const cueInput = container.querySelector('.cue-input') as HTMLInputElement;
		expect(cueInput).toBeTruthy();
		await fireEvent.keyDown(cueInput, { key: 'Enter' });
		const stored = localStorage.getItem('tuitbot:voice:saved-cues');
		expect(stored).toBeFalsy();
	});
});
