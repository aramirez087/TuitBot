/**
 * approvalStorePagination.test.ts — Unit tests for pagination + search state
 * added in Task ade3b21a (approval queue pagination, filters, bulk actions).
 *
 * Covers: currentPage, pageSize, searchQuery, paginatedItems, totalCount,
 * totalPages, setCurrentPage, setSearchQuery, page-reset-on-filter-change.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

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

// ---------------------------------------------------------------------------
// Import store as namespace so we access the same module-level singleton
// that the store's derived stores observe. This matches the pattern used in
// approvalStore.test.ts and avoids stale-binding issues with named imports.
// ---------------------------------------------------------------------------

import * as store from '../../src/lib/stores/approval';

/** Build an ApprovalItem stub matching the full ApprovalItem interface. */
function makeItem(id: number, content = `Generated content ${id}`) {
	return {
		id,
		action_type: 'tweet',
		target_tweet_id: '',
		target_author: `author_${id}`,
		generated_content: content,
		topic: 'test-topic',
		archetype: 'default',
		score: 0.8,
		status: 'pending',
		created_at: '2026-01-01T00:00:00Z',
		media_paths: [],
		detected_risks: [],
		qa_score: 1.0,
		qa_hard_flags: [],
		qa_soft_flags: [],
		qa_requires_override: false
	};
}

function resetStores() {
	store.items.set([]);
	store.currentPage.set(1);
	store.pageSize.set(20);
	store.searchQuery.set('');
}

beforeEach(() => {
	resetStores();
	vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// setCurrentPage
// ---------------------------------------------------------------------------

describe('setCurrentPage', () => {
	it('sets current page to given value', () => {
		store.setCurrentPage(3);
		expect(get(store.currentPage)).toBe(3);
	});

	it('clamps to minimum of 1', () => {
		store.setCurrentPage(0);
		expect(get(store.currentPage)).toBe(1);
	});

	it('clamps negative values to 1', () => {
		store.setCurrentPage(-5);
		expect(get(store.currentPage)).toBe(1);
	});

	it('accepts large page numbers', () => {
		store.setCurrentPage(999);
		expect(get(store.currentPage)).toBe(999);
	});
});

// ---------------------------------------------------------------------------
// setSearchQuery
// ---------------------------------------------------------------------------

describe('setSearchQuery', () => {
	it('updates searchQuery store', () => {
		store.setSearchQuery('hello world');
		expect(get(store.searchQuery)).toBe('hello world');
	});

	it('resets currentPage to 1 on query change', () => {
		store.currentPage.set(5);
		store.setSearchQuery('reset me');
		expect(get(store.currentPage)).toBe(1);
	});

	it('accepts empty string (clear search)', () => {
		store.searchQuery.set('something');
		store.setSearchQuery('');
		expect(get(store.searchQuery)).toBe('');
	});
});

// ---------------------------------------------------------------------------
// paginatedItems / totalCount / totalPages
// ---------------------------------------------------------------------------

describe('paginatedItems derived store', () => {
	it('returns empty array when items is empty', () => {
		expect(get(store.paginatedItems)).toEqual([]);
	});

	it('returns first page slice when items exist', () => {
		const all = Array.from({ length: 25 }, (_, i) => makeItem(i));
		store.items.set(all);
		store.pageSize.set(20);
		store.currentPage.set(1);
		expect(get(store.paginatedItems)).toHaveLength(20);
		expect(get(store.paginatedItems)[0].id).toBe(0);
	});

	it('returns second page slice correctly', () => {
		const all = Array.from({ length: 25 }, (_, i) => makeItem(i));
		store.items.set(all);
		store.pageSize.set(20);
		store.currentPage.set(2);
		expect(get(store.paginatedItems)).toHaveLength(5);
		expect(get(store.paginatedItems)[0].id).toBe(20);
	});

	it('returns all items when count is less than page size', () => {
		store.items.set([makeItem(1), makeItem(2), makeItem(3)]);
		store.pageSize.set(20);
		store.currentPage.set(1);
		expect(get(store.paginatedItems)).toHaveLength(3);
	});
});

describe('totalCount derived store', () => {
	it('reflects number of items when search is empty', () => {
		store.items.set(Array.from({ length: 7 }, (_, i) => makeItem(i)));
		expect(get(store.totalCount)).toBe(7);
	});

	it('is 0 when items empty', () => {
		expect(get(store.totalCount)).toBe(0);
	});

	it('narrows when search query matches generated_content', () => {
		store.items.set([
			makeItem(1, 'hello world'),
			makeItem(2, 'goodbye world'),
			makeItem(3, 'unrelated')
		]);
		store.setSearchQuery('world');
		expect(get(store.totalCount)).toBe(2);
	});

	it('returns 0 when no items match query', () => {
		store.items.set([makeItem(1, 'totally unrelated'), makeItem(2, 'also unrelated')]);
		store.setSearchQuery('zzznomatch');
		expect(get(store.totalCount)).toBe(0);
	});
});

describe('totalPages derived store', () => {
	it('is 0 when no items', () => {
		expect(get(store.totalPages)).toBe(0);
	});

	it('is 1 when items fit on one page', () => {
		store.items.set(Array.from({ length: 15 }, (_, i) => makeItem(i)));
		store.pageSize.set(20);
		expect(get(store.totalPages)).toBe(1);
	});

	it('rounds up partial last page', () => {
		store.items.set(Array.from({ length: 21 }, (_, i) => makeItem(i)));
		store.pageSize.set(20);
		expect(get(store.totalPages)).toBe(2);
	});

	it('equals exact count when evenly divisible', () => {
		store.items.set(Array.from({ length: 40 }, (_, i) => makeItem(i)));
		store.pageSize.set(20);
		expect(get(store.totalPages)).toBe(2);
	});
});

// ---------------------------------------------------------------------------
// Page reset on filter changes
// ---------------------------------------------------------------------------

describe('page resets to 1 on filter changes', () => {
	beforeEach(() => {
		store.currentPage.set(4);
	});

	it('setStatusFilter resets page', () => {
		store.setStatusFilter('approved');
		expect(get(store.currentPage)).toBe(1);
	});

	it('setTypeFilter resets page', () => {
		store.setTypeFilter('thread');
		expect(get(store.currentPage)).toBe(1);
	});

	it('setReviewerFilter resets page', () => {
		store.setReviewerFilter('user-123');
		expect(get(store.currentPage)).toBe(1);
	});

	it('setDateFilter resets page', () => {
		store.setDateFilter('7d');
		expect(get(store.currentPage)).toBe(1);
	});

	it('setSearchQuery resets page', () => {
		store.setSearchQuery('query');
		expect(get(store.currentPage)).toBe(1);
	});
});

// ---------------------------------------------------------------------------
// Search filtering
// ---------------------------------------------------------------------------

describe('search filtering in paginatedItems', () => {
	it('matches generated_content (case-insensitive)', () => {
		store.items.set([
			makeItem(1, 'Hello World'),
			makeItem(2, 'hello world lowercase'),
			makeItem(3, 'no match here')
		]);
		store.setSearchQuery('hello');
		expect(get(store.totalCount)).toBe(2);
	});

	it('matches target_author field', () => {
		store.items.set([makeItem(1), makeItem(2), makeItem(3)]);
		// makeItem sets target_author to `author_${id}`
		store.setSearchQuery('author_2');
		expect(get(store.totalCount)).toBe(1);
	});

	it('returns all items when query is empty', () => {
		store.items.set(Array.from({ length: 5 }, (_, i) => makeItem(i)));
		store.setSearchQuery('');
		expect(get(store.totalCount)).toBe(5);
	});
});
