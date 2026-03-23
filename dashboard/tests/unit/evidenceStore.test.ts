/**
 * evidenceStore.test.ts — Unit tests for evidence state utilities.
 */

import { describe, it, expect } from 'vitest';
import {
	createEvidenceState,
	pinEvidence,
	unpinEvidence,
	dismissEvidence,
	toggleAutoQuery,
	filterResults,
	isPinned,
	canPin,
	setLastManualQuery,
} from '$lib/stores/evidenceStore';
import type { EvidenceResult } from '$lib/api/types';

function makeResult(overrides: Partial<EvidenceResult> = {}): EvidenceResult {
	return {
		chunk_id: overrides.chunk_id ?? 1,
		node_id: overrides.node_id ?? 10,
		heading_path: overrides.heading_path ?? '# Test Heading',
		snippet: overrides.snippet ?? 'Test snippet text',
		match_reason: overrides.match_reason ?? 'semantic',
		score: overrides.score ?? 0.85,
		node_title: overrides.node_title ?? 'Test Note',
		relative_path: overrides.relative_path ?? 'notes/test.md',
	};
}

describe('evidenceStore', () => {
	describe('createEvidenceState', () => {
		it('returns empty defaults', () => {
			const state = createEvidenceState();
			expect(state.pinned).toEqual([]);
			expect(state.dismissedChunkIds.size).toBe(0);
			expect(state.autoQueryEnabled).toBe(false);
			expect(state.lastManualQuery).toBe('');
		});
	});

	describe('pinEvidence', () => {
		it('adds result to pinned array', () => {
			const state = createEvidenceState();
			const result = makeResult({ chunk_id: 1 });
			const next = pinEvidence(state, result);
			expect(next.pinned).toHaveLength(1);
			expect(next.pinned[0].chunk_id).toBe(1);
			expect(next.pinned[0].node_id).toBe(10);
			expect(next.pinned[0].heading_path).toBe('# Test Heading');
			expect(next.pinned[0].snippet).toBe('Test snippet text');
			expect(next.pinned[0].match_reason).toBe('semantic');
			expect(next.pinned[0].score).toBe(0.85);
			expect(next.pinned[0].node_title).toBe('Test Note');
			expect(next.pinned[0].relative_path).toBe('notes/test.md');
		});

		it('rejects when at max (5)', () => {
			let state = createEvidenceState();
			for (let i = 0; i < 5; i++) {
				state = pinEvidence(state, makeResult({ chunk_id: i + 1 }));
			}
			expect(state.pinned).toHaveLength(5);
			const overflow = pinEvidence(state, makeResult({ chunk_id: 99 }));
			expect(overflow.pinned).toHaveLength(5);
			expect(overflow).toBe(state);
		});

		it('rejects duplicate chunk_id', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 1 }));
			const dup = pinEvidence(state, makeResult({ chunk_id: 1 }));
			expect(dup.pinned).toHaveLength(1);
			expect(dup).toBe(state);
		});

		it('does not mutate original state', () => {
			const state = createEvidenceState();
			const next = pinEvidence(state, makeResult());
			expect(state.pinned).toHaveLength(0);
			expect(next.pinned).toHaveLength(1);
		});
	});

	describe('unpinEvidence', () => {
		it('removes from pinned by chunk_id', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 1 }));
			state = pinEvidence(state, makeResult({ chunk_id: 2 }));
			const next = unpinEvidence(state, 1);
			expect(next.pinned).toHaveLength(1);
			expect(next.pinned[0].chunk_id).toBe(2);
		});

		it('returns unchanged state if chunk_id not found', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 1 }));
			const next = unpinEvidence(state, 999);
			expect(next.pinned).toHaveLength(1);
		});
	});

	describe('dismissEvidence', () => {
		it('adds chunk_id to dismissed set', () => {
			const state = createEvidenceState();
			const next = dismissEvidence(state, 42);
			expect(next.dismissedChunkIds.has(42)).toBe(true);
			expect(next.dismissedChunkIds.size).toBe(1);
		});

		it('does not mutate original set', () => {
			const state = createEvidenceState();
			const next = dismissEvidence(state, 42);
			expect(state.dismissedChunkIds.size).toBe(0);
			expect(next.dismissedChunkIds.size).toBe(1);
		});
	});

	describe('toggleAutoQuery', () => {
		it('flips boolean', () => {
			const state = createEvidenceState();
			expect(state.autoQueryEnabled).toBe(false);
			const next = toggleAutoQuery(state);
			expect(next.autoQueryEnabled).toBe(true);
			const again = toggleAutoQuery(next);
			expect(again.autoQueryEnabled).toBe(false);
		});
	});

	describe('filterResults', () => {
		it('excludes dismissed chunk_ids', () => {
			let state = createEvidenceState();
			state = dismissEvidence(state, 2);
			const results = [makeResult({ chunk_id: 1 }), makeResult({ chunk_id: 2 }), makeResult({ chunk_id: 3 })];
			const filtered = filterResults(results, state, new Set());
			expect(filtered).toHaveLength(2);
			expect(filtered.map((r) => r.chunk_id)).toEqual([1, 3]);
		});

		it('excludes already-pinned chunk_ids', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 1 }));
			const results = [makeResult({ chunk_id: 1 }), makeResult({ chunk_id: 2 })];
			const filtered = filterResults(results, state, new Set());
			expect(filtered).toHaveLength(1);
			expect(filtered[0].chunk_id).toBe(2);
		});

		it('excludes graph neighbor chunk_ids', () => {
			const state = createEvidenceState();
			const results = [makeResult({ chunk_id: 1 }), makeResult({ chunk_id: 2 })];
			const graphIds = new Set([1]);
			const filtered = filterResults(results, state, graphIds);
			expect(filtered).toHaveLength(1);
			expect(filtered[0].chunk_id).toBe(2);
		});

		it('applies all filters together', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 1 }));
			state = dismissEvidence(state, 3);
			const results = [
				makeResult({ chunk_id: 1 }),
				makeResult({ chunk_id: 2 }),
				makeResult({ chunk_id: 3 }),
				makeResult({ chunk_id: 4 }),
			];
			const graphIds = new Set([4]);
			const filtered = filterResults(results, state, graphIds);
			expect(filtered).toHaveLength(1);
			expect(filtered[0].chunk_id).toBe(2);
		});
	});

	describe('isPinned', () => {
		it('returns true for pinned chunk_id', () => {
			let state = createEvidenceState();
			state = pinEvidence(state, makeResult({ chunk_id: 5 }));
			expect(isPinned(state, 5)).toBe(true);
		});

		it('returns false for non-pinned chunk_id', () => {
			const state = createEvidenceState();
			expect(isPinned(state, 5)).toBe(false);
		});
	});

	describe('canPin', () => {
		it('returns true when under limit', () => {
			const state = createEvidenceState();
			expect(canPin(state)).toBe(true);
		});

		it('returns false at limit', () => {
			let state = createEvidenceState();
			for (let i = 0; i < 5; i++) {
				state = pinEvidence(state, makeResult({ chunk_id: i + 1 }));
			}
			expect(canPin(state)).toBe(false);
		});
	});

	describe('setLastManualQuery', () => {
		it('updates lastManualQuery', () => {
			const state = createEvidenceState();
			const next = setLastManualQuery(state, 'async patterns');
			expect(next.lastManualQuery).toBe('async patterns');
		});
	});
});
