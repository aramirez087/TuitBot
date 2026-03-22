/**
 * SlotTargetPanel.test.ts — Unit tests for SlotTargetPanel.svelte
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import SlotTargetPanel from '$lib/components/composer/SlotTargetPanel.svelte';
import type { NeighborItem, DraftInsertState, DraftInsert } from '$lib/api/types';
import { createInsertState, pushInsert } from '$lib/stores/draftInsertStore';

const sampleNeighbor: NeighborItem = {
	node_id: 55,
	node_title: 'Async Patterns',
	reason: 'linked_note',
	reason_label: 'linked note',
	intent: 'pro_tip',
	matched_tags: [],
	score: 3.5,
	snippet: 'Async patterns in Rust use tokio for efficient runtime management.',
	best_chunk_id: 120,
	heading_path: '# Async',
	relative_path: 'notes/async-patterns.md',
};

const sampleNeighbor2: NeighborItem = {
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
};

const sampleBlocks = [
	{ id: 'b1', text: 'Opening hook text here', media_paths: [], order: 0 },
	{ id: 'b2', text: 'Middle content here', media_paths: [], order: 1 },
	{ id: 'b3', text: 'Closing takeaway text', media_paths: [], order: 2 },
];

function makeAccepted(...neighbors: NeighborItem[]): Map<number, { neighbor: NeighborItem; role: string }> {
	const map = new Map<number, { neighbor: NeighborItem; role: string }>();
	for (const n of neighbors) {
		map.set(n.node_id, { neighbor: n, role: n.intent });
	}
	return map;
}

function makeInsert(overrides: Partial<DraftInsert> = {}): DraftInsert {
	return {
		id: overrides.id ?? 'ins-1',
		blockId: overrides.blockId ?? 'b1',
		slotLabel: overrides.slotLabel ?? 'Opening hook',
		previousText: overrides.previousText ?? 'old',
		insertedText: overrides.insertedText ?? 'new',
		sourceNodeId: overrides.sourceNodeId ?? 55,
		sourceTitle: overrides.sourceTitle ?? 'Async Patterns',
		provenance: { node_id: 55, edge_type: 'linked_note', edge_label: 'linked note' },
		timestamp: Date.now(),
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('SlotTargetPanel', () => {
	it('renders slot labels for thread blocks', () => {
		const { getByText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: createInsertState(),
			},
		});
		expect(getByText('Refine specific parts')).toBeTruthy();
		expect(getByText('Async Patterns')).toBeTruthy();
	});

	it('shows accepted neighbors with Apply buttons', () => {
		const { getAllByText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor, sampleNeighbor2),
				insertState: createInsertState(),
			},
		});
		const applyBtns = getAllByText('Refine');
		expect(applyBtns.length).toBe(2);
	});

	it('slot selector shows correct labels', () => {
		const { getAllByRole } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: createInsertState(),
			},
		});
		const selects = getAllByRole('combobox') as HTMLSelectElement[];
		expect(selects.length).toBeGreaterThan(0);
		const options = selects[0].querySelectorAll('option');
		expect(options[0].textContent).toBe('Opening hook');
		expect(options[1].textContent).toBe('Tweet 2');
		expect(options[2].textContent).toBe('Closing takeaway');
	});

	it('clicking Apply fires oninsert with correct args', async () => {
		const oninsert = vi.fn();
		const { getByText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: createInsertState(),
				oninsert,
			},
		});
		await fireEvent.click(getByText('Refine'));
		expect(oninsert).toHaveBeenCalledWith(sampleNeighbor, 0, 'Opening hook');
	});

	it('shows applied badge for slots with active inserts', () => {
		let state = createInsertState();
		state = pushInsert(state, makeInsert({ sourceNodeId: 55 }));
		const { getByText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: state,
			},
		});
		expect(getByText('Applied refinements')).toBeTruthy();
		expect(getByText('Async Patterns')).toBeTruthy();
	});

	it('undo button on applied insert fires onundoinsert', async () => {
		const onundoinsert = vi.fn();
		let state = createInsertState();
		state = pushInsert(state, makeInsert({ id: 'ins-1', sourceNodeId: 55 }));
		const { getByTitle } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: state,
				onundoinsert,
			},
		});
		await fireEvent.click(getByTitle('Undo'));
		expect(onundoinsert).toHaveBeenCalledWith('ins-1');
	});

	it('empty state: no accepted neighbors shows helpful message', () => {
		const { getByText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: new Map(),
				insertState: createInsertState(),
			},
		});
		expect(getByText(/Include related notes/)).toBeTruthy();
	});

	it('tweet mode: single slot, no dropdown needed', () => {
		const { queryAllByRole, getByText } = render(SlotTargetPanel, {
			props: {
				mode: 'tweet',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: createInsertState(),
			},
		});
		expect(getByText('Refine')).toBeTruthy();
		// Only 1 slot, no select needed
		const selects = queryAllByRole('combobox');
		expect(selects.length).toBe(0);
	});

	it('accessibility: aria-labels on apply buttons', () => {
		const { getByLabelText } = render(SlotTargetPanel, {
			props: {
				threadBlocks: sampleBlocks,
				mode: 'thread',
				acceptedNeighbors: makeAccepted(sampleNeighbor),
				insertState: createInsertState(),
			},
		});
		expect(getByLabelText('Refine slot with Async Patterns')).toBeTruthy();
	});
});
