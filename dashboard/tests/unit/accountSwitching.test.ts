/**
 * accountSwitching.test.ts
 *
 * Tests for the tuitbot:account-switched event handler added to each store
 * in the multi-account UI work (PR #211). Verifies that all stores reset
 * and refetch when the account switches.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		analytics: {
			summary: vi.fn().mockResolvedValue({
				followers: { current: 0, change_7d: 0, change_30d: 0 },
				actions_today: { replies: 0, tweets: 0, threads: 0 },
				engagement: {
					avg_reply_score: 0,
					avg_tweet_score: 0,
					total_replies_sent: 0,
					total_tweets_posted: 0
				}
			}),
			followers: vi.fn().mockResolvedValue([]),
			recentPerformance: vi.fn().mockResolvedValue([])
		},
		approval: {
			list: vi.fn().mockResolvedValue({ items: [], total: 0, page: 1, page_size: 20 }),
			stats: vi.fn().mockResolvedValue(null)
		},
		costs: {
			summary: vi.fn().mockResolvedValue(null),
			daily: vi.fn().mockResolvedValue([]),
			modelBreakdown: vi.fn().mockResolvedValue([]),
			typeBreakdown: vi.fn().mockResolvedValue([]),
			xapiSummary: vi.fn().mockResolvedValue(null),
			xapiDaily: vi.fn().mockResolvedValue([]),
			xapiEndpoints: vi.fn().mockResolvedValue([])
		},
		activity: {
			list: vi.fn().mockResolvedValue({ items: [], total: 0 }),
			rateLimits: vi.fn().mockResolvedValue(null)
		},
		observability: { list: vi.fn().mockResolvedValue({ items: [] }) },
		targets: { list: vi.fn().mockResolvedValue([]) },
		strategy: { get: vi.fn().mockResolvedValue(null) },
		drafts: { list: vi.fn().mockResolvedValue({ items: [] }) }
	},
	events: { subscribe: vi.fn(() => () => {}) }
}));

vi.mock('../../src/lib/stores/websocket', () => ({
	events: { subscribe: vi.fn(() => () => {}) }
}));

// ---------------------------------------------------------------------------

import * as analyticsStore from '../../src/lib/stores/analytics';
import * as approvalStore from '../../src/lib/stores/approval';

const fireAccountSwitched = () =>
	window.dispatchEvent(new Event('tuitbot:account-switched'));

describe('account switching — analytics store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		analyticsStore.summary.set(null);
		analyticsStore.loading.set(false);
		analyticsStore.error.set(null);
	});

	it('triggers loadDashboard when account-switched fires', async () => {
		const { api } = await import('$lib/api');
		fireAccountSwitched();
		await vi.runAllTimersAsync().catch(() => {});
		// Allow microtasks to flush
		await new Promise((r) => setTimeout(r, 0));
		expect(api.analytics.summary).toHaveBeenCalled();
	});
});

describe('account switching — approval store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		approvalStore.currentPage.set(5);
		approvalStore.focusedIndex.set(3);
	});

	it('resets page and focusedIndex when account-switched fires', async () => {
		fireAccountSwitched();
		await new Promise((r) => setTimeout(r, 0));
		expect(get(approvalStore.currentPage)).toBe(1);
		expect(get(approvalStore.focusedIndex)).toBe(0);
	});

	it('calls loadItems after account switch', async () => {
		const { api } = await import('$lib/api');
		fireAccountSwitched();
		await new Promise((r) => setTimeout(r, 0));
		expect(api.approval.list).toHaveBeenCalled();
	});
});
