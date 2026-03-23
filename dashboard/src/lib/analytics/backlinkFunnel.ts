/**
 * Backlink synthesizer funnel events.
 *
 * Wraps `trackFunnel()` with typed helpers for each key moment
 * in the related-note suggestion → insertion → draft flow.
 * All events are prefixed with `backlink.` for namespace isolation.
 */

import { trackFunnel } from './funnel';
import type { FunnelEvent } from './funnel';

// ── Typed event helpers ──────────────────────────────────────

export function trackSuggestionsShown(count: number, sessionId: string, graphState: string): void {
	trackFunnel('backlink.suggestions_shown', { count, session_id: sessionId, graph_state: graphState });
}

export function trackSuggestionAccepted(nodeId: number, reason: string, intent: string, score: number, sessionId: string): void {
	trackFunnel('backlink.suggestion_accepted', { node_id: nodeId, reason, intent, score, session_id: sessionId });
}

export function trackSuggestionDismissed(nodeId: number, reason: string, sessionId: string): void {
	trackFunnel('backlink.suggestion_dismissed', { node_id: nodeId, reason, session_id: sessionId });
}

export function trackSuggestionRestored(nodeId: number, sessionId: string): void {
	trackFunnel('backlink.suggestion_restored', { node_id: nodeId, session_id: sessionId });
}

export function trackSynthesisToggled(enabled: boolean, sessionId: string): void {
	trackFunnel('backlink.synthesis_toggled', { enabled, session_id: sessionId });
}

export function trackSlotTargeted(slotLabel: string, sourceNodeId: number, sessionId: string): void {
	trackFunnel('backlink.slot_targeted', { slot_label: slotLabel, source_node_id: sourceNodeId, session_id: sessionId });
}

export function trackInsertUndone(insertId: string, slotLabel: string, sessionId: string): void {
	trackFunnel('backlink.insert_undone', { insert_id: insertId, slot_label: slotLabel, session_id: sessionId });
}

export function trackHooksGenerated(acceptedCount: number, totalNeighbors: number, hookCount: number, sessionId: string): void {
	trackFunnel('backlink.hooks_generated', { accepted_count: acceptedCount, total_neighbors: totalNeighbors, hook_count: hookCount, session_id: sessionId });
}

export function trackDraftCompleted(insertCount: number, acceptedCount: number, mode: string, sessionId: string): void {
	trackFunnel('backlink.draft_completed', { insert_count: insertCount, accepted_count: acceptedCount, mode, session_id: sessionId });
}

export function trackEmptyGraph(graphState: string, sessionId: string): void {
	trackFunnel('backlink.empty_graph', { graph_state: graphState, session_id: sessionId });
}

export function trackCitationClicked(sourceTitle: string, isGraphInsert: boolean, isDesktop: boolean): void {
	trackFunnel('backlink.citation_clicked', { source_title: sourceTitle, is_graph_insert: isGraphInsert, is_desktop: isDesktop });
}

export function trackAnglesMined(acceptedCount: number, angleCount: number, sessionId: string): void {
	trackFunnel('backlink.angles_mined', { accepted_count: acceptedCount, angle_count: angleCount, session_id: sessionId });
}

export function trackAngleFallback(reason: string, acceptedCount: number, sessionId: string): void {
	trackFunnel('backlink.angle_fallback', { reason, accepted_count: acceptedCount, session_id: sessionId });
}

// ── Backend relay (opt-in, not wired by default) ─────────────

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
