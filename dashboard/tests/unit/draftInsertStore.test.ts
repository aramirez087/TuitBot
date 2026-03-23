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
	partitionInserts,
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

	describe('buildInsert with evidence metadata', () => {
		it('populates provenance with match_reason, similarity_score, chunk_id, and source_role', () => {
			const insert = buildInsert({
				blockId: 'b1',
				slotLabel: 'Tweet',
				previousText: 'old',
				insertedText: 'new',
				sourceNodeId: 10,
				sourceTitle: 'Research Note',
				matchReason: 'semantic',
				similarityScore: 0.92,
				chunkId: 42,
				sourceRole: 'semantic_evidence',
				headingPath: 'Overview > Key findings',
				snippet: 'Important findings here',
			});
			expect(insert.provenance.match_reason).toBe('semantic');
			expect(insert.provenance.similarity_score).toBe(0.92);
			expect(insert.provenance.chunk_id).toBe(42);
			expect(insert.provenance.source_role).toBe('semantic_evidence');
			expect(insert.provenance.heading_path).toBe('Overview > Key findings');
			expect(insert.provenance.snippet).toBe('Important findings here');
			expect(insert.provenance.node_id).toBe(10);
		});

		it('leaves optional evidence fields undefined when not provided', () => {
			const insert = buildInsert({
				blockId: 'b1',
				slotLabel: 'Tweet',
				previousText: 'old',
				insertedText: 'new',
				sourceNodeId: 10,
				sourceTitle: 'Note',
			});
			expect(insert.provenance.match_reason).toBeUndefined();
			expect(insert.provenance.similarity_score).toBeUndefined();
			expect(insert.provenance.chunk_id).toBeUndefined();
			expect(insert.provenance.source_role).toBeUndefined();
		});
	});

	describe('partitionInserts', () => {
		it('separates graph and evidence inserts', () => {
			let state = createInsertState();
			const graphInsert = makeInsert({
				id: 'g1',
				blockId: 'b1',
				provenance: { node_id: 1, edge_type: 'linked_note', source_role: 'accepted_neighbor' },
			});
			const evidenceInsert = makeInsert({
				id: 'e1',
				blockId: 'b1',
				provenance: { node_id: 2, source_role: 'semantic_evidence', match_reason: 'semantic' },
			});
			state = pushInsert(state, graphInsert);
			state = pushInsert(state, evidenceInsert);
			const { graphInserts, evidenceInserts } = partitionInserts(state);
			expect(graphInserts).toHaveLength(1);
			expect(graphInserts[0].id).toBe('g1');
			expect(evidenceInserts).toHaveLength(1);
			expect(evidenceInserts[0].id).toBe('e1');
		});

		it('returns empty arrays for empty state', () => {
			const state = createInsertState();
			const { graphInserts, evidenceInserts } = partitionInserts(state);
			expect(graphInserts).toHaveLength(0);
			expect(evidenceInserts).toHaveLength(0);
		});

		it('undo of evidence insert removes from evidence partition', () => {
			let state = createInsertState();
			const evidenceInsert = makeInsert({
				id: 'e1',
				blockId: 'b1',
				provenance: { node_id: 2, source_role: 'semantic_evidence' },
			});
			state = pushInsert(state, evidenceInsert);
			const result = undoInsertById(state, 'e1');
			expect(result).not.toBeNull();
			const { evidenceInserts } = partitionInserts(result!.newState);
			expect(evidenceInserts).toHaveLength(0);
		});

		it('multiple evidence inserts to same block are individually undoable', () => {
			let state = createInsertState();
			const e1 = makeInsert({
				id: 'e1',
				blockId: 'b1',
				provenance: { node_id: 1, source_role: 'semantic_evidence' },
			});
			const e2 = makeInsert({
				id: 'e2',
				blockId: 'b1',
				provenance: { node_id: 2, source_role: 'semantic_evidence' },
			});
			state = pushInsert(state, e1);
			state = pushInsert(state, e2);
			const result = undoInsertById(state, 'e1');
			expect(result).not.toBeNull();
			const { evidenceInserts } = partitionInserts(result!.newState);
			expect(evidenceInserts).toHaveLength(1);
			expect(evidenceInserts[0].id).toBe('e2');
		});
	});
});
