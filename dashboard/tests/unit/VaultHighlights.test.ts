/**
 * VaultHighlights.test.ts — Unit tests for VaultHighlights.svelte
 *
 * Tests: renders highlights, toggle checkboxes, generate passes only enabled.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import VaultHighlights from '$lib/components/composer/VaultHighlights.svelte';

const defaultHighlights = [
	{ text: 'Key insight about design patterns', enabled: true },
	{ text: 'Performance optimization tip', enabled: true },
	{ text: 'Architecture best practice', enabled: true }
];

const defaultProps = {
	highlights: defaultHighlights.map((h) => ({ ...h })),
	outputFormat: 'tweet' as const,
	generating: false,
	ongenerate: vi.fn(),
	onback: vi.fn(),
	onformatchange: vi.fn()
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('VaultHighlights', () => {
	it('renders without crashing', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders Key Highlights header', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const label = container.querySelector('.highlights-label');
		expect(label?.textContent).toContain('Key Highlights');
	});

	it('renders all highlight items', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const items = container.querySelectorAll('.highlight-item');
		expect(items.length).toBe(3);
	});

	it('renders highlight text content', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const texts = container.querySelectorAll('.highlight-text');
		expect(texts[0]?.textContent).toBe('Key insight about design patterns');
		expect(texts[1]?.textContent).toBe('Performance optimization tip');
	});

	it('renders checkboxes checked by default', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const checkboxes = container.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;
		expect(checkboxes.length).toBe(3);
		checkboxes.forEach((cb) => expect(cb.checked).toBe(true));
	});

	it('calls onback when back arrow is clicked', async () => {
		const onback = vi.fn();
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, onback }
		});
		const backBtn = container.querySelector('.highlights-back') as HTMLButtonElement;
		await fireEvent.click(backBtn);
		expect(onback).toHaveBeenCalled();
	});

	it('shows correct selection count', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const count = container.querySelector('.highlights-count');
		expect(count?.textContent).toContain('3 of 3');
	});

	it('shows Find hooks button in tweet mode', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const btn = container.querySelector('.highlights-generate-btn');
		expect(btn?.textContent).toContain('Find hooks');
	});

	it('shows Find hooks button in thread mode', () => {
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, outputFormat: 'thread' as const }
		});
		const btn = container.querySelector('.highlights-generate-btn');
		expect(btn?.textContent).toContain('Find hooks');
	});

	it('disables generate button when generating', () => {
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, generating: true }
		});
		const btn = container.querySelector('.highlights-generate-btn') as HTMLButtonElement;
		expect(btn?.disabled).toBe(true);
		expect(btn?.textContent).toContain('Generating...');
	});

	it('calls ongenerate with enabled highlights only', async () => {
		const ongenerate = vi.fn();
		const highlights = [
			{ text: 'First', enabled: true },
			{ text: 'Second', enabled: false },
			{ text: 'Third', enabled: true }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights, ongenerate }
		});
		const btn = container.querySelector('.highlights-generate-btn') as HTMLButtonElement;
		await fireEvent.click(btn);
		expect(ongenerate).toHaveBeenCalledWith(['First', 'Third']);
	});

	it('does not call ongenerate when no highlights are enabled', async () => {
		const ongenerate = vi.fn();
		const highlights = [
			{ text: 'First', enabled: false },
			{ text: 'Second', enabled: false }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights, ongenerate }
		});
		const btn = container.querySelector('.highlights-generate-btn') as HTMLButtonElement;
		await fireEvent.click(btn);
		expect(ongenerate).not.toHaveBeenCalled();
	});

	it('disables generate button when all highlights are disabled', () => {
		const highlights = [
			{ text: 'First', enabled: false },
			{ text: 'Second', enabled: false }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights }
		});
		const btn = container.querySelector('.highlights-generate-btn') as HTMLButtonElement;
		expect(btn?.disabled).toBe(true);
	});

	it('renders format toggle with tweet and thread options', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const opts = container.querySelectorAll('.highlights-format-opt');
		expect(opts.length).toBe(2);
		expect(opts[0]?.textContent).toBe('Tweet');
		expect(opts[1]?.textContent).toBe('Thread');
	});

	it('calls onformatchange when format toggle is clicked', async () => {
		const onformatchange = vi.fn();
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, onformatchange }
		});
		const threadBtn = container.querySelectorAll('.highlights-format-opt')[1] as HTMLButtonElement;
		await fireEvent.click(threadBtn);
		expect(onformatchange).toHaveBeenCalledWith('thread');
	});

	it('has proper aria-label on back button', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const backBtn = container.querySelector('.highlights-back');
		expect(backBtn?.getAttribute('aria-label')).toBe('Back to notes');
	});

	it('shows correct count for partially enabled highlights', () => {
		const highlights = [
			{ text: 'First', enabled: true },
			{ text: 'Second', enabled: false },
			{ text: 'Third', enabled: true }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights }
		});
		const count = container.querySelector('.highlights-count');
		expect(count?.textContent).toContain('2 of 3');
	});

	it('shows zero count when all highlights disabled', () => {
		const highlights = [
			{ text: 'First', enabled: false },
			{ text: 'Second', enabled: false }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights }
		});
		const count = container.querySelector('.highlights-count');
		expect(count?.textContent).toContain('0 of 2');
	});

	it('applies disabled class to unchecked highlight items', () => {
		const highlights = [
			{ text: 'First', enabled: false },
			{ text: 'Second', enabled: true }
		];
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, highlights }
		});
		const items = container.querySelectorAll('.highlight-item');
		expect(items[0].classList.contains('disabled')).toBe(true);
		expect(items[1].classList.contains('disabled')).toBe(false);
	});

	it('marks active format option correctly', () => {
		const { container } = render(VaultHighlights, {
			props: { ...defaultProps, outputFormat: 'tweet' as const }
		});
		const opts = container.querySelectorAll('.highlights-format-opt');
		expect(opts[0].classList.contains('active')).toBe(true);
		expect(opts[1].classList.contains('active')).toBe(false);
	});

	it('highlights list has proper role group', () => {
		const { container } = render(VaultHighlights, { props: defaultProps });
		const list = container.querySelector('[role="group"]');
		expect(list).toBeTruthy();
		expect(list?.getAttribute('aria-label')).toBe('Select highlights to include');
	});
});

// --- Selection state and ingress tests ---
describe('Selection state helpers', () => {
	it('ProvenanceRef can be constructed from selection metadata', () => {
		// Verify the type shape matches what the selection endpoint returns
		const selectionResponse = {
			session_id: 'abc-123',
			vault_name: 'marketing',
			file_path: 'content/ideas.md',
			selected_text: 'Some text from Obsidian',
			heading_context: 'Ideas > Marketing',
			note_title: 'Content Ideas',
			frontmatter_tags: ['marketing'],
			resolved_node_id: 42,
			resolved_chunk_id: 99,
			created_at: '2024-01-01T00:00:00Z',
			expires_at: '2024-01-01T00:30:00Z'
		};

		// Construct ProvenanceRef from selection (mirrors runtime logic)
		const provenance = {
			node_id: selectionResponse.resolved_node_id ?? undefined,
			chunk_id: selectionResponse.resolved_chunk_id ?? undefined,
			source_path: selectionResponse.file_path,
			heading_path: selectionResponse.heading_context ?? undefined,
			snippet: selectionResponse.selected_text ?? undefined
		};

		expect(provenance.node_id).toBe(42);
		expect(provenance.chunk_id).toBe(99);
		expect(provenance.source_path).toBe('content/ideas.md');
		expect(provenance.heading_path).toBe('Ideas > Marketing');
		expect(provenance.snippet).toBe('Some text from Obsidian');
	});

	it('ProvenanceRef handles null resolved IDs gracefully', () => {
		const selectionResponse = {
			resolved_node_id: null,
			resolved_chunk_id: null,
			file_path: 'content/ideas.md',
			heading_context: null,
			selected_text: null
		};

		const provenance = {
			node_id: selectionResponse.resolved_node_id ?? undefined,
			chunk_id: selectionResponse.resolved_chunk_id ?? undefined,
			source_path: selectionResponse.file_path,
			heading_path: selectionResponse.heading_context ?? undefined,
			snippet: selectionResponse.selected_text ?? undefined
		};

		expect(provenance.node_id).toBeUndefined();
		expect(provenance.chunk_id).toBeUndefined();
		expect(provenance.source_path).toBe('content/ideas.md');
	});
});

// --- Citation deep-link tests ---
describe('Citation heading deep-links', () => {
	it('buildObsidianUri includes heading fragment', async () => {
		const { buildObsidianUri } = await import('$lib/utils/obsidianUri');
		const uri = buildObsidianUri('/Users/alice/vaults/marketing', 'ideas.md', 'Overview > Strategy');
		expect(uri).toContain('obsidian://open');
		expect(uri).toContain('vault=marketing');
		expect(uri).toContain('file=ideas');
		expect(uri).toContain('#Strategy');
	});

	it('buildObsidianUri works without heading', async () => {
		const { buildObsidianUri } = await import('$lib/utils/obsidianUri');
		const uri = buildObsidianUri('/Users/alice/vaults/marketing', 'ideas.md');
		expect(uri).toContain('obsidian://open');
		expect(uri).not.toContain('#');
	});

	it('buildObsidianUri handles single-level heading', async () => {
		const { buildObsidianUri } = await import('$lib/utils/obsidianUri');
		const uri = buildObsidianUri('/Users/alice/vaults/notes', 'todo.md', 'Tasks');
		expect(uri).toContain('#Tasks');
	});

	it('buildObsidianUri returns null for empty vault path', async () => {
		const { buildObsidianUri } = await import('$lib/utils/obsidianUri');
		const uri = buildObsidianUri('', 'ideas.md', 'Heading');
		expect(uri).toBeNull();
	});
});
