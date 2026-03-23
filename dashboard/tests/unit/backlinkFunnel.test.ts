/**
 * backlinkFunnel.test.ts — Unit tests for backlink funnel analytics helpers.
 *
 * Verifies that each typed event helper calls trackFunnel with the correct
 * event name and property shape, and that the backend relay buffers/flushes.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('$lib/analytics/funnel', () => ({
	trackFunnel: vi.fn(),
}));

import { trackFunnel } from '$lib/analytics/funnel';
import {
	trackSuggestionsShown,
	trackSuggestionAccepted,
	trackSuggestionDismissed,
	trackSuggestionRestored,
	trackSynthesisToggled,
	trackSlotTargeted,
	trackInsertUndone,
	trackHooksGenerated,
	trackDraftCompleted,
	trackEmptyGraph,
	trackCitationClicked,
	trackAnglesMined,
	trackAngleFallback,
	bufferEvent,
	flushToBackend,
} from '$lib/analytics/backlinkFunnel';

beforeEach(() => {
	vi.clearAllMocks();
});

describe('backlink funnel event helpers', () => {
	it('trackSuggestionsShown calls trackFunnel with correct event and props', () => {
		trackSuggestionsShown(3, 'sess-1', 'available');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.suggestions_shown', {
			count: 3,
			session_id: 'sess-1',
			graph_state: 'available',
		});
	});

	it('trackSuggestionAccepted calls trackFunnel with correct event and props', () => {
		trackSuggestionAccepted(55, 'linked_note', 'pro_tip', 3.5, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.suggestion_accepted', {
			node_id: 55,
			reason: 'linked_note',
			intent: 'pro_tip',
			score: 3.5,
			session_id: 'sess-1',
		});
	});

	it('trackSuggestionDismissed calls trackFunnel with correct event and props', () => {
		trackSuggestionDismissed(78, 'shared_tag', 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.suggestion_dismissed', {
			node_id: 78,
			reason: 'shared_tag',
			session_id: 'sess-1',
		});
	});

	it('trackSuggestionRestored calls trackFunnel with correct event and props', () => {
		trackSuggestionRestored(55, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.suggestion_restored', {
			node_id: 55,
			session_id: 'sess-1',
		});
	});

	it('trackSynthesisToggled calls trackFunnel with correct event and props', () => {
		trackSynthesisToggled(false, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.synthesis_toggled', {
			enabled: false,
			session_id: 'sess-1',
		});
	});

	it('trackSlotTargeted calls trackFunnel with correct event and props', () => {
		trackSlotTargeted('Opening hook', 55, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.slot_targeted', {
			slot_label: 'Opening hook',
			source_node_id: 55,
			session_id: 'sess-1',
		});
	});

	it('trackInsertUndone calls trackFunnel with correct event and props', () => {
		trackInsertUndone('ins-abc', 'Opening hook', 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.insert_undone', {
			insert_id: 'ins-abc',
			slot_label: 'Opening hook',
			session_id: 'sess-1',
		});
	});

	it('trackHooksGenerated calls trackFunnel with correct event and props', () => {
		trackHooksGenerated(2, 5, 3, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.hooks_generated', {
			accepted_count: 2,
			total_neighbors: 5,
			hook_count: 3,
			session_id: 'sess-1',
		});
	});

	it('trackDraftCompleted calls trackFunnel with correct event and props', () => {
		trackDraftCompleted(1, 2, 'tweet', 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.draft_completed', {
			insert_count: 1,
			accepted_count: 2,
			mode: 'tweet',
			session_id: 'sess-1',
		});
	});

	it('trackEmptyGraph calls trackFunnel with correct event and props', () => {
		trackEmptyGraph('no_related_notes', 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.empty_graph', {
			graph_state: 'no_related_notes',
			session_id: 'sess-1',
		});
	});

	it('trackCitationClicked calls trackFunnel with correct event and props', () => {
		trackCitationClicked('Async Patterns', true, false);
		expect(trackFunnel).toHaveBeenCalledWith('backlink.citation_clicked', {
			source_title: 'Async Patterns',
			is_graph_insert: true,
			is_desktop: false,
		});
	});

	it('trackAnglesMined calls trackFunnel with correct event and props', () => {
		trackAnglesMined(3, 5, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.angles_mined', {
			accepted_count: 3,
			angle_count: 5,
			session_id: 'sess-1',
		});
	});

	it('trackAngleFallback calls trackFunnel with correct event and props', () => {
		trackAngleFallback('weak_signal', 2, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('backlink.angle_fallback', {
			reason: 'weak_signal',
			accepted_count: 2,
			session_id: 'sess-1',
		});
	});
});

describe('backend relay', () => {
	let fetchSpy: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		fetchSpy = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', fetchSpy);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('bufferEvent triggers flush at threshold of 10', async () => {
		for (let i = 0; i < 10; i++) {
			bufferEvent({ event: `backlink.test_${i}`, timestamp: new Date().toISOString() });
		}
		// flush is called synchronously when threshold hit
		await vi.waitFor(() => {
			expect(fetchSpy).toHaveBeenCalledTimes(1);
		});
		const body = JSON.parse(fetchSpy.mock.calls[0][1].body);
		expect(body.events).toHaveLength(10);
	});

	it('flushToBackend sends POST with correct payload', async () => {
		bufferEvent({ event: 'backlink.test', timestamp: '2024-01-01T00:00:00Z' });
		await flushToBackend();
		expect(fetchSpy).toHaveBeenCalledWith('/api/telemetry/events', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: expect.stringContaining('backlink.test'),
		});
	});

	it('flushToBackend does nothing when buffer is empty', async () => {
		// Flush any leftover events from previous tests
		await flushToBackend();
		fetchSpy.mockClear();

		await flushToBackend();
		expect(fetchSpy).not.toHaveBeenCalled();
	});

	it('flushToBackend re-queues events on network failure', async () => {
		fetchSpy.mockRejectedValueOnce(new Error('network error'));
		bufferEvent({ event: 'backlink.retry', timestamp: '2024-01-01T00:00:00Z' });
		await flushToBackend();
		// Event should be re-queued — a second flush should attempt again
		fetchSpy.mockResolvedValueOnce({ ok: true });
		await flushToBackend();
		expect(fetchSpy).toHaveBeenCalledTimes(2);
	});
});
