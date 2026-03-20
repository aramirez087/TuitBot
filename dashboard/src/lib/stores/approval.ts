import { writable, derived, get } from 'svelte/store';
import { api, type ApprovalItem, type ApprovalStats } from '$lib/api';
import { events as wsEvents } from './websocket';

// --- Writable stores ---

export const items = writable<ApprovalItem[]>([]);
export const stats = writable<ApprovalStats | null>(null);
export const loading = writable(true);
export const error = writable<string | null>(null);
export const selectedStatus = writable('pending');
export const selectedType = writable('all');
export const reviewerFilter = writable('');
export const dateFilter = writable('all');
export const focusedIndex = writable(0);
export const currentPage = writable(1);
export const pageSize = writable(20);
export const searchQuery = writable('');

// --- Derived stores ---

export const focusedItem = derived([items, focusedIndex], ([$items, $idx]) => $items[$idx] ?? null);
export const isEmpty = derived([items, loading], ([$items, $loading]) => !$loading && $items.length === 0);
export const pendingCount = derived(stats, ($s) => $s?.pending ?? 0);

// Search + filter items
const searchedItems = derived([items, searchQuery], ([$items, $search]) => {
	if (!$search.trim()) return $items;
	const q = $search.toLowerCase();
	return $items.filter(
		(i) =>
			i.generated_content.toLowerCase().includes(q) ||
			i.target_author.toLowerCase().includes(q) ||
			i.topic.toLowerCase().includes(q)
	);
});

// Paginate searched items
export const paginatedItems = derived(
	[searchedItems, currentPage, pageSize],
	([$searched, $page, $size]) => {
		const start = ($page - 1) * $size;
		return $searched.slice(start, start + $size);
	}
);

export const totalCount = derived(searchedItems, ($s) => $s.length);
export const totalPages = derived([totalCount, pageSize], ([$count, $size]) =>
	Math.ceil($count / $size)
);

// --- Data loading ---

export async function loadItems(reset = false) {
	if (reset) {
		focusedIndex.set(0);
	}

	loading.set(true);
	error.set(null);

	try {
		const status = get(selectedStatus);
		const type = get(selectedType);
		const reviewer = get(reviewerFilter);
		const dateFilt = get(dateFilter);

		// Handle "failed" status: maps to action_type=failed_post_recovery&status=pending
		let statusParam: string | undefined;
		let actionTypeParam: string | undefined;

		if (status === 'failed') {
			statusParam = 'pending';
			actionTypeParam = 'failed_post_recovery';
		} else {
			statusParam = status === 'all' ? 'pending,approved,rejected' : status;
		}

		const typeParam = type === 'all' ? undefined : type;
		const reviewerParam = reviewer.trim() || undefined;

		let sinceParam: string | undefined;
		if (dateFilt !== 'all') {
			const now = new Date();
			const hours = dateFilt === '24h' ? 24 : dateFilt === '7d' ? 168 : 720;
			sinceParam = new Date(now.getTime() - hours * 3600_000).toISOString();
		}

		const data = await api.approval.list({
			status: statusParam,
			type: typeParam,
			reviewed_by: reviewerParam,
			since: sinceParam,
			action_type: actionTypeParam,
		});
		items.set(data);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load approval items');
	} finally {
		loading.set(false);
	}
}

export async function loadStats() {
	try {
		const data = await api.approval.stats();
		stats.set(data);
	} catch {
		// Silent fail on stats refresh
	}
}

export async function approveItem(id: number) {
	try {
		await api.approval.approve(id, 'dashboard');
		items.update(($items) => $items.filter((i) => i.id !== id));
		focusedIndex.update(($idx) => {
			const len = get(items).length;
			return len === 0 ? 0 : Math.min($idx, len - 1);
		});
		await loadStats();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to approve item');
	}
}

export async function rejectItem(id: number, notes?: string) {
	try {
		await api.approval.reject(id, 'dashboard', notes);
		items.update(($items) => $items.filter((i) => i.id !== id));
		focusedIndex.update(($idx) => {
			const len = get(items).length;
			return len === 0 ? 0 : Math.min($idx, len - 1);
		});
		await loadStats();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to reject item');
	}
}

export async function editItem(id: number, content: string, media_paths?: string[]) {
	try {
		const updated = await api.approval.edit(id, content, media_paths);
		items.update(($items) => $items.map((i) => (i.id === id ? updated : i)));
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to edit item');
	}
}

export async function approveAllItems() {
	try {
		await api.approval.approveAll();
		items.set([]);
		focusedIndex.set(0);
		await loadStats();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to approve all items');
	}
}

export function setStatusFilter(status: string) {
	selectedStatus.set(status);
	currentPage.set(1);
	loadItems(true);
}

export function setTypeFilter(type: string) {
	selectedType.set(type);
	currentPage.set(1);
	loadItems(true);
}

export function setReviewerFilter(reviewer: string) {
	reviewerFilter.set(reviewer);
	currentPage.set(1);
	loadItems(true);
}

export function setDateFilter(range: string) {
	dateFilter.set(range);
	currentPage.set(1);
	loadItems(true);
}

export function setCurrentPage(page: number) {
	currentPage.set(Math.max(1, page));
}

export function setSearchQuery(q: string) {
	searchQuery.set(q);
	currentPage.set(1);
}

export function moveFocus(delta: number) {
	const len = get(items).length;
	if (len === 0) return;
	focusedIndex.update(($idx) => Math.max(0, Math.min(len - 1, $idx + delta)));
}

// --- WebSocket integration ---

let lastEventCount = 0;

wsEvents.subscribe(($events) => {
	if ($events.length === 0 || $events.length === lastEventCount) return;
	lastEventCount = $events.length;

	const latest = $events[0];

	if (latest.type === 'ApprovalQueued') {
		// Reload stats to keep counts fresh.
		loadStats();

		// If viewing pending items, reload the list to get the new item.
		if (get(selectedStatus) === 'pending' || get(selectedStatus) === 'all') {
			loadItems();
		}
	}

	if (latest.type === 'ApprovalUpdated') {
		loadStats();

		const updatedId = latest.id as number;
		const updatedStatus = latest.status as string;

		// If the current filter wouldn't show this status, remove the item.
		// "scheduled" is a terminal status like "approved" — remove from pending view.
		const currentFilter = get(selectedStatus);
		if (currentFilter !== 'all' && currentFilter !== updatedStatus) {
			items.update(($items) => $items.filter((i) => i.id !== updatedId));
		} else {
			// Update the item status in place.
			items.update(($items) =>
				$items.map((i) => (i.id === updatedId ? { ...i, status: updatedStatus } : i))
			);
		}
	}
});

// --- Auto-refresh for stats ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 30_000) {
	stopAutoRefresh();
	refreshInterval = setInterval(() => {
		loadStats();
	}, intervalMs);
}

export function stopAutoRefresh() {
	if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = null;
	}
}

// --- Account switching integration ---

// When user switches accounts, refetch all approval data for the new account.
if (typeof window !== 'undefined') {
	window.addEventListener('tuitbot:account-switched', () => {
		focusedIndex.set(0);
		currentPage.set(1);
		loadItems(true);
		loadStats();
	});
}
