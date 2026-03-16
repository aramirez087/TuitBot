/**
 * analyticsStore.test.ts — Unit tests for src/lib/stores/analytics.ts
 *
 * Covers: loadDashboard (success/error), derived stores (followerCount,
 * followerChange7d, repliesToday, tweetsToday, avgEngagement),
 * startAutoRefresh / stopAutoRefresh lifecycle.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		analytics: {
			summary: vi.fn(),
			followers: vi.fn(),
			recentPerformance: vi.fn()
		}
	}
}));

// --- Imports after mocks ----------------------------------------------------

import { api } from '$lib/api';
import * as store from '../../src/lib/stores/analytics';
import type { AnalyticsSummary, FollowerSnapshot, PerformanceItem } from '../../src/lib/api/types';

// --- Fixtures ---------------------------------------------------------------

const mockSummary: AnalyticsSummary = {
	followers: { current: 1500, change_7d: 50, change_30d: 200 },
	actions_today: { replies: 4, tweets: 1, threads: 0 },
	engagement: {
		avg_reply_score: 0.80,
		avg_tweet_score: 0.70,
		total_replies_sent: 100,
		total_tweets_posted: 20
	},
	top_topics: [
		{ topic: 'devtools', format: 'reply', total_posts: 40, avg_performance: 0.85 }
	]
};

const mockSnapshots: FollowerSnapshot[] = [
	{ snapshot_date: '2026-03-01', follower_count: 1450, following_count: 200, tweet_count: 300 },
	{ snapshot_date: '2026-03-07', follower_count: 1500, following_count: 205, tweet_count: 310 }
];

const mockPerformance: PerformanceItem[] = [
	{
		content_type: 'reply',
		content_preview: 'Great point!',
		likes: 10,
		replies_received: 3,
		retweets: 2,
		impressions: 500,
		performance_score: 0.8,
		posted_at: '2026-03-01T10:00:00.000Z'
	}
];

// --- Reset helper -----------------------------------------------------------

function resetStores() {
	store.summary.set(null);
	store.followerSnapshots.set([]);
	store.recentPerformance.set([]);
	store.loading.set(false);
	store.error.set(null);
}

// --- Tests ------------------------------------------------------------------

beforeEach(() => {
	resetStores();
	vi.clearAllMocks();
	(api.analytics.summary as ReturnType<typeof vi.fn>).mockResolvedValue(mockSummary);
	(api.analytics.followers as ReturnType<typeof vi.fn>).mockResolvedValue(mockSnapshots);
	(api.analytics.recentPerformance as ReturnType<typeof vi.fn>).mockResolvedValue(mockPerformance);
});

afterEach(() => {
	store.stopAutoRefresh();
});

// ---------------------------------------------------------------------------
// loadDashboard
// ---------------------------------------------------------------------------

describe('loadDashboard', () => {
	it('sets loading true then false', async () => {
		const states: boolean[] = [];
		const unsub = store.loading.subscribe((v) => states.push(v));
		await store.loadDashboard();
		unsub();
		expect(states).toContain(true);
		expect(states[states.length - 1]).toBe(false);
	});

	it('populates all three stores on success', async () => {
		await store.loadDashboard();
		expect(get(store.summary)).toEqual(mockSummary);
		expect(get(store.followerSnapshots)).toHaveLength(2);
		expect(get(store.recentPerformance)).toHaveLength(1);
	});

	it('calls analytics.followers with the provided days arg', async () => {
		await store.loadDashboard(7);
		expect(api.analytics.followers).toHaveBeenCalledWith(7);
	});

	it('defaults to 30 days when no argument provided', async () => {
		await store.loadDashboard();
		expect(api.analytics.followers).toHaveBeenCalledWith(30);
	});

	it('clears error before fetching', async () => {
		store.error.set('stale error');
		await store.loadDashboard();
		expect(get(store.error)).toBeNull();
	});

	it('sets error when any API call fails', async () => {
		(api.analytics.summary as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('503 Service Unavailable')
		);
		await store.loadDashboard();
		expect(get(store.error)).toBe('503 Service Unavailable');
		expect(get(store.loading)).toBe(false);
	});

	it('sets generic error for non-Error rejections', async () => {
		(api.analytics.summary as ReturnType<typeof vi.fn>).mockRejectedValueOnce('timeout');
		await store.loadDashboard();
		expect(get(store.error)).toBe('Failed to load analytics');
	});

	it('still sets loading to false after error', async () => {
		(api.analytics.followers as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Network')
		);
		await store.loadDashboard();
		expect(get(store.loading)).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// Derived: followerCount
// ---------------------------------------------------------------------------

describe('followerCount', () => {
	it('returns followers.current from summary', () => {
		store.summary.set(mockSummary);
		expect(get(store.followerCount)).toBe(1500);
	});

	it('returns 0 when summary is null', () => {
		store.summary.set(null);
		expect(get(store.followerCount)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// Derived: followerChange7d
// ---------------------------------------------------------------------------

describe('followerChange7d', () => {
	it('returns followers.change_7d from summary', () => {
		store.summary.set(mockSummary);
		expect(get(store.followerChange7d)).toBe(50);
	});

	it('returns 0 when summary is null', () => {
		store.summary.set(null);
		expect(get(store.followerChange7d)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// Derived: repliesToday / tweetsToday
// ---------------------------------------------------------------------------

describe('repliesToday', () => {
	it('returns actions_today.replies', () => {
		store.summary.set(mockSummary);
		expect(get(store.repliesToday)).toBe(4);
	});

	it('returns 0 for null summary', () => {
		store.summary.set(null);
		expect(get(store.repliesToday)).toBe(0);
	});
});

describe('tweetsToday', () => {
	it('returns actions_today.tweets', () => {
		store.summary.set(mockSummary);
		expect(get(store.tweetsToday)).toBe(1);
	});

	it('returns 0 for null summary', () => {
		store.summary.set(null);
		expect(get(store.tweetsToday)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// Derived: avgEngagement
// ---------------------------------------------------------------------------

describe('avgEngagement', () => {
	it('averages reply and tweet scores when both are non-zero', () => {
		store.summary.set(mockSummary); // reply=0.80, tweet=0.70
		expect(get(store.avgEngagement)).toBeCloseTo(0.75);
	});

	it('returns reply score when tweet score is 0', () => {
		store.summary.set({
			...mockSummary,
			engagement: { ...mockSummary.engagement, avg_tweet_score: 0 }
		});
		expect(get(store.avgEngagement)).toBe(0.80);
	});

	it('returns tweet score when reply score is 0', () => {
		store.summary.set({
			...mockSummary,
			engagement: { ...mockSummary.engagement, avg_reply_score: 0 }
		});
		expect(get(store.avgEngagement)).toBe(0.70);
	});

	it('returns 0 when both scores are 0', () => {
		store.summary.set({
			...mockSummary,
			engagement: { ...mockSummary.engagement, avg_reply_score: 0, avg_tweet_score: 0 }
		});
		expect(get(store.avgEngagement)).toBe(0);
	});

	it('returns 0 when summary is null', () => {
		store.summary.set(null);
		expect(get(store.avgEngagement)).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// Auto-refresh lifecycle
// ---------------------------------------------------------------------------

describe('auto-refresh', () => {
	it('startAutoRefresh / stopAutoRefresh do not throw', () => {
		expect(() => store.startAutoRefresh(5000)).not.toThrow();
		expect(() => store.stopAutoRefresh()).not.toThrow();
	});

	it('stopAutoRefresh is idempotent when not running', () => {
		expect(() => {
			store.stopAutoRefresh();
			store.stopAutoRefresh();
		}).not.toThrow();
	});
});
