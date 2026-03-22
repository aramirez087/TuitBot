/**
 * GraphSuggestionCards.test.ts — Unit tests for GraphSuggestionCards.svelte
 *
 * Tests: loading shimmer, available state with cards, empty state, not-indexed state,
 * fallback state, accept/dismiss interactions, reason badges, snippet truncation.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import GraphSuggestionCards from '$lib/components/composer/GraphSuggestionCards.svelte';
import type { NeighborItem, GraphState } from '$lib/api/types';

const sampleNeighbors: NeighborItem[] = [
	{
		node_id: 55,
		node_title: 'Async Patterns',
		reason: 'linked_note',
		reason_label: 'linked note',
		intent: 'pro_tip',
		matched_tags: [],
		score: 3.5,
		snippet: 'Async patterns in Rust use tokio for runtime management.',
		best_chunk_id: 120,
		heading_path: '# Async > ## Tokio',
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
		snippet: 'Tokio provides a multi-threaded runtime for async Rust applications.',
		best_chunk_id: 145,
		heading_path: '# Runtime',
		relative_path: 'notes/tokio-runtime.md',
	},
];

const defaultProps = {
	neighbors: sampleNeighbors,
	graphState: 'available' as GraphState,
	loading: false,
	onaccept: vi.fn(),
	ondismiss: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('GraphSuggestionCards', () => {
	// --- Loading state ---

	it('renders shimmer cards when loading=true', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, loading: true },
		});
		const shimmers = container.querySelectorAll('.graph-shimmer-card');
		expect(shimmers.length).toBe(3);
	});

	it('shows "Finding related notes..." text during loading', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, loading: true },
		});
		expect(container.textContent).toContain('Finding related notes...');
	});

	it('does not render neighbor cards when loading', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, loading: true },
		});
		expect(container.querySelectorAll('.graph-card').length).toBe(0);
	});

	// --- Available state with neighbors ---

	it('renders neighbor cards when graphState=available', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const cards = container.querySelectorAll('.graph-card');
		expect(cards.length).toBe(2);
	});

	it('shows note titles in card headers', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const titles = container.querySelectorAll('.graph-card-title');
		expect(titles[0]?.textContent).toBe('Async Patterns');
		expect(titles[1]?.textContent).toBe('Tokio Runtime');
	});

	it('shows reason badge with correct reason_label', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const badges = container.querySelectorAll('.graph-reason-badge');
		expect(badges[0]?.textContent).toContain('linked note');
		expect(badges[1]?.textContent).toContain('shared tag: #async');
	});

	it('shows snippet text in cards', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const snippets = container.querySelectorAll('.graph-card-snippet');
		expect(snippets[0]?.textContent).toContain('Async patterns in Rust');
	});

	it('truncates long snippets with ellipsis', () => {
		const longSnippet = 'A'.repeat(200);
		const { container } = render(GraphSuggestionCards, {
			props: {
				...defaultProps,
				neighbors: [{ ...sampleNeighbors[0], snippet: longSnippet }],
			},
		});
		const snippet = container.querySelector('.graph-card-snippet');
		expect(snippet?.textContent?.length).toBeLessThanOrEqual(121); // 120 + ellipsis
		expect(snippet?.textContent).toContain('\u2026');
	});

	it('shows count badge with neighbor count', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const count = container.querySelector('.graph-suggestions-count');
		expect(count?.textContent).toBe('2');
	});

	it('shows "Related notes from your vault" header', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		expect(container.textContent).toContain('Related notes from your vault');
	});

	// --- Accept interaction ---

	it('fires onaccept with correct neighbor and role when action button clicked', async () => {
		const onaccept = vi.fn();
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, onaccept },
		});
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		await fireEvent.click(actionBtns[0]);
		expect(onaccept).toHaveBeenCalledWith(sampleNeighbors[0], 'pro_tip');
	});

	it('maps all intents to unified "Include" action button', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const actionBtns = container.querySelectorAll('.graph-action-btn');
		expect(actionBtns[0]?.textContent?.trim()).toBe('Include');
		expect(actionBtns[1]?.textContent?.trim()).toBe('Include');
	});

	// --- Dismiss interaction ---

	it('fires ondismiss with correct nodeId when dismiss clicked', async () => {
		const ondismiss = vi.fn();
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, ondismiss },
		});
		const dismissBtns = container.querySelectorAll('.graph-card-dismiss');
		await fireEvent.click(dismissBtns[0]);
		expect(ondismiss).toHaveBeenCalledWith(55);
	});

	// --- Empty state: no_related_notes ---

	it('shows empty message for no_related_notes', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, graphState: 'no_related_notes' as GraphState, neighbors: [] },
		});
		expect(container.textContent).toContain("doesn't link to other indexed notes");
	});

	// --- Not indexed state ---

	it('shows not-indexed message for node_not_indexed', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, graphState: 'node_not_indexed' as GraphState, neighbors: [] },
		});
		expect(container.textContent).toContain("hasn't been indexed yet");
	});

	// --- Fallback state ---

	it('renders nothing for fallback_active', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, graphState: 'fallback_active' as GraphState, neighbors: [] },
		});
		expect(container.querySelector('.graph-suggestions')).toBeFalsy();
		expect(container.textContent?.trim()).toBe('');
	});

	// --- Available but empty neighbors ---

	it('handles empty neighbors array with available state gracefully', () => {
		const { container } = render(GraphSuggestionCards, {
			props: { ...defaultProps, graphState: 'available' as GraphState, neighbors: [] },
		});
		expect(container.textContent).toContain("doesn't link to other indexed notes");
		expect(container.querySelectorAll('.graph-card').length).toBe(0);
	});

	// --- Accessibility ---

	it('has role=list on suggestions container', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const list = container.querySelector('[role="list"]');
		expect(list).toBeTruthy();
	});

	it('has role=listitem on each card', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const items = container.querySelectorAll('[role="listitem"]');
		expect(items.length).toBe(2);
	});

	it('dismiss buttons have aria-label', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const dismissBtns = container.querySelectorAll('.graph-card-dismiss');
		expect(dismissBtns[0]?.getAttribute('aria-label')).toBe('Skip Async Patterns');
	});

	it('reason badges have aria-label', () => {
		const { container } = render(GraphSuggestionCards, { props: defaultProps });
		const badges = container.querySelectorAll('.graph-reason-badge');
		expect(badges[0]?.getAttribute('aria-label')).toBe('Reason: linked note');
	});
});
