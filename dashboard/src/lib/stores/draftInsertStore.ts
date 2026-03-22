/**
 * Draft insert history utilities.
 *
 * Pure functions for managing per-slot suggestion insertions with undo support.
 * Instantiated as component state — not a global Svelte store.
 */

import type { DraftInsert, DraftInsertState, ProvenanceRef } from '$lib/api/types';

/** Create an empty insert state. */
export function createInsertState(): DraftInsertState {
	return { history: [], blockInserts: new Map() };
}

/** Push a new insert onto the history and block map. Returns new state. */
export function pushInsert(state: DraftInsertState, insert: DraftInsert): DraftInsertState {
	const history = [...state.history, insert];
	const blockInserts = new Map(state.blockInserts);
	const existing = blockInserts.get(insert.blockId) ?? [];
	blockInserts.set(insert.blockId, [...existing, insert]);
	return { history, blockInserts };
}

/** Pop the most recent insert. Returns { newState, undone } or null if empty. */
export function popInsert(
	state: DraftInsertState
): { newState: DraftInsertState; undone: DraftInsert } | null {
	if (state.history.length === 0) return null;
	const undone = state.history[state.history.length - 1];
	const history = state.history.slice(0, -1);
	const blockInserts = new Map(state.blockInserts);
	const blockList = (blockInserts.get(undone.blockId) ?? []).filter((i) => i.id !== undone.id);
	if (blockList.length === 0) {
		blockInserts.delete(undone.blockId);
	} else {
		blockInserts.set(undone.blockId, blockList);
	}
	return { newState: { history, blockInserts }, undone };
}

/** Remove a specific insert by ID. Returns { newState, undone } or null if not found. */
export function undoInsertById(
	state: DraftInsertState,
	insertId: string
): { newState: DraftInsertState; undone: DraftInsert } | null {
	const idx = state.history.findIndex((i) => i.id === insertId);
	if (idx === -1) return null;
	const undone = state.history[idx];
	const history = [...state.history.slice(0, idx), ...state.history.slice(idx + 1)];
	const blockInserts = new Map(state.blockInserts);
	const blockList = (blockInserts.get(undone.blockId) ?? []).filter((i) => i.id !== undone.id);
	if (blockList.length === 0) {
		blockInserts.delete(undone.blockId);
	} else {
		blockInserts.set(undone.blockId, blockList);
	}
	return { newState: { history, blockInserts }, undone };
}

/** Get all active inserts for a given block. */
export function getInsertsForBlock(state: DraftInsertState, blockId: string): DraftInsert[] {
	return state.blockInserts.get(blockId) ?? [];
}

/** Derive a human-readable slot label from position and total count. */
export function getSlotLabel(index: number, total: number): string {
	if (total === 1) return 'Tweet';
	if (index === 0) return 'Opening hook';
	if (index === total - 1) return 'Closing takeaway';
	return `Tweet ${index + 1}`;
}

/** Reset to empty state. */
export function clearInserts(): DraftInsertState {
	return createInsertState();
}

/** Check if state has any inserts. */
export function hasInserts(state: DraftInsertState): boolean {
	return state.history.length > 0;
}

/** Build a DraftInsert from acceptance parameters. */
export function buildInsert(params: {
	blockId: string;
	slotLabel: string;
	previousText: string;
	insertedText: string;
	sourceNodeId: number;
	sourceTitle: string;
	edgeType?: string;
	edgeLabel?: string;
}): DraftInsert {
	const provenance: ProvenanceRef = {
		node_id: params.sourceNodeId,
		edge_type: params.edgeType,
		edge_label: params.edgeLabel,
	};
	return {
		id: crypto.randomUUID(),
		blockId: params.blockId,
		slotLabel: params.slotLabel,
		previousText: params.previousText,
		insertedText: params.insertedText,
		sourceNodeId: params.sourceNodeId,
		sourceTitle: params.sourceTitle,
		provenance,
		timestamp: Date.now(),
	};
}
