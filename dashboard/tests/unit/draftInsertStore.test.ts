/**
 * draftInsertStore.test.ts — Unit tests for draft insert history utilities.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	createInsertState,
	pushInsert,
	popInsert,
	undoInsertById,
	getInsertsForBlock,
	getSlotLabel,
	clearInserts,
	hasInserts,
	buildInsert,
} from '$lib/stores/draftInsertStore';
import type { DraftInsert } from '$lib/api/types';

function makeInsert(overrides: Partial<DraftInsert> = {}): DraftInsert {
	return {
		id: overrides.id ?? crypto.randomUUID(),
		blockId: overrides.blockId ?? 'block-1',
		slotLabel: overrides.slotLabel ?? 'Tweet 2',
		previousText: overrides.previousText ?? 'old text',
		insertedText: overrides.insertedText ?? 'new text',
		sourceNodeId: overrides.sourceNodeId ?? 55,
		sourceTitle: overrides.sourceTitle ?? 'Test Note',
		provenance: overrides.provenance ?? { node_id: 55, edge_type: 'linked_note', edge_label: 'linked note' },
		timestamp: overrides.timestamp ?? Date.now(),
	};
}

describe('draftInsertStore', () => {
	describe('createInsertState', () => {
		it('returns empty state', () => {
			const state = createInsertState();
			expect(state.history).toEqual([]);
			expect(state.blockInserts.size).toBe(0);
		});
	});

	describe('pushInsert', () => {
		it('adds to history and blockInserts', () => {
			const state = createInsertState();
			const insert = makeInsert({ blockId: 'b1' });
			const next = pushInsert(state, insert);
			expect(next.history).toHaveLength(1);
			expect(next.history[0]).toBe(insert);
			expect(next.blockInserts.get('b1')).toHaveLength(1);
		});

		it('multiple inserts to same block tracked correctly', () => {
			let state = createInsertState();
			const i1 = makeInsert({ blockId: 'b1', id: 'i1' });
			const i2 = makeInsert({ blockId: 'b1', id: 'i2' });
			state = pushInsert(state, i1);
			state = pushInsert(state, i2);
			expect(state.history).toHaveLength(2);
			expect(state.blockInserts.get('b1')).toHaveLength(2);
		});

		it('does not mutate original state', () => {
			const state = createInsertState();
			const insert = makeInsert();
			const next = pushInsert(state, insert);
			expect(state.history).toHaveLength(0);
			expect(next.history).toHaveLength(1);
		});
	});

	describe('popInsert', () => {
		it('removes most recent insert and returns it', () => {
			let state = createInsertState();
			const i1 = makeInsert({ id: 'i1', blockId: 'b1' });
			const i2 = makeInsert({ id: 'i2', blockId: 'b2' });
			state = pushInsert(state, i1);
			state = pushInsert(state, i2);
			const result = popInsert(state);
			expect(result).not.toBeNull();
			expect(result!.undone.id).toBe('i2');
			expect(result!.newState.history).toHaveLength(1);
			expect(result!.newState.blockInserts.has('b2')).toBe(false);
		});

		it('returns null on empty state', () => {
			const state = createInsertState();
			expect(popInsert(state)).toBeNull();
		});

		it('cleans up blockInserts when last insert for block is removed', () => {
			let state = createInsertState();
			const insert = makeInsert({ blockId: 'b1' });
			state = pushInsert(state, insert);
			const result = popInsert(state);
			expect(result!.newState.blockInserts.has('b1')).toBe(false);
		});
	});

	describe('undoInsertById', () => {
		it('removes specific insert from middle of history', () => {
			let state = createInsertState();
			const i1 = makeInsert({ id: 'i1', blockId: 'b1' });
			const i2 = makeInsert({ id: 'i2', blockId: 'b1' });
			const i3 = makeInsert({ id: 'i3', blockId: 'b2' });
			state = pushInsert(state, i1);
			state = pushInsert(state, i2);
			state = pushInsert(state, i3);
			const result = undoInsertById(state, 'i2');
			expect(result).not.toBeNull();
			expect(result!.undone.id).toBe('i2');
			expect(result!.newState.history).toHaveLength(2);
			expect(result!.newState.blockInserts.get('b1')).toHaveLength(1);
		});

		it('returns null for non-existent ID', () => {
			const state = createInsertState();
			expect(undoInsertById(state, 'nonexistent')).toBeNull();
		});
	});

	describe('getInsertsForBlock', () => {
		it('returns inserts for a given block', () => {
			let state = createInsertState();
			const i1 = makeInsert({ blockId: 'b1', id: 'i1' });
			const i2 = makeInsert({ blockId: 'b2', id: 'i2' });
			state = pushInsert(state, i1);
			state = pushInsert(state, i2);
			expect(getInsertsForBlock(state, 'b1')).toHaveLength(1);
			expect(getInsertsForBlock(state, 'b1')[0].id).toBe('i1');
		});

		it('returns empty array for unknown block', () => {
			const state = createInsertState();
			expect(getInsertsForBlock(state, 'unknown')).toEqual([]);
		});
	});

	describe('getSlotLabel', () => {
		it('returns "Tweet" for single tweet mode', () => {
			expect(getSlotLabel(0, 1)).toBe('Tweet');
		});

		it('returns "Opening hook" for first block in thread', () => {
			expect(getSlotLabel(0, 5)).toBe('Opening hook');
		});

		it('returns "Tweet N" for middle blocks', () => {
			expect(getSlotLabel(1, 5)).toBe('Tweet 2');
			expect(getSlotLabel(3, 5)).toBe('Tweet 4');
		});

		it('returns "Closing takeaway" for last block', () => {
			expect(getSlotLabel(4, 5)).toBe('Closing takeaway');
		});

		it('handles two-block thread', () => {
			expect(getSlotLabel(0, 2)).toBe('Opening hook');
			expect(getSlotLabel(1, 2)).toBe('Closing takeaway');
		});
	});

	describe('clearInserts', () => {
		it('returns empty state', () => {
			const state = clearInserts();
			expect(state.history).toEqual([]);
			expect(state.blockInserts.size).toBe(0);
		});
	});

	describe('hasInserts', () => {
		it('returns false for empty state', () => {
			expect(hasInserts(createInsertState())).toBe(false);
		});

		it('returns true when inserts exist', () => {
			let state = createInsertState();
			state = pushInsert(state, makeInsert());
			expect(hasInserts(state)).toBe(true);
		});
	});

	describe('buildInsert', () => {
		it('creates insert with provenance including edge_type and edge_label', () => {
			const insert = buildInsert({
				blockId: 'b1',
				slotLabel: 'Opening hook',
				previousText: 'old',
				insertedText: 'new',
				sourceNodeId: 55,
				sourceTitle: 'Async Patterns',
				edgeType: 'linked_note',
				edgeLabel: 'linked note',
			});
			expect(insert.id).toBeTruthy();
			expect(insert.blockId).toBe('b1');
			expect(insert.provenance.node_id).toBe(55);
			expect(insert.provenance.edge_type).toBe('linked_note');
			expect(insert.provenance.edge_label).toBe('linked note');
			expect(insert.timestamp).toBeGreaterThan(0);
		});
	});

	describe('undo restores previousText correctly', () => {
		it('popInsert returns the correct previousText for restoration', () => {
			let state = createInsertState();
			const insert = makeInsert({ previousText: 'original draft text', insertedText: 'refined text' });
			state = pushInsert(state, insert);
			const result = popInsert(state);
			expect(result!.undone.previousText).toBe('original draft text');
		});
	});
});
