/**
 * approvalStore.test.ts — Unit tests for src/lib/stores/approval.ts
 *
 * Covers: loadItems, loadStats, approveItem, rejectItem, editItem,
 * approveAllItems, filter setters, moveFocus, and derived stores.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { ApprovalItem } from '$lib/api';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		approval: {
			list: vi.fn(),
			stats: vi.fn(),
			approve: vi.fn(),
			reject: vi.fn(),
			edit: vi.fn(),
			approveAll: vi.fn()
		}
	}
}));

vi.mock('$lib/stores/websocket', () => ({
	events: {
		subscribe: vi.fn((cb: (v: unknown[]) => void) => {
			cb([]);
			return () => {};
		})
	}
}));

// --- Imports after mocks ----------------------------------------------------

import { api } from '$lib/api';
import * as store from '../../src/lib/stores/approval';

// --- Fixtures ---------------------------------------------------------------

const makeItem = (id: number, status = 'pending') => ({
	id,
	action_type: 'reply' as const,
	target_tweet_id: '123',
	target_author: `author_${id}`,
	generated_content: `Content ${id}`,
	topic: 'test',
	archetype: 'builder',
	score: 0.8,
	status,
	created_at: '2026-03-14T00:00:00.000Z',
	media_paths: [],
	detected_risks: [],
	qa_score: 0.9,
	qa_hard_flags: [],
	qa_soft_flags: [],
	qa_requires_override: false
});

const ITEMS = [makeItem(1), makeItem(2), makeItem(3)];
const STATS = { pending: 3, approved: 0, rejected: 0, failed: 0, scheduled: 0 };

// --- Helpers ----------------------------------------------------------------

function resetStores() {
	store.items.set([]);
	store.stats.set(null);
	store.loading.set(false);
	store.error.set(null);
	store.selectedStatus.set('pending');
	store.selectedType.set('all');
	store.reviewerFilter.set('');
	store.dateFilter.set('all');
	store.focusedIndex.set(0);
}

// --- Tests ------------------------------------------------------------------

beforeEach(() => {
	resetStores();
	vi.clearAllMocks();
	(api.approval.list as ReturnType<typeof vi.fn>).mockResolvedValue(ITEMS);
	(api.approval.stats as ReturnType<typeof vi.fn>).mockResolvedValue(STATS);
	(api.approval.approve as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true });
	(api.approval.reject as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true });
	(api.approval.edit as ReturnType<typeof vi.fn>).mockResolvedValue(makeItem(1));
	(api.approval.approveAll as ReturnType<typeof vi.fn>).mockResolvedValue({ count: 3 });
});

// ---------------------------------------------------------------------------
// loadItems
// ---------------------------------------------------------------------------

describe('loadItems', () => {
	it('sets loading true then false around the API call', async () => {
		const loadingStates: boolean[] = [];
		const unsub = store.loading.subscribe((v) => loadingStates.push(v));
		await store.loadItems();
		unsub();
		expect(loadingStates).toContain(true);
		expect(loadingStates[loadingStates.length - 1]).toBe(false);
	});

	it('populates items on success', async () => {
		await store.loadItems();
		expect(get(store.items)).toHaveLength(3);
		expect(get(store.items)[0].id).toBe(1);
	});

	it('clears error on success', async () => {
		store.error.set('old error');
		await store.loadItems();
		expect(get(store.error)).toBeNull();
	});

	it('sets error on API failure', async () => {
		(api.approval.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Network error')
		);
		await store.loadItems();
		expect(get(store.error)).toBe('Network error');
	});

	it('sets generic error for non-Error rejections', async () => {
		(api.approval.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce('boom');
		await store.loadItems();
		expect(get(store.error)).toBe('Failed to load approval items');
	});

	it('resets focusedIndex when reset=true', async () => {
		store.focusedIndex.set(5);
		await store.loadItems(true);
		expect(get(store.focusedIndex)).toBe(0);
	});

	it('does not reset focusedIndex when reset=false', async () => {
		store.focusedIndex.set(2);
		await store.loadItems(false);
		expect(get(store.focusedIndex)).toBe(2);
	});

	it('passes status filter to API', async () => {
		store.selectedStatus.set('approved');
		await store.loadItems();
		expect(api.approval.list).toHaveBeenCalledWith(
			expect.objectContaining({ status: 'approved' })
		);
	});

	it('passes type filter to API when not "all"', async () => {
		store.selectedType.set('tweet');
		await store.loadItems();
		expect(api.approval.list).toHaveBeenCalledWith(
			expect.objectContaining({ type: 'tweet' })
		);
	});

	it('omits type from API when "all"', async () => {
		store.selectedType.set('all');
		await store.loadItems();
		expect(api.approval.list).toHaveBeenCalledWith(
			expect.objectContaining({ type: undefined })
		);
	});
});

// ---------------------------------------------------------------------------
// loadStats
// ---------------------------------------------------------------------------

describe('loadStats', () => {
	it('sets stats on success', async () => {
		await store.loadStats();
		expect(get(store.stats)).toEqual(STATS);
	});

	it('silently ignores API failure', async () => {
		(api.approval.stats as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('oops'));
		await expect(store.loadStats()).resolves.toBeUndefined();
		expect(get(store.error)).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// approveItem
// ---------------------------------------------------------------------------

describe('approveItem', () => {
	beforeEach(() => {
		store.items.set([...ITEMS]);
		store.focusedIndex.set(0);
	});

	it('removes the item from the list', async () => {
		await store.approveItem(1);
		const remaining = get(store.items);
		expect(remaining.find((i) => i.id === 1)).toBeUndefined();
		expect(remaining).toHaveLength(2);
	});

	it('calls the approve API', async () => {
		await store.approveItem(2);
		expect(api.approval.approve).toHaveBeenCalledWith(2, 'dashboard');
	});

	it('refreshes stats after approval', async () => {
		await store.approveItem(1);
		expect(api.approval.stats).toHaveBeenCalled();
	});

	it('sets error on API failure', async () => {
		(api.approval.approve as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Forbidden')
		);
		await store.approveItem(1);
		expect(get(store.error)).toBe('Forbidden');
	});

	it('clamps focusedIndex when last item is removed', async () => {
		store.items.set([makeItem(99)]);
		store.focusedIndex.set(0);
		await store.approveItem(99);
		expect(get(store.focusedIndex)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// rejectItem
// ---------------------------------------------------------------------------

describe('rejectItem', () => {
	beforeEach(() => {
		store.items.set([...ITEMS]);
	});

	it('removes the item from the list', async () => {
		await store.rejectItem(2);
		expect(get(store.items).find((i) => i.id === 2)).toBeUndefined();
	});

	it('calls reject API with optional notes', async () => {
		await store.rejectItem(1, 'Off-topic');
		expect(api.approval.reject).toHaveBeenCalledWith(1, 'dashboard', 'Off-topic');
	});

	it('calls reject API without notes', async () => {
		await store.rejectItem(1);
		expect(api.approval.reject).toHaveBeenCalledWith(1, 'dashboard', undefined);
	});

	it('refreshes stats after rejection', async () => {
		await store.rejectItem(1);
		expect(api.approval.stats).toHaveBeenCalled();
	});

	it('sets error on API failure', async () => {
		(api.approval.reject as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Server error')
		);
		await store.rejectItem(1);
		expect(get(store.error)).toBe('Server error');
	});
});

// ---------------------------------------------------------------------------
// editItem
// ---------------------------------------------------------------------------

describe('editItem', () => {
	beforeEach(() => {
		store.items.set([...ITEMS]);
	});

	it('updates the item in place', async () => {
		const updated = { ...makeItem(1), generated_content: 'Updated content' };
		(api.approval.edit as ReturnType<typeof vi.fn>).mockResolvedValueOnce(updated);
		await store.editItem(1, 'Updated content');
		const item = get(store.items).find((i) => i.id === 1);
		expect(item?.generated_content).toBe('Updated content');
	});

	it('calls edit API with content and optional media paths', async () => {
		await store.editItem(1, 'New text', ['path/to/img.png']);
		expect(api.approval.edit).toHaveBeenCalledWith(1, 'New text', ['path/to/img.png']);
	});

	it('sets error on API failure', async () => {
		(api.approval.edit as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Edit failed')
		);
		await store.editItem(1, 'bad');
		expect(get(store.error)).toBe('Edit failed');
	});
});

// ---------------------------------------------------------------------------
// approveAllItems
// ---------------------------------------------------------------------------

describe('approveAllItems', () => {
	beforeEach(() => {
		store.items.set([...ITEMS]);
		store.focusedIndex.set(2);
	});

	it('clears items list', async () => {
		await store.approveAllItems();
		expect(get(store.items)).toHaveLength(0);
	});

	it('resets focusedIndex to 0', async () => {
		await store.approveAllItems();
		expect(get(store.focusedIndex)).toBe(0);
	});

	it('calls approveAll API', async () => {
		await store.approveAllItems();
		expect(api.approval.approveAll).toHaveBeenCalled();
	});

	it('refreshes stats', async () => {
		await store.approveAllItems();
		expect(api.approval.stats).toHaveBeenCalled();
	});

	it('sets error on API failure', async () => {
		(api.approval.approveAll as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Bulk fail')
		);
		await store.approveAllItems();
		expect(get(store.error)).toBe('Bulk fail');
	});
});

// ---------------------------------------------------------------------------
// Filter setters
// ---------------------------------------------------------------------------

describe('filter setters', () => {
	it('setStatusFilter updates selectedStatus and reloads', async () => {
		await store.setStatusFilter('approved');
		expect(get(store.selectedStatus)).toBe('approved');
		expect(api.approval.list).toHaveBeenCalled();
	});

	it('setTypeFilter updates selectedType and reloads', async () => {
		await store.setTypeFilter('tweet');
		expect(get(store.selectedType)).toBe('tweet');
		expect(api.approval.list).toHaveBeenCalled();
	});

	it('setReviewerFilter updates reviewerFilter and reloads', async () => {
		await store.setReviewerFilter('alice');
		expect(get(store.reviewerFilter)).toBe('alice');
		expect(api.approval.list).toHaveBeenCalled();
	});

	it('setDateFilter updates dateFilter and reloads', async () => {
		await store.setDateFilter('7d');
		expect(get(store.dateFilter)).toBe('7d');
		expect(api.approval.list).toHaveBeenCalled();
	});
});

// ---------------------------------------------------------------------------
// moveFocus
// ---------------------------------------------------------------------------

describe('moveFocus', () => {
	beforeEach(() => {
		store.items.set([...ITEMS]); // 3 items → indices 0, 1, 2
		store.focusedIndex.set(1);
	});

	it('moves forward by 1', () => {
		store.moveFocus(1);
		expect(get(store.focusedIndex)).toBe(2);
	});

	it('moves backward by 1', () => {
		store.moveFocus(-1);
		expect(get(store.focusedIndex)).toBe(0);
	});

	it('clamps at upper bound', () => {
		store.focusedIndex.set(2);
		store.moveFocus(5);
		expect(get(store.focusedIndex)).toBe(2);
	});

	it('clamps at lower bound', () => {
		store.focusedIndex.set(0);
		store.moveFocus(-5);
		expect(get(store.focusedIndex)).toBe(0);
	});

	it('no-ops when items list is empty', () => {
		store.items.set([]);
		store.focusedIndex.set(0);
		store.moveFocus(1);
		expect(get(store.focusedIndex)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// Derived stores
// ---------------------------------------------------------------------------

describe('derived stores', () => {
	it('focusedItem returns the item at focusedIndex', () => {
		store.items.set([...ITEMS]);
		store.focusedIndex.set(1);
		expect(get(store.focusedItem)?.id).toBe(2);
	});

	it('focusedItem returns null for empty list', () => {
		store.items.set([]);
		expect(get(store.focusedItem)).toBeNull();
	});

	it('focusedItem returns null for out-of-range index', () => {
		store.items.set([makeItem(1)]);
		store.focusedIndex.set(99);
		expect(get(store.focusedItem)).toBeNull();
	});

	it('isEmpty is true when not loading and items is empty', () => {
		store.items.set([]);
		store.loading.set(false);
		expect(get(store.isEmpty)).toBe(true);
	});

	it('isEmpty is false when loading even if items is empty', () => {
		store.items.set([]);
		store.loading.set(true);
		expect(get(store.isEmpty)).toBe(false);
	});

	it('isEmpty is false when items are present', () => {
		store.items.set([...ITEMS]);
		store.loading.set(false);
		expect(get(store.isEmpty)).toBe(false);
	});

	it('pendingCount returns stats.pending', () => {
		store.stats.set(STATS);
		expect(get(store.pendingCount)).toBe(3);
	});

	it('pendingCount returns 0 when stats is null', () => {
		store.stats.set(null);
		expect(get(store.pendingCount)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// startAutoRefresh / stopAutoRefresh
// ---------------------------------------------------------------------------

describe('startAutoRefresh / stopAutoRefresh', () => {
	it('startAutoRefresh sets up an interval without crashing', () => {
		vi.useFakeTimers();
		expect(() => store.startAutoRefresh(1000)).not.toThrow();
		vi.runOnlyPendingTimers(); // advance one tick only (not infinite loop)
		vi.useRealTimers();
	});

	it('stopAutoRefresh clears the interval', () => {
		vi.useFakeTimers();
		store.startAutoRefresh(1000);
		expect(() => store.stopAutoRefresh()).not.toThrow();
		vi.useRealTimers();
	});

	it('stopAutoRefresh is safe to call when no interval is running', () => {
		expect(() => store.stopAutoRefresh()).not.toThrow();
	});

	it('startAutoRefresh replaces any existing interval (calls stop first)', () => {
		vi.useFakeTimers();
		store.startAutoRefresh(1000);
		// Calling again should not throw and should replace the interval
		expect(() => store.startAutoRefresh(2000)).not.toThrow();
		store.stopAutoRefresh();
		vi.useRealTimers();
	});
});

// --- Tests for failed_post_recovery filter (Core C2 integration) -----------

describe('Failed Post Recovery Filter', () => {
	it('should load failed posts when selectedStatus is set to "failed"', async () => {
		const mockItems = [
			{
				id: 100,
				action_type: 'failed_post_recovery',
				status: 'pending',
				generated_content: 'Failed thread retry',
				target_tweet_id: '',
				target_author: '',
				topic: 'Tech',
				archetype: 'builder',
				score: 0,
				created_at: '2026-03-19T00:00:00.000Z',
				media_paths: [],
				detected_risks: [],
				qa_score: 0.5,
				qa_hard_flags: [],
				qa_soft_flags: [],
				qa_requires_override: false
			}
		];

		vi.mocked(api.approval.list).mockResolvedValue(mockItems);

		store.selectedStatus.set('failed');
		await store.loadItems(true);

		// Should pass action_type=failed_post_recovery and status=pending to API
		expect(vi.mocked(api.approval.list)).toHaveBeenCalledWith({
			status: 'pending',
			type: undefined,
			reviewed_by: undefined,
			since: undefined,
			action_type: 'failed_post_recovery'
		});

		expect(get(store.items)).toEqual(mockItems);
	});

	it('should pass action_type=undefined when status is not "failed"', async () => {
		vi.mocked(api.approval.list).mockResolvedValue([]);

		store.selectedStatus.set('approved');
		await store.loadItems(true);

		expect(vi.mocked(api.approval.list)).toHaveBeenCalledWith({
			status: 'approved',
			type: undefined,
			reviewed_by: undefined,
			since: undefined,
			action_type: undefined
		});
	});

	it('should switch from pending to failed filter', async () => {
		const pendingItem = makeItem(10, 'pending');
		const failedItem: ApprovalItem = {
			id: 20,
			action_type: 'failed_post_recovery',
			status: 'pending',
			generated_content: 'Failed post',
			target_tweet_id: '',
			target_author: '',
			topic: 'test',
			archetype: 'builder',
			score: 0,
			created_at: '2026-03-19T00:00:00.000Z',
			media_paths: [],
			detected_risks: [],
			qa_score: 0.5,
			qa_hard_flags: [],
			qa_soft_flags: [],
			qa_requires_override: false
		};

		// First load pending
		vi.mocked(api.approval.list).mockResolvedValueOnce([pendingItem]);
		store.selectedStatus.set('pending');
		await store.loadItems(true);
		expect(get(store.items)).toEqual([pendingItem]);

		// Then switch to failed
		vi.mocked(api.approval.list).mockResolvedValueOnce([failedItem]);
		await store.setStatusFilter('failed');
		expect(vi.mocked(api.approval.list)).toHaveBeenLastCalledWith({
			status: 'pending',
			type: undefined,
			reviewed_by: undefined,
			since: undefined,
			action_type: 'failed_post_recovery'
		});
		expect(get(store.items)).toEqual([failedItem]);
	});
});
