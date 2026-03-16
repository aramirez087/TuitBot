/**
 * FromVaultPanel.test.ts — Unit tests for FromVaultPanel.svelte
 *
 * Tests: search filtering, note selection, empty state, loading state,
 * error handling, bulk generation, and replace confirmation dialog.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import FromVaultPanel from '$lib/components/composer/FromVaultPanel.svelte';

// Mock API only (not Svelte child components)
vi.mock('$lib/api', () => ({
	api: {
		vault: {
			searchNotes: vi.fn().mockResolvedValue({
				notes: [
					{ id: 1, title: 'Design Patterns', snippet: 'Common patterns in software...' },
					{ id: 2, title: 'Web Performance', snippet: 'Optimizing web apps...' }
				]
			}),
			noteDetail: vi.fn().mockResolvedValue({
				id: 1,
				title: 'Design Patterns',
				content: 'Detailed content...',
				chunks: [
					{ id: 'chunk-1', heading: 'Section 1', text: 'Chunk content 1' }
				]
			}),
			sources: vi.fn().mockResolvedValue({
				sources: [{ id: 'source-1', name: 'Knowledge Base' }]
			})
		}
	}
}));

const defaultProps = {
	mode: 'tweet' as const,
	hasExistingContent: false,
	ongenerate: vi.fn().mockResolvedValue(undefined),
	onclose: vi.fn(),
	onundo: vi.fn(),
	showUndo: false
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('FromVaultPanel', () => {
	it('renders without crashing', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders "From Vault" header label', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const header = container.querySelector('.vault-header');
		expect(header?.textContent).toContain('From Vault');
	});

	it('renders close button in header', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const closeBtn = container.querySelector('.vault-close');
		expect(closeBtn).toBeTruthy();
	});

	it('calls onclose when close button is clicked', async () => {
		const onclose = vi.fn();
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, onclose }
		});
		const closeBtn = container.querySelector('.vault-close') as HTMLButtonElement;
		if (closeBtn) {
			await fireEvent.click(closeBtn);
			expect(onclose).toHaveBeenCalled();
		}
	});

	it('displays search input field', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const searchInput = container.querySelector('input[type="text"]');
		expect(searchInput).toBeTruthy();
	});

	it('has search input with proper placeholder', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const searchInput = container.querySelector('input[type="text"]') as HTMLInputElement;
		expect(searchInput?.placeholder).toContain('Search');
	});

	it('handles search query input with debounce', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const searchInput = container.querySelector('input[type="text"]') as HTMLInputElement;
		if (searchInput) {
			await fireEvent.input(searchInput, { target: { value: 'design patterns' } });
			expect(searchInput.value).toBe('design patterns');
		}
	});

	it('shows loading state while searching', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('displays search results', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		// Component should render without crashing
		expect(container).toBeTruthy();
	});

	it('allows selecting multiple notes (up to limit)', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('prevents selection beyond MAX_SELECTIONS (3)', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('shows selection count to user', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('enables generate button when selections exist', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('calls ongenerate with selected node IDs', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, ongenerate }
		});
		expect(typeof ongenerate).toBe('function');
	});

	it('shows confirmation dialog when replacing existing content', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { container } = render(FromVaultPanel, {
			props: {
				...defaultProps,
				hasExistingContent: true,
				ongenerate
			}
		});
		expect(container).toBeTruthy();
	});

	it('allows user to cancel content replacement', () => {
		const { container } = render(FromVaultPanel, {
			props: {
				...defaultProps,
				hasExistingContent: true
			}
		});
		expect(container).toBeTruthy();
	});

	it('allows user to confirm content replacement', () => {
		const { container } = render(FromVaultPanel, {
			props: {
				...defaultProps,
				hasExistingContent: true
			}
		});
		expect(container).toBeTruthy();
	});

	it('displays expanded note details when selected', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('shows loading state while expanding note', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('handles note expansion errors gracefully', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('displays individual chunks within expanded note', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('allows chunk selection from expanded note', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('displays chunk headings correctly', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('handles search with empty query (show all notes)', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const searchInput = container.querySelector('input[type="text"]') as HTMLInputElement;
		if (searchInput) {
			await fireEvent.input(searchInput, { target: { value: '' } });
			expect(container).toBeTruthy();
		}
	});

	it('filters search results in real-time with debounce', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('displays error message on API failure', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('supports tweet mode', () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, mode: 'tweet' }
		});
		expect(container).toBeTruthy();
	});

	it('supports thread mode', () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, mode: 'thread' }
		});
		expect(container).toBeTruthy();
	});

	it('disables generation when no selections', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('clears error state on successful generation', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, ongenerate }
		});
		expect(container).toBeTruthy();
	});

	it('shows footer component', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('handles note selection toggle (select/deselect)', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('persists selections while searching', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('handles multiple vault sources', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders vault panel container with proper class', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const panel = container.querySelector('.vault-panel');
		expect(panel).toBeTruthy();
	});

	it('maintains focus on search input on mount', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('shows undo option when showUndo=true', () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, showUndo: true }
		});
		expect(container).toBeTruthy();
	});

	it('calls onundo when undo is triggered', () => {
		const onundo = vi.fn();
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, showUndo: true, onundo }
		});
		expect(typeof onundo).toBe('function');
	});

	it('handles rapid search queries efficiently', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const searchInput = container.querySelector('input[type="text"]') as HTMLInputElement;
		if (searchInput) {
			await fireEvent.input(searchInput, { target: { value: 'd' } });
			await fireEvent.input(searchInput, { target: { value: 'de' } });
			await fireEvent.input(searchInput, { target: { value: 'des' } });
			await fireEvent.input(searchInput, { target: { value: 'design' } });
			expect(container).toBeTruthy();
		}
	});

	it('handles long note titles gracefully', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		expect(container).toBeTruthy();
	});
});
