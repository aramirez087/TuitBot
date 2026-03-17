import { writable, derived, get } from 'svelte/store';
import { api, type ActionLogEntry, type RateLimitUsage } from '$lib/api';
import { events as wsEvents } from './websocket';

const PAGE_SIZE = 50;

// --- Filter category → API param mapping ---

interface ApiFilter {
	type?: string;
	status?: string;
}

const FILTER_MAP: Record<string, ApiFilter> = {
	all: {},
	search: { type: 'search' },
	reply: { type: 'reply' },
	tweet: { type: 'tweet' },
	errors: { status: 'failure' }
};

// --- Writable stores ---

export const actions = writable<ActionLogEntry[]>([]);
export const rateLimits = writable<RateLimitUsage | null>(null);
export const loading = writable(true);
export const error = writable<string | null>(null);
export const totalCount = writable(0);
export const currentOffset = writable(0);
export const selectedFilter = writable('all');

// --- Derived stores ---

export const hasMore = derived(
	[currentOffset, totalCount],
	([$offset, $total]) => $offset + PAGE_SIZE < $total
);

// --- Data loading ---

export async function loadActivity(reset = false) {
	if (reset) {
		currentOffset.set(0);
	}

	loading.set(true);
	error.set(null);

	try {
		const offset = get(currentOffset);
		const filter = FILTER_MAP[get(selectedFilter)] ?? {};

		const [activityData, limits] = await Promise.all([
			api.activity.list({ limit: PAGE_SIZE, offset, ...filter }),
			api.activity.rateLimits()
		]);

		actions.set(activityData.actions);
		totalCount.set(activityData.total);
		rateLimits.set(limits);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load activity');
	} finally {
		loading.set(false);
	}
}

export async function loadMore() {
	const offset = get(currentOffset) + PAGE_SIZE;
	currentOffset.set(offset);

	error.set(null);

	try {
		const filter = FILTER_MAP[get(selectedFilter)] ?? {};
		const data = await api.activity.list({ limit: PAGE_SIZE, offset, ...filter });

		actions.update(($actions) => [...$actions, ...data.actions]);
		totalCount.set(data.total);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load more activity');
		// Revert offset on failure
		currentOffset.set(offset - PAGE_SIZE);
	}
}

export function setFilter(category: string) {
	selectedFilter.set(category);
	loadActivity(true);
}

// --- WebSocket integration ---

let lastEventCount = 0;

wsEvents.subscribe(($events) => {
	if ($events.length === 0 || $events.length === lastEventCount) return;
	lastEventCount = $events.length;

	const latest = $events[0];
	if (get(currentOffset) !== 0) return;
	if (get(selectedFilter) !== 'all') return;

	if (latest.type === 'ActionPerformed') {
		const entry: ActionLogEntry = {
			id: -Date.now(),
			action_type: (latest as Record<string, unknown>).action_type as string || 'unknown',
			status: 'success',
			message: (latest as Record<string, unknown>).content as string || null,
			metadata: null,
			created_at: (latest as Record<string, unknown>).timestamp as string || new Date().toISOString()
		};

		actions.update(($a) => [entry, ...$a].slice(0, PAGE_SIZE));
		totalCount.update((t) => t + 1);
	}
});

// --- Auto-refresh for rate limits ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 30_000) {
	stopAutoRefresh();
	refreshInterval = setInterval(async () => {
		try {
			const limits = await api.activity.rateLimits();
			rateLimits.set(limits);
		} catch {
			// Silent fail on background refresh
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

// When user switches accounts, refetch all activity for the new account.
if (typeof window !== 'undefined') {
	window.addEventListener('tuitbot:account-switched', () => {
		currentOffset.set(0);
		selectedFilter.set('all');
		loadActivity(true);
	});
}
