/**
 * VaultSelectionReview.test.ts — Unit tests for VaultSelectionReview.svelte
 *
 * Tests: loading state, selection display, expired state, hook generation,
 * HookPicker rendering, replace confirmation, back navigation, error states,
 * onSelectionConsumed callback, and edge cases (null note_title, null selected_text).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import VaultSelectionReview from '$lib/components/composer/VaultSelectionReview.svelte';

// Mock API
vi.mock('$lib/api', () => ({
	api: {
		vault: { getSelection: vi.fn() },
		assist: { hooks: vi.fn() }
	}
}));

const sampleNeighbors = [
	{
		node_id: 55,
		node_title: 'Async Patterns',
		reason: 'linked_note',
		reason_label: 'linked note',
		intent: 'pro_tip',
		matched_tags: [],
		score: 3.5,
		snippet: 'Async patterns in Rust use tokio.',
		best_chunk_id: 120,
		heading_path: '# Async',
		relative_path: 'notes/async-patterns.md',
	},
	{
		node_id: 78,
		node_title: 'Tokio Runtime',
		reason: 'shared_tag',
		reason_label: 'shared tag: #async',
		intent: 'evidence',
		matched_tags: ['async'],
		score: 1.8,
		snippet: 'Tokio provides a multi-threaded runtime.',
		best_chunk_id: 145,
		heading_path: '# Runtime',
		relative_path: 'notes/tokio-runtime.md',
	},
];

const sampleSelection = {
	session_id: 'test-session-abc',
	vault_name: 'my-vault',
	file_path: 'notes/article.md',
	selected_text: 'The key insight here is that async patterns differ by context.',
	heading_context: 'Patterns > Async',
	note_title: 'Design Patterns',
	frontmatter_tags: ['patterns', 'async'],
	resolved_node_id: 7,
	resolved_chunk_id: 22,
	created_at: '2024-01-01T00:00:00Z',
	expires_at: '2024-01-01T00:30:00Z'
};

const sampleSelectionWithGraph = {
	...sampleSelection,
	graph_neighbors: sampleNeighbors,
	graph_state: 'available' as const,
};

const sampleHooks = [
	{ style: 'question', text: 'What if async patterns could simplify everything?', char_count: 50, confidence: 'high' as const },
	{ style: 'contrarian_take', text: 'Most devs use async wrong.', char_count: 28, confidence: 'high' as const },
	{ style: 'tip', text: 'One async trick for cleaner code.', char_count: 33, confidence: 'high' as const },
	{ style: 'storytelling', text: 'I rewrote async code and it changed everything.', char_count: 47, confidence: 'medium' as const },
	{ style: 'list', text: '5 async patterns every dev needs:', char_count: 34, confidence: 'high' as const },
];

const defaultProps = {
	sessionId: 'test-session-abc',
	outputFormat: 'tweet' as const,
	hasExistingContent: false,
	showUndo: false,
	onundo: vi.fn(),
	ongenerate: vi.fn().mockResolvedValue(undefined),
	onSelectionConsumed: vi.fn(),
	onexpired: vi.fn(),
	onformatchange: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('VaultSelectionReview', () => {
	// --- Loading state ---

	it('renders loading shimmer initially before fetch resolves', async () => {
		// getSelection never resolves in this tick — component stays in loading state
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockReturnValue(new Promise(() => {}));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		const shimmer = container.querySelector('.vault-loading-shimmer');
		expect(shimmer).toBeTruthy();
	});

	it('renders "Loading selection..." text while loading', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockReturnValue(new Promise(() => {}));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		expect(container.textContent).toContain('Loading selection...');
	});

	// --- Successful selection display ---

	it('shows selection review container after successful fetch', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
	});

	it('displays note_title in selection source path', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const path = container.querySelector('.selection-source-path');
			expect(path?.textContent).toContain('Design Patterns');
		});
	});

	it('displays heading_context when present', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const heading = container.querySelector('.selection-heading');
			expect(heading?.textContent).toContain('Patterns > Async');
		});
	});

	it('displays selected_text in text preview', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const preview = container.querySelector('.selection-text-preview');
			expect(preview?.textContent).toContain('The key insight here is that async patterns differ by context.');
		});
	});

	it('renders frontmatter tags', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const tags = container.querySelectorAll('.selection-tag');
			expect(tags.length).toBe(2);
			expect(tags[0]?.textContent).toContain('patterns');
			expect(tags[1]?.textContent).toContain('async');
		});
	});

	it('does not render tags section when frontmatter_tags is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			frontmatter_tags: null
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const tags = container.querySelectorAll('.selection-tag');
		expect(tags.length).toBe(0);
	});

	it('does not render tags section when frontmatter_tags is empty array', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			frontmatter_tags: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const tags = container.querySelectorAll('.selection-tag');
		expect(tags.length).toBe(0);
	});

	// --- Fallback: file_path when note_title is null ---

	it('shows file_path when note_title is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			note_title: null
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const path = container.querySelector('.selection-source-path');
			expect(path?.textContent).toContain('notes/article.md');
		});
	});

	// --- Cloud mode: selected_text null ---

	it('shows cloud mode privacy note when selected_text is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			selected_text: null
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const cloudNote = container.querySelector('.selection-text-cloud-note');
			expect(cloudNote?.textContent).toContain('cloud mode');
		});
	});

	it('does not show selection-text-preview when selected_text is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			selected_text: null
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.selection-text-cloud-note')).toBeTruthy();
		});
		expect(container.querySelector('.selection-text-preview')).toBeFalsy();
	});

	// --- Expired state ---

	it('shows expired state when getSelection rejects', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Not found'));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.textContent).toContain('This selection has expired');
		});
	});

	it('shows hint text in expired state', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Gone'));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const hint = container.querySelector('.vault-empty-hint');
			expect(hint).toBeTruthy();
		});
	});

	it('shows Browse vault button in expired state', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Gone'));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const btn = container.querySelector('.vault-expired-dismiss');
			expect(btn?.textContent).toContain('Browse vault');
		});
	});

	it('Browse vault button calls onexpired', async () => {
		const onexpired = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Gone'));
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, onexpired }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-expired-dismiss')).toBeTruthy();
		});
		const btn = container.querySelector('.vault-expired-dismiss') as HTMLButtonElement;
		await fireEvent.click(btn);
		expect(onexpired).toHaveBeenCalled();
	});

	// --- onSelectionConsumed callback ---

	it('calls onSelectionConsumed after successful load', async () => {
		const onSelectionConsumed = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		render(VaultSelectionReview, {
			props: { ...defaultProps, onSelectionConsumed }
		});
		await vi.waitFor(() => {
			expect(onSelectionConsumed).toHaveBeenCalled();
		});
	});

	it('calls onSelectionConsumed even when getSelection fails', async () => {
		const onSelectionConsumed = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Expired'));
		render(VaultSelectionReview, {
			props: { ...defaultProps, onSelectionConsumed }
		});
		await vi.waitFor(() => {
			expect(onSelectionConsumed).toHaveBeenCalled();
		});
	});

	// --- Generate hooks (handleGenerate) ---

	it('VaultFooter shows Generate hooks button after load', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const btn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate hooks')
			);
			expect(btn).toBeTruthy();
		});
	});

	it('clicking Generate hooks calls api.assist.hooks', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const btn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate hooks')
			);
			expect(btn).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(api.assist.hooks).toHaveBeenCalledWith(
				expect.any(String),
				{ sessionId: 'test-session-abc' }
			);
		});
	});

	it('api.assist.hooks is called with selected_text as topic when present', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(api.assist.hooks).toHaveBeenCalledWith(
				'The key insight here is that async patterns differ by context.',
				expect.any(Object)
			);
		});
	});

	// --- HookPicker rendered after hooks load ---

	it('renders HookPicker after hooks load', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
	});

	it('HookPicker replaces selection review view after hooks load', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		expect(container.querySelector('.vault-selection-review')).toBeFalsy();
	});

	// --- Hook error state ---

	it('returns to selection view after api.assist.hooks rejects (hookOptions stays null)', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Hook generation failed'));
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		// After hooks fail, hookOptions stays null and hookLoading goes false →
		// component falls back to selection view (not HookPicker branch)
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
			expect(container.querySelector('.hook-picker')).toBeFalsy();
		});
	});

	// --- handleHookSelected → calls ongenerate ---

	it('handleHookSelected calls ongenerate with nodeIds, format, hookText, hookStyle', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		// Select first hook card
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			expect(ongenerate).toHaveBeenCalledWith(
				[7],
				'tweet',
				['What if async patterns could simplify everything?'],
				'question',
				undefined
			);
		});
	});

	it('handleHookSelected passes empty nodeIds when resolved_node_id is null', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			resolved_node_id: null
		});
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			expect(ongenerate).toHaveBeenCalledWith(
				[],
				expect.any(String),
				expect.any(Array),
				expect.any(String),
				undefined
			);
		});
	});

	// --- handleHookSelected with hasExistingContent → replace confirmation ---

	it('does not call ongenerate on first confirm when hasExistingContent=true (waits for replace)', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, hasExistingContent: true, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		// First confirm: sets confirmReplace=true and returns — ongenerate NOT called
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		// Small tick to let any async settle
		await new Promise((r) => setTimeout(r, 10));
		expect(ongenerate).not.toHaveBeenCalled();
	});

	it('calls ongenerate on second confirm (confirmReplace bypass) when hasExistingContent=true', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, hasExistingContent: true, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		// First click: sets confirmReplace=true
		await fireEvent.click(confirmBtn);
		await new Promise((r) => setTimeout(r, 10));
		expect(ongenerate).not.toHaveBeenCalled();
		// Second click: confirmReplace=true already, so ongenerate is called
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			expect(ongenerate).toHaveBeenCalled();
		});
	});

	// --- Back from hooks (handleBackFromHooks) ---

	it('back button in HookPicker clears hookOptions and returns to selection view', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const backBtn = container.querySelector('.hook-back') as HTMLButtonElement;
		await fireEvent.click(backBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		expect(container.querySelector('.hook-picker')).toBeFalsy();
	});

	// --- Regenerate hooks ---

	it('regenerate hooks button calls api.assist.hooks again', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const callCountAfterFirst = (api.assist.hooks as ReturnType<typeof vi.fn>).mock.calls.length;
		const regenBtn = container.querySelector('.hook-regen-btn') as HTMLButtonElement;
		await fireEvent.click(regenBtn);
		await vi.waitFor(() => {
			expect((api.assist.hooks as ReturnType<typeof vi.fn>).mock.calls.length).toBeGreaterThan(callCountAfterFirst);
		});
	});

	// --- Error during generation (ongenerate rejects) ---

	it('sets error state when ongenerate rejects', async () => {
		const ongenerate = vi.fn().mockRejectedValue(new Error('Server error'));
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			const errorEl = container.querySelector('.vault-error');
			expect(errorEl?.textContent).toContain('Server error');
		});
	});

	// --- Format change propagation ---

	it('calls onformatchange when format toggle is changed in VaultFooter', async () => {
		const onformatchange = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, onformatchange }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		const threadBtn = container.querySelectorAll('.vault-format-opt')[1] as HTMLButtonElement;
		await fireEvent.click(threadBtn);
		expect(onformatchange).toHaveBeenCalledWith('thread');
	});

	// --- Undo visibility ---

	it('shows undo button when showUndo=true', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const onundo = vi.fn();
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, showUndo: true, onundo }
		});
		await vi.waitFor(() => {
			const undoBtn = container.querySelector('.vault-undo-btn');
			expect(undoBtn).toBeTruthy();
		});
	});

	it('calls onundo when undo button clicked', async () => {
		const onundo = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, showUndo: true, onundo }
		});
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-undo-btn')).toBeTruthy();
		});
		const undoBtn = container.querySelector('.vault-undo-btn') as HTMLButtonElement;
		await fireEvent.click(undoBtn);
		expect(onundo).toHaveBeenCalled();
	});

	it('does not show undo button when showUndo=false', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		expect(container.querySelector('.vault-undo-btn')).toBeFalsy();
	});

	// --- VaultFooter selectionMode label ---

	it('VaultFooter generate button reads "Generate hooks" (selectionMode=true)', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			const btn = container.querySelector('.vault-generate-btn');
			expect(btn?.textContent?.trim()).toBe('Generate hooks');
		});
	});

	// --- Loading state clears after fetch ---

	it('loading shimmer disappears after fetch completes', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-loading-shimmer')).toBeFalsy();
		});
	});

	// --- heading_context absent ---

	it('does not render heading div when heading_context is null', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			heading_context: null
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		expect(container.querySelector('.selection-heading')).toBeFalsy();
	});

	// --- Graph suggestion cards integration ---

	it('shows GraphSuggestionCards when selection has graph_neighbors', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.graph-suggestions')).toBeTruthy();
		});
		const cards = container.querySelectorAll('.graph-card');
		expect(cards.length).toBe(2);
	});

	it('shows empty state when graph_state is no_related_notes', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			graph_neighbors: [],
			graph_state: 'no_related_notes',
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.textContent).toContain("doesn't link to other indexed notes");
		});
	});

	it('shows not-indexed message when graph_state is node_not_indexed', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			graph_neighbors: [],
			graph_state: 'node_not_indexed',
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.textContent).toContain("hasn't been indexed yet");
		});
	});

	it('does not show graph section when graph_state is fallback_active', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			...sampleSelection,
			graph_neighbors: [],
			graph_state: 'fallback_active',
		});
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		expect(container.querySelector('.graph-suggestions')).toBeFalsy();
		// Toggle should also not appear for fallback
		expect(container.querySelector('.synthesis-toggle')).toBeFalsy();
	});

	it('does not show graph section when selection has no graph fields', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});
		// fallback_active is the default when graph_state is undefined
		expect(container.querySelector('.graph-suggestions')).toBeFalsy();
	});

	it('synthesis toggle hides graph cards when toggled off', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.graph-suggestions')).toBeTruthy();
		});
		// Toggle off
		const toggle = container.querySelector('.synthesis-toggle') as HTMLButtonElement;
		expect(toggle).toBeTruthy();
		await fireEvent.click(toggle);
		expect(container.querySelector('.graph-suggestions')).toBeFalsy();
	});

	it('synthesis toggle shows graph cards when toggled back on', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.graph-suggestions')).toBeTruthy();
		});
		const toggle = container.querySelector('.synthesis-toggle') as HTMLButtonElement;
		// Toggle off then on
		await fireEvent.click(toggle);
		expect(container.querySelector('.graph-suggestions')).toBeFalsy();
		await fireEvent.click(toggle);
		expect(container.querySelector('.graph-suggestions')).toBeTruthy();
	});

	it('dismiss neighbor removes it from visible list', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		const dismissBtns = container.querySelectorAll('.graph-card-dismiss');
		await fireEvent.click(dismissBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(1);
		});
	});

	it('accepted neighbors are included in ongenerate call', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Accept first neighbor
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		// Shows accepted summary
		await vi.waitFor(() => {
			expect(container.textContent).toContain('1 note included in context');
		});
		// Generate hooks
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		// Select first hook
		const hookCards = container.querySelectorAll('.hook-card');
		await fireEvent.click(hookCards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			expect(ongenerate).toHaveBeenCalled();
			const call = ongenerate.mock.calls[0];
			// nodeIds should include both selection node (7) and accepted neighbor (55)
			expect(call[0]).toContain(7);
			expect(call[0]).toContain(55);
		});
	});

	it('synthesis toggle has correct aria-pressed attribute', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.synthesis-toggle')).toBeTruthy();
		});
		const toggle = container.querySelector('.synthesis-toggle') as HTMLButtonElement;
		expect(toggle.getAttribute('aria-pressed')).toBe('true');
		await fireEvent.click(toggle);
		expect(toggle.getAttribute('aria-pressed')).toBe('false');
	});

	// --- Slot targeting integration (Session 5) ---

	it('shows SlotTargetPanel when hasExistingContent and accepted neighbors exist', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, {
			props: {
				...defaultProps,
				hasExistingContent: true,
				threadBlocks: [
					{ id: 'b1', text: 'Opening', media_paths: [], order: 0 },
					{ id: 'b2', text: 'Closing', media_paths: [], order: 1 },
				],
				mode: 'thread',
			}
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Accept first neighbor
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelector('.slot-target-panel')).toBeTruthy();
		});
	});

	it('does not show SlotTargetPanel when no existing content', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, hasExistingContent: false }
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		await vi.waitFor(() => {
			expect(container.textContent).toContain('1 note included in context');
		});
		expect(container.querySelector('.slot-target-panel')).toBeFalsy();
	});

	it('slot insert callback is fired with correct neighbor and slot index', async () => {
		const oninsert = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, {
			props: {
				...defaultProps,
				hasExistingContent: true,
				threadBlocks: [
					{ id: 'b1', text: 'Opening text', media_paths: [], order: 0 },
					{ id: 'b2', text: 'Closing text', media_paths: [], order: 1 },
				],
				mode: 'thread',
				oninsert,
			}
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Accept first neighbor
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelector('.slot-target-panel')).toBeTruthy();
		});
		// Click "Apply" in the SlotTargetPanel
		const applyBtn = container.querySelector('.apply-btn') as HTMLButtonElement;
		await fireEvent.click(applyBtn);
		expect(oninsert).toHaveBeenCalledWith(
			expect.objectContaining({ node_id: 55, node_title: 'Async Patterns' }),
			0,
			'Opening hook'
		);
	});

	it('tweet mode: insert targets single slot', async () => {
		const oninsert = vi.fn();
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, {
			props: {
				...defaultProps,
				hasExistingContent: true,
				mode: 'tweet',
				oninsert,
			}
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelector('.slot-target-panel')).toBeTruthy();
		});
		const applyBtn = container.querySelector('.apply-btn') as HTMLButtonElement;
		await fireEvent.click(applyBtn);
		expect(oninsert).toHaveBeenCalledWith(
			expect.objectContaining({ node_id: 55 }),
			0,
			'Tweet'
		);
	});

	it('shows dismissed recovery section after dismissing a card', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Dismiss first neighbor
		const dismissBtns = container.querySelectorAll('.graph-card-dismiss');
		await fireEvent.click(dismissBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(1);
		});
		// "Show skipped" toggle should appear
		const toggle = container.querySelector('.dismissed-toggle');
		expect(toggle).toBeTruthy();
		expect(toggle?.textContent).toContain('Show skipped');
		expect(toggle?.textContent).toContain('1');
	});

	it('restoring a dismissed card adds it back to suggestions', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Dismiss first neighbor
		const dismissBtns = container.querySelectorAll('.graph-card-dismiss');
		await fireEvent.click(dismissBtns[0]);
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(1);
		});
		// Expand dismissed list
		const toggle = container.querySelector('.dismissed-toggle') as HTMLButtonElement;
		await fireEvent.click(toggle);
		await vi.waitFor(() => {
			expect(container.querySelector('.dismissed-list')).toBeTruthy();
		});
		// Restore the dismissed card
		const restoreBtn = container.querySelector('.dismissed-restore') as HTMLButtonElement;
		await fireEvent.click(restoreBtn);
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Dismissed section should disappear
		expect(container.querySelector('.dismissed-toggle')).toBeFalsy();
	});

	it('ongenerate includes neighbor provenance edge_type and edge_label', async () => {
		const ongenerate = vi.fn().mockResolvedValue(undefined);
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelectionWithGraph);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue({
			hooks: sampleHooks, topic: 'test', vault_citations: []
		});
		const { container } = render(VaultSelectionReview, {
			props: { ...defaultProps, ongenerate }
		});
		await vi.waitFor(() => {
			expect(container.querySelectorAll('.graph-card').length).toBe(2);
		});
		// Accept first neighbor
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		// Generate and confirm
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate hooks')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);
		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
		const hookCards = container.querySelectorAll('.hook-card');
		await fireEvent.click(hookCards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		await vi.waitFor(() => {
			expect(ongenerate).toHaveBeenCalled();
			const call = ongenerate.mock.calls[0];
			const neighborProv = call[4];
			expect(neighborProv).toBeDefined();
			expect(neighborProv[0]).toEqual(
				expect.objectContaining({ node_id: 55, edge_type: 'linked_note', edge_label: 'linked note' })
			);
		});
	});
});
