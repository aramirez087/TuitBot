/**
 * evidenceFunnel.test.ts — Unit tests for evidence funnel analytics helpers.
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
	trackEvidenceRailOpened,
	trackEvidenceSearchExecuted,
	trackEvidencePinned,
	trackEvidenceDismissed,
	trackEvidenceAppliedToSlot,
	trackAutoQueryToggled,
	trackEvidenceContributedToDraft,
	trackEvidenceSearchLatency,
	trackEvidenceDraftMode,
	trackEvidenceStrengthen,
	bufferEvent,
	flushToBackend,
} from '$lib/analytics/evidenceFunnel';

beforeEach(() => {
	vi.clearAllMocks();
});

describe('evidence funnel event helpers', () => {
	it('trackEvidenceRailOpened calls trackFunnel with correct event and props', () => {
		trackEvidenceRailOpened('sess-1', true);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.rail_opened', {
			session_id: 'sess-1',
			has_selection: true,
		});
	});

	it('trackEvidenceSearchExecuted calls trackFunnel with correct event and props', () => {
		trackEvidenceSearchExecuted(50, 5, 'hybrid');
		expect(trackFunnel).toHaveBeenCalledWith('evidence.search_executed', {
			query_length: 50,
			result_count: 5,
			mode: 'hybrid',
		});
	});

	it('trackEvidencePinned calls trackFunnel with correct event and props', () => {
		trackEvidencePinned(42, 'semantic', 0.91);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.pinned', {
			chunk_id: 42,
			match_reason: 'semantic',
			score: 0.91,
		});
	});

	it('trackEvidenceDismissed calls trackFunnel with correct event and props', () => {
		trackEvidenceDismissed(42, 'keyword');
		expect(trackFunnel).toHaveBeenCalledWith('evidence.dismissed', {
			chunk_id: 42,
			match_reason: 'keyword',
		});
	});

	it('trackEvidenceAppliedToSlot calls trackFunnel with correct event and props', () => {
		trackEvidenceAppliedToSlot(42, 1, 'Tweet 2', 'hybrid');
		expect(trackFunnel).toHaveBeenCalledWith('evidence.applied_to_slot', {
			chunk_id: 42,
			slot_index: 1,
			slot_label: 'Tweet 2',
			match_reason: 'hybrid',
		});
	});

	it('trackAutoQueryToggled calls trackFunnel with correct event and props', () => {
		trackAutoQueryToggled(false);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.auto_query_toggled', {
			enabled: false,
		});
	});

	it('trackEvidenceContributedToDraft calls trackFunnel with correct event and props', () => {
		trackEvidenceContributedToDraft(3, 2, 'sess-1');
		expect(trackFunnel).toHaveBeenCalledWith('evidence.contributed_to_draft', {
			pinned_count: 3,
			applied_count: 2,
			session_id: 'sess-1',
		});
	});

	it('trackEvidenceSearchLatency calls trackFunnel with correct event and props', () => {
		trackEvidenceSearchLatency(45, 'semantic', false);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.search_latency', {
			latency_ms: 45,
			mode: 'semantic',
			fallback: false,
		});
	});

	it('trackEvidenceDraftMode calls trackFunnel with correct event and props', () => {
		trackEvidenceDraftMode('thread', 2, 1);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.draft_mode', {
			mode: 'thread',
			pinned_count: 2,
			applied_count: 1,
		});
	});

	it('trackEvidenceStrengthen calls trackFunnel with correct event and props', () => {
		trackEvidenceStrengthen(3, 2);
		expect(trackFunnel).toHaveBeenCalledWith('evidence.strengthen_draft', {
			block_count: 3,
			pinned_count: 2,
		});
	});
});

describe('evidence backend relay', () => {
	let fetchSpy: ReturnType<typeof vi.fn>;

	beforeEach(async () => {
		fetchSpy = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', fetchSpy);
		// Drain any buffered events from previous test helpers
		await flushToBackend();
		fetchSpy.mockClear();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('bufferEvent triggers flush at threshold of 10', async () => {
		for (let i = 0; i < 10; i++) {
			bufferEvent({ event: `evidence.test_${i}`, timestamp: new Date().toISOString() });
		}
		await vi.waitFor(() => {
			expect(fetchSpy).toHaveBeenCalledTimes(1);
		});
		const body = JSON.parse(fetchSpy.mock.calls[0][1].body);
		expect(body.events).toHaveLength(10);
	});

	it('flushToBackend sends POST with correct payload', async () => {
		bufferEvent({ event: 'evidence.test', timestamp: '2026-03-23T00:00:00Z' });
		await flushToBackend();
		expect(fetchSpy).toHaveBeenCalledWith('/api/telemetry/events', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: expect.stringContaining('evidence.test'),
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
		bufferEvent({ event: 'evidence.retry', timestamp: '2026-03-23T00:00:00Z' });
		await flushToBackend();
		fetchSpy.mockResolvedValueOnce({ ok: true });
		await flushToBackend();
		expect(fetchSpy).toHaveBeenCalledTimes(2);
	});
});
