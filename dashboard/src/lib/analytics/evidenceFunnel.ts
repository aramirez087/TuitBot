/**
 * Evidence rail funnel events.
 *
 * Wraps `trackFunnel()` with typed helpers for each key moment
 * in the semantic evidence search → pin → draft flow.
 * All events are prefixed with `evidence.` for namespace isolation.
 */

import { trackFunnel } from './funnel';
import type { FunnelEvent } from './funnel';

export function trackEvidenceRailOpened(sessionId: string, hasSelection: boolean): void {
	trackFunnel('evidence.rail_opened', { session_id: sessionId, has_selection: hasSelection });
	bufferEvent({ event: 'evidence.rail_opened', properties: { session_id: sessionId, has_selection: hasSelection }, timestamp: new Date().toISOString() });
}

export function trackEvidenceSearchExecuted(queryLength: number, resultCount: number, mode: string): void {
	trackFunnel('evidence.search_executed', { query_length: queryLength, result_count: resultCount, mode });
	bufferEvent({ event: 'evidence.search_executed', properties: { query_length: queryLength, result_count: resultCount, mode }, timestamp: new Date().toISOString() });
}

export function trackEvidencePinned(chunkId: number, matchReason: string, score: number): void {
	trackFunnel('evidence.pinned', { chunk_id: chunkId, match_reason: matchReason, score });
	bufferEvent({ event: 'evidence.pinned', properties: { chunk_id: chunkId, match_reason: matchReason, score }, timestamp: new Date().toISOString() });
}

export function trackEvidenceDismissed(chunkId: number, matchReason: string): void {
	trackFunnel('evidence.dismissed', { chunk_id: chunkId, match_reason: matchReason });
	bufferEvent({ event: 'evidence.dismissed', properties: { chunk_id: chunkId, match_reason: matchReason }, timestamp: new Date().toISOString() });
}

export function trackEvidenceAppliedToSlot(chunkId: number, slotIndex: number, slotLabel: string, matchReason: string): void {
	trackFunnel('evidence.applied_to_slot', { chunk_id: chunkId, slot_index: slotIndex, slot_label: slotLabel, match_reason: matchReason });
	bufferEvent({ event: 'evidence.applied_to_slot', properties: { chunk_id: chunkId, slot_index: slotIndex, slot_label: slotLabel, match_reason: matchReason }, timestamp: new Date().toISOString() });
}

export function trackAutoQueryToggled(enabled: boolean): void {
	trackFunnel('evidence.auto_query_toggled', { enabled });
	bufferEvent({ event: 'evidence.auto_query_toggled', properties: { enabled }, timestamp: new Date().toISOString() });
}

export function trackEvidenceContributedToDraft(pinnedCount: number, appliedCount: number, sessionId: string): void {
	trackFunnel('evidence.contributed_to_draft', { pinned_count: pinnedCount, applied_count: appliedCount, session_id: sessionId });
	bufferEvent({ event: 'evidence.contributed_to_draft', properties: { pinned_count: pinnedCount, applied_count: appliedCount, session_id: sessionId }, timestamp: new Date().toISOString() });
}

export function trackEvidenceSearchLatency(latencyMs: number, mode: string, fallback: boolean): void {
	trackFunnel('evidence.search_latency', { latency_ms: latencyMs, mode, fallback });
	bufferEvent({ event: 'evidence.search_latency', properties: { latency_ms: latencyMs, mode, fallback }, timestamp: new Date().toISOString() });
}

export function trackEvidenceDraftMode(mode: 'tweet' | 'thread', pinnedCount: number, appliedCount: number): void {
	trackFunnel('evidence.draft_mode', { mode, pinned_count: pinnedCount, applied_count: appliedCount });
	bufferEvent({ event: 'evidence.draft_mode', properties: { mode, pinned_count: pinnedCount, applied_count: appliedCount }, timestamp: new Date().toISOString() });
}

export function trackEvidenceStrengthen(blockCount: number, pinnedCount: number): void {
	trackFunnel('evidence.strengthen_draft', { block_count: blockCount, pinned_count: pinnedCount });
	bufferEvent({ event: 'evidence.strengthen_draft', properties: { block_count: blockCount, pinned_count: pinnedCount }, timestamp: new Date().toISOString() });
}

// ── Backend relay ─────────────────────────────────────────────

let eventBuffer: FunnelEvent[] = [];

export function bufferEvent(event: FunnelEvent): void {
	eventBuffer.push(event);
	if (eventBuffer.length >= 10) {
		flushToBackend();
	}
}

export async function flushToBackend(): Promise<void> {
	if (eventBuffer.length === 0) return;
	const batch = eventBuffer.splice(0);
	try {
		await fetch('/api/telemetry/events', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ events: batch }),
		});
	} catch {
		// Fail silently — telemetry is best-effort
		eventBuffer.unshift(...batch);
	}
}
