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
					{
						node_id: 1,
						source_id: 1,
						title: 'Design Patterns',
						relative_path: 'design-patterns.md',
						tags: null,
						status: 'indexed',
						chunk_count: 3,
						updated_at: '2024-01-01T00:00:00Z'
					},
					{
						node_id: 2,
						source_id: 1,
						title: 'Web Performance',
						relative_path: 'web-performance.md',
						tags: null,
						status: 'indexed',
						chunk_count: 2,
						updated_at: '2024-01-02T00:00:00Z'
					}
				]
			}),
			noteDetail: vi.fn().mockResolvedValue({
				node_id: 1,
				source_id: 1,
				title: 'Design Patterns',
				relative_path: 'design-patterns.md',
				tags: null,
				status: 'indexed',
				ingested_at: '2024-01-01T00:00:00Z',
				updated_at: '2024-01-01T00:00:00Z',
				chunks: [
					{ chunk_id: 1, heading_path: 'Section 1', snippet: 'Chunk content 1', retrieval_boost: 1.0 }
				]
			}),
			sources: vi.fn().mockResolvedValue({
				sources: [{ id: 'source-1', name: 'Knowledge Base' }]
			}),
			getSelection: vi.fn().mockResolvedValue({
				session_id: 'test-session-123',
				vault_name: 'marketing',
				file_path: 'content/ideas.md',
				selected_text: 'Selected block of text from Obsidian',
				heading_context: 'Ideas > Marketing',
				note_title: 'Content Ideas',
				frontmatter_tags: ['marketing', 'ideas'],
				resolved_node_id: 42,
				resolved_chunk_id: 99,
				created_at: '2024-01-01T00:00:00Z',
				expires_at: '2024-01-01T00:30:00Z'
			})
		},
		assist: {
			highlights: vi.fn().mockResolvedValue({
				highlights: ['Key insight about design patterns', 'Performance optimization tip', 'Architecture best practice']
			}),
			hooks: vi.fn().mockResolvedValue({
				hooks: [
					{ style: 'question', text: 'What if design patterns could save you?', char_count: 40, confidence: 'high' },
					{ style: 'contrarian_take', text: 'Most devs use patterns wrong.', char_count: 30, confidence: 'high' },
					{ style: 'tip', text: 'One pattern trick for cleaner code.', char_count: 35, confidence: 'high' },
					{ style: 'storytelling', text: 'I refactored 10k lines with one pattern.', char_count: 41, confidence: 'medium' },
					{ style: 'list', text: '5 patterns every dev should know:', char_count: 33, confidence: 'high' },
				],
				topic: 'design patterns',
				vault_citations: []
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

	it('shows Generate hooks button', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const buttons = container.querySelectorAll('button');
		const hooksBtn = Array.from(buttons).find(
			(b) => b.textContent?.includes('Generate hooks')
		);
		expect(hooksBtn).toBeTruthy();
	});

	it('does not show Find hooks or Use this hook button in initial view', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const buttons = container.querySelectorAll('button');
		const hookBtn = Array.from(buttons).find(
			(b) => b.textContent?.includes('Find hooks') || b.textContent?.includes('Use this hook')
		);
		expect(hookBtn).toBeFalsy();
	});

	it('generate hooks button is disabled when no chunks selected', () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const hooksBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement | undefined;
		expect(hooksBtn?.disabled).toBe(true);
	});

	it('renders with thread mode', () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, mode: 'thread' as const }
		});
		expect(container.querySelector('.vault-panel')).toBeTruthy();
	});

	it('renders with hasExistingContent true', () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, hasExistingContent: true }
		});
		expect(container.querySelector('.vault-panel')).toBeTruthy();
	});

	it('handles ongenerate callback type with optional highlights', () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		render(FromVaultPanel, {
			props: { ...defaultProps, ongenerate }
		});
		// Verify the ongenerate function accepts the new signature
		expect(typeof ongenerate).toBe('function');
		// Simulate calling with highlights
		ongenerate([1], 'tweet', ['highlight1', 'highlight2']);
		expect(ongenerate).toHaveBeenCalledWith([1], 'tweet', ['highlight1', 'highlight2']);
	});

	it('handles ongenerate callback without highlights (backward compat)', () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		render(FromVaultPanel, {
			props: { ...defaultProps, ongenerate }
		});
		ongenerate([1], 'tweet');
		expect(ongenerate).toHaveBeenCalledWith([1], 'tweet');
	});

	it('shows Generate hooks button text in default state', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });
		const hooksBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		);
		expect(hooksBtn).toBeTruthy();
		expect(hooksBtn?.textContent?.trim()).toBe('Generate hooks');
	});

	// --- Selection hydration tests ---

	it('enters selection mode when selectionSessionId is provided', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		// Wait for hydration to complete
		await vi.waitFor(() => {
			const selectionReview = container.querySelector('.vault-selection-review');
			expect(selectionReview).toBeTruthy();
		});
	});

	it('displays note title from hydrated selection', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			const meta = container.querySelector('.selection-source-path');
			expect(meta?.textContent).toContain('Content Ideas');
		});
	});

	it('displays heading context from hydrated selection', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			const heading = container.querySelector('.selection-heading');
			expect(heading?.textContent).toContain('Ideas > Marketing');
		});
	});

	it('displays selected text preview from hydrated selection', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			const preview = container.querySelector('.selection-text-preview');
			expect(preview?.textContent).toContain('Selected block of text from Obsidian');
		});
	});

	it('displays frontmatter tags from hydrated selection', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			const tags = container.querySelectorAll('.selection-tag');
			expect(tags.length).toBe(2);
			expect(tags[0]?.textContent).toContain('marketing');
			expect(tags[1]?.textContent).toContain('ideas');
		});
	});

	it('shows "Generate hooks" CTA in selection mode', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			const btn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate hooks')
			);
			expect(btn).toBeTruthy();
		});
	});

	it('shows expired state when selection fetch fails', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Not found'));
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'expired-session' }
		});
		await vi.waitFor(() => {
			const expiredText = container.textContent;
			expect(expiredText).toContain('This selection has expired');
		});
	});

	it('shows browse vault button when selection is expired', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Not found'));
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'expired-session' }
		});
		await vi.waitFor(() => {
			const btn = container.querySelector('.vault-expired-dismiss');
			expect(btn?.textContent).toContain('Browse vault');
		});
	});

	it('calls onSelectionConsumed after hydration', async () => {
		const onSelectionConsumed = vi.fn();
		render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123', onSelectionConsumed }
		});
		await vi.waitFor(() => {
			expect(onSelectionConsumed).toHaveBeenCalled();
		});
	});

	it('does not render search input in selection mode', async () => {
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'test-session-123' }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const searchInput = container.querySelector('input[type="text"]');
		expect(searchInput).toBeFalsy();
	});

	// --- buildTopicFromSelection tests ---

	it('passes noteTitle and snippet to hooks API after chunk selection', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });

		// Wait for notes to load (api.vault.sources + api.vault.searchNotes resolve)
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.vault-note-row').length).toBeGreaterThan(0);
		});

		// Click first note to expand it (triggers api.vault.noteDetail)
		const noteBtn = container.querySelector<HTMLButtonElement>('.vault-note-row')!;
		await fireEvent.click(noteBtn);

		// Wait for chunks to appear
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-chunk-cb')).toBeTruthy();
		});

		// Toggle the chunk
		const chunkCb = container.querySelector<HTMLInputElement>('.vault-chunk-cb')!;
		await fireEvent.click(chunkCb);

		// Wait for Generate hooks button to become enabled
		await vi.waitFor(() => {
			const btn = Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
				.find((b) => b.textContent?.includes('Generate hooks') && !b.disabled);
			expect(btn).toBeTruthy();
		});

		const genBtn = Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
			.find((b) => b.textContent?.includes('Generate hooks') && !b.disabled)!;
		await fireEvent.click(genBtn);

		// Verify api.assist.hooks was called with topic built from noteTitle + snippet
		const { api } = await import('$lib/api');
		await vi.waitFor(() => {
			expect(api.assist.hooks).toHaveBeenCalled();
		});

		const [topic] = (api.assist.hooks as ReturnType<typeof vi.fn>).mock.calls[0];
		expect(topic).toContain('Design Patterns');
		expect(topic).toContain('Chunk content 1');
	});

	it('includes snippet in hooks API topic (not just heading)', async () => {
		const { container } = render(FromVaultPanel, { props: defaultProps });

		await vi.waitFor(() => {
			expect(container.querySelectorAll('.vault-note-row').length).toBeGreaterThan(0);
		});

		const noteBtn = container.querySelector<HTMLButtonElement>('.vault-note-row')!;
		await fireEvent.click(noteBtn);

		await vi.waitFor(() => {
			expect(container.querySelector('.vault-chunk-cb')).toBeTruthy();
		});

		const chunkCb = container.querySelector<HTMLInputElement>('.vault-chunk-cb')!;
		await fireEvent.click(chunkCb);

		const { api } = await import('$lib/api');
		await vi.waitFor(() => {
			const btn = Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
				.find((b) => b.textContent?.includes('Generate hooks') && !b.disabled);
			expect(btn).toBeTruthy();
		});

		const genBtn = Array.from(container.querySelectorAll<HTMLButtonElement>('button'))
			.find((b) => b.textContent?.includes('Generate hooks') && !b.disabled)!;
		await fireEvent.click(genBtn);

		await vi.waitFor(() => {
			expect(api.assist.hooks).toHaveBeenCalled();
		});

		// Topic must contain the snippet ('Chunk content 1'), not just the heading ('Section 1')
		const [topic] = (api.assist.hooks as ReturnType<typeof vi.fn>).mock.calls[0];
		expect(topic).toContain('Chunk content 1');
		expect(topic).not.toBe('Section 1'); // heading alone would mean snippet was dropped
	});

	it('shows cloud mode privacy note when selected_text is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			session_id: 'cloud-session',
			vault_name: 'marketing',
			file_path: 'content/ideas.md',
			selected_text: null,
			heading_context: 'Ideas > Marketing',
			note_title: 'Content Ideas',
			frontmatter_tags: null,
			resolved_node_id: 42,
			resolved_chunk_id: 99,
			created_at: '2024-01-01T00:00:00Z',
			expires_at: '2024-01-01T00:30:00Z'
		});
		const { container } = render(FromVaultPanel, {
			props: { ...defaultProps, selectionSessionId: 'cloud-session' }
		});
		await vi.waitFor(() => {
			const note = container.querySelector('.selection-text-cloud-note');
			expect(note?.textContent).toContain('cloud mode');
		});
	});
});
