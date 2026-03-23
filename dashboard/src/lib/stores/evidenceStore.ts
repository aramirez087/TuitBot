/**
 * Evidence state utilities.
 *
 * Pure functions for managing semantic evidence search state inside the
 * Ghostwriter composer. Instantiated as component state — not a global store.
 */

import type { EvidenceResult, MatchReason } from '$lib/api/types';

export interface PinnedEvidence {
	chunk_id: number;
	node_id: number;
	heading_path: string;
	snippet: string;
	match_reason: MatchReason;
	score: number;
	node_title: string | null;
	relative_path?: string;
}

export interface EvidenceState {
	pinned: PinnedEvidence[];
	dismissedChunkIds: Set<number>;
	autoQueryEnabled: boolean;
	lastManualQuery: string;
}

const MAX_PINNED = 5;

export function createEvidenceState(): EvidenceState {
	return {
		pinned: [],
		dismissedChunkIds: new Set(),
		autoQueryEnabled: false,
		lastManualQuery: '',
	};
}

export function pinEvidence(state: EvidenceState, result: EvidenceResult): EvidenceState {
	if (state.pinned.length >= MAX_PINNED) return state;
	if (state.pinned.some((p) => p.chunk_id === result.chunk_id)) return state;
	const pinned: PinnedEvidence = {
		chunk_id: result.chunk_id,
		node_id: result.node_id,
		heading_path: result.heading_path,
		snippet: result.snippet,
		match_reason: result.match_reason,
		score: result.score,
		node_title: result.node_title,
		relative_path: result.relative_path,
	};
	return { ...state, pinned: [...state.pinned, pinned] };
}

export function unpinEvidence(state: EvidenceState, chunkId: number): EvidenceState {
	return { ...state, pinned: state.pinned.filter((p) => p.chunk_id !== chunkId) };
}

export function dismissEvidence(state: EvidenceState, chunkId: number): EvidenceState {
	const dismissedChunkIds = new Set(state.dismissedChunkIds);
	dismissedChunkIds.add(chunkId);
	return { ...state, dismissedChunkIds };
}

export function toggleAutoQuery(state: EvidenceState): EvidenceState {
	return { ...state, autoQueryEnabled: !state.autoQueryEnabled };
}

export function filterResults(
	results: EvidenceResult[],
	state: EvidenceState,
	graphChunkIds: Set<number>
): EvidenceResult[] {
	const pinnedIds = new Set(state.pinned.map((p) => p.chunk_id));
	return results.filter(
		(r) =>
			!state.dismissedChunkIds.has(r.chunk_id) &&
			!pinnedIds.has(r.chunk_id) &&
			!graphChunkIds.has(r.chunk_id)
	);
}

export function isPinned(state: EvidenceState, chunkId: number): boolean {
	return state.pinned.some((p) => p.chunk_id === chunkId);
}

export function canPin(state: EvidenceState): boolean {
	return state.pinned.length < MAX_PINNED;
}

export function setLastManualQuery(state: EvidenceState, query: string): EvidenceState {
	return { ...state, lastManualQuery: query };
}
