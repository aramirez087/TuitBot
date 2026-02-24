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
export const focusedIndex = writable(0);

// --- Derived stores ---

export const focusedItem = derived([items, focusedIndex], ([$items, $idx]) => $items[$idx] ?? null);
export const isEmpty = derived([items, loading], ([$items, $loading]) => !$loading && $items.length === 0);
export const pendingCount = derived(stats, ($s) => $s?.pending ?? 0);

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

		const statusParam = status === 'all' ? 'pending,approved,rejected' : status;
		const typeParam = type === 'all' ? undefined : type;

		const data = await api.approval.list({ status: statusParam, type: typeParam });
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
		await api.approval.approve(id);
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

export async function rejectItem(id: number) {
	try {
		await api.approval.reject(id);
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

export async function editItem(id: number, content: string) {
	try {
		const updated = await api.approval.edit(id, content);
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
	loadItems(true);
}

export function setTypeFilter(type: string) {
	selectedType.set(type);
	loadItems(true);
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
