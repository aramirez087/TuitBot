import { writable, derived } from 'svelte/store';
import {
	api,
	type AnalyticsSummary,
	type FollowerSnapshot,
	type PerformanceItem
} from '$lib/api';

// --- Writable stores ---

export const summary = writable<AnalyticsSummary | null>(null);
export const followerSnapshots = writable<FollowerSnapshot[]>([]);
export const recentPerformance = writable<PerformanceItem[]>([]);
export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Derived stores for stat cards ---

export const followerCount = derived(summary, ($s) => $s?.followers.current ?? 0);
export const followerChange7d = derived(summary, ($s) => $s?.followers.change_7d ?? 0);
export const repliesToday = derived(summary, ($s) => $s?.actions_today.replies ?? 0);
export const tweetsToday = derived(summary, ($s) => $s?.actions_today.tweets ?? 0);
export const avgEngagement = derived(summary, ($s) => {
	if (!$s) return 0;
	const { avg_reply_score, avg_tweet_score } = $s.engagement;
	if (avg_reply_score === 0 && avg_tweet_score === 0) return 0;
	if (avg_reply_score === 0) return avg_tweet_score;
	if (avg_tweet_score === 0) return avg_reply_score;
	return (avg_reply_score + avg_tweet_score) / 2;
});

// --- Data loading ---

export async function loadDashboard(days: number = 30) {
	loading.set(true);
	error.set(null);

	try {
		const [summaryData, snapshots, performance] = await Promise.all([
			api.analytics.summary(),
			api.analytics.followers(days),
			api.analytics.recentPerformance(20)
		]);

		summary.set(summaryData);
		followerSnapshots.set(snapshots);
		recentPerformance.set(performance);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load analytics');
	} finally {
		loading.set(false);
	}
}

// --- Auto-refresh ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 60_000) {
	stopAutoRefresh();
	refreshInterval = setInterval(async () => {
		try {
			const data = await api.analytics.summary();
			summary.set(data);
		} catch {
			// Silent fail on background refresh — don't disrupt the UI
		}
	}, intervalMs);
}

export function stopAutoRefresh() {
	if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = null;
	}
}

// --- Account switching integration ---

// When user switches accounts, refetch all analytics data for the new account.
if (typeof window !== 'undefined') {
	window.addEventListener('tuitbot:account-switched', () => {
		loadDashboard();
	});
}
