/**
 * Evidence rail funnel events.
 *
 * Wraps `trackFunnel()` with typed helpers for each key moment
 * in the semantic evidence search → pin → draft flow.
 * All events are prefixed with `evidence.` for namespace isolation.
 */

import { trackFunnel } from './funnel';

export function trackEvidenceRailOpened(sessionId: string, hasSelection: boolean): void {
	trackFunnel('evidence.rail_opened', { session_id: sessionId, has_selection: hasSelection });
}

export function trackEvidenceSearchExecuted(queryLength: number, resultCount: number, mode: string): void {
	trackFunnel('evidence.search_executed', { query_length: queryLength, result_count: resultCount, mode });
}

export function trackEvidencePinned(chunkId: number, matchReason: string, score: number): void {
	trackFunnel('evidence.pinned', { chunk_id: chunkId, match_reason: matchReason, score });
}

export function trackEvidenceDismissed(chunkId: number, matchReason: string): void {
	trackFunnel('evidence.dismissed', { chunk_id: chunkId, match_reason: matchReason });
}

export function trackEvidenceAppliedToSlot(chunkId: number, slotIndex: number, slotLabel: string, matchReason: string): void {
	trackFunnel('evidence.applied_to_slot', { chunk_id: chunkId, slot_index: slotIndex, slot_label: slotLabel, match_reason: matchReason });
}

export function trackAutoQueryToggled(enabled: boolean): void {
	trackFunnel('evidence.auto_query_toggled', { enabled });
}

export function trackEvidenceContributedToDraft(pinnedCount: number, appliedCount: number, sessionId: string): void {
	trackFunnel('evidence.contributed_to_draft', { pinned_count: pinnedCount, applied_count: appliedCount, session_id: sessionId });
}
