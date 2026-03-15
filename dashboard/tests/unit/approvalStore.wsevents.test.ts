/**
 * approvalStore.wsevents.test.ts — Tests for the module-level wsEvents
 * subscription branches in approval.ts (lines 153-181).
 *
 * Uses vi.hoisted() to capture the subscriber callback so we can replay
 * WebSocket events after module import.
 *
 * NOTE: The approval store tracks `lastEventCount` as module-level state.
 * The guard `if ($events.length === lastEventCount) return` means each
 * successive dispatch must use a strictly larger array. We use `dispatch()`
 * helper which auto-increments the padding length.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { approvalItem } from '../helpers/fixtures';

// --- Capture the wsEvents callback via vi.hoisted() -------------------------

const captured = vi.hoisted(() => ({ cb: null as ((v: unknown[]) => void) | null }));

const mockApi = vi.hoisted(() => ({
	approval: {
		list: vi.fn().mockResolvedValue([]),
		stats: vi.fn().mockResolvedValue({
			pending: 0, approved: 0, rejected: 0, scheduled: 0, total: 0
		}),
		approve: vi.fn().mockResolvedValue({ success: true }),
		reject: vi.fn().mockResolvedValue({ success: true }),
		edit: vi.fn().mockResolvedValue({ success: true }),
		approveAll: vi.fn().mockResolvedValue({ success: true })
	}
}));

vi.mock('$lib/api', () => ({ api: mockApi }));

vi.mock('$lib/stores/websocket', () => ({
	events: {
		subscribe: vi.fn((cb: (v: unknown[]) => void) => {
			captured.cb = cb;
			cb([]); // initial call with empty events (length=0, early returns, lastEventCount stays 0)
			return () => {};
		})
	}
}));

// --- Imports after mocks ----------------------------------------------------

import * as store from '../../src/lib/stores/approval';

// --- Fixtures ----------------------------------------------------------------

const ITEM = approvalItem({
	id: 1,
	status: 'pending',
	created_at: '2026-01-01T00:00:00Z'
});

// --- Dispatch helper ---------------------------------------------------------
// The store uses `lastEventCount` to debounce: if the new event array has the
// same length as the previous, it early-returns. We must always pass a longer
// array. This counter starts at 0 (matching module init where cb([]) left
// lastEventCount=0) and is incremented on each dispatch.

let dispatchSeq = 0;

function dispatch(event: Record<string, unknown>) {
	dispatchSeq++;
	// Build array: real event at [0], padding nulls at [1..dispatchSeq-1]
	const arr: unknown[] = [event, ...Array(dispatchSeq - 1).fill({ type: '_pad' })];
	captured.cb?.(arr);
}

// ---------------------------------------------------------------------------
// WebSocket event branches
// ---------------------------------------------------------------------------

describe('wsEvents subscription branches', () => {
	beforeEach(() => {
		mockApi.approval.stats.mockClear();
		mockApi.approval.list.mockClear();
		store.items.set([]);
		store.stats.set(null);
		store.selectedStatus.set('pending');
		// NOTE: dispatchSeq intentionally NOT reset — module lastEventCount is
		// persistent and we must keep in sync with it.
	});

	it('fires ApprovalQueued event and calls stats API', () => {
		dispatch({ type: 'ApprovalQueued', id: 10 });
		// loadStats() is called synchronously before the first await
		expect(mockApi.approval.stats).toHaveBeenCalled();
	});

	it('fires ApprovalQueued with pending filter and calls list API', () => {
		store.selectedStatus.set('pending');
		mockApi.approval.list.mockClear();
		dispatch({ type: 'ApprovalQueued', id: 11 });
		expect(mockApi.approval.list).toHaveBeenCalled();
	});

	it('fires ApprovalQueued with "all" filter and calls list API', () => {
		store.selectedStatus.set('all');
		mockApi.approval.list.mockClear();
		dispatch({ type: 'ApprovalQueued', id: 12 });
		expect(mockApi.approval.list).toHaveBeenCalled();
	});

	it('fires ApprovalQueued with "approved" filter and does NOT call list API', () => {
		store.selectedStatus.set('approved');
		mockApi.approval.list.mockClear();
		dispatch({ type: 'ApprovalQueued', id: 13 });
		expect(mockApi.approval.list).not.toHaveBeenCalled();
	});

	it('fires ApprovalUpdated and calls stats API', () => {
		mockApi.approval.stats.mockClear();
		dispatch({ type: 'ApprovalUpdated', id: 1, status: 'approved' });
		expect(mockApi.approval.stats).toHaveBeenCalled();
	});

	it('fires ApprovalUpdated and removes item when filter does not match new status', () => {
		store.items.set([{ ...ITEM, id: 1, status: 'pending' }]);
		store.selectedStatus.set('pending');
		// Status → 'approved', but filter is 'pending' → item should be removed
		dispatch({ type: 'ApprovalUpdated', id: 1, status: 'approved' });
		expect(get(store.items).find((i) => i.id === 1)).toBeUndefined();
	});

	it('fires ApprovalUpdated with "all" filter and updates item in place', () => {
		store.items.set([{ ...ITEM, id: 1, status: 'pending' }]);
		store.selectedStatus.set('all');
		dispatch({ type: 'ApprovalUpdated', id: 1, status: 'approved' });
		expect(get(store.items).find((i) => i.id === 1)?.status).toBe('approved');
	});

	it('fires ApprovalUpdated with matching status filter and updates item in place', () => {
		store.items.set([{ ...ITEM, id: 2, status: 'approved' }]);
		store.selectedStatus.set('approved');
		dispatch({ type: 'ApprovalUpdated', id: 2, status: 'approved' });
		expect(get(store.items).find((i) => i.id === 2)?.status).toBe('approved');
	});

	it('ignores unknown event types without calling APIs', () => {
		mockApi.approval.stats.mockClear();
		mockApi.approval.list.mockClear();
		dispatch({ type: 'UnknownEvent', id: 99 });
		expect(mockApi.approval.stats).not.toHaveBeenCalled();
		expect(mockApi.approval.list).not.toHaveBeenCalled();
	});
});
