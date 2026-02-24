import { writable, derived, get } from 'svelte/store';
import { api, type TargetAccount } from '$lib/api';
import { events as wsEvents } from './websocket';

// --- Writable stores ---

export const targets = writable<TargetAccount[]>([]);
export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Derived stores ---

export const targetCount = derived(targets, ($t) => $t.length);
export const repliesToday = derived(targets, ($t) =>
	$t.reduce((sum, t) => sum + t.interactions_today, 0)
);

// --- Data loading ---

export async function loadTargets() {
	loading.set(true);
	error.set(null);

	try {
		const data = await api.targets.list();
		targets.set(data);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load targets');
	} finally {
		loading.set(false);
	}
}

export async function addTarget(username: string): Promise<string | null> {
	try {
		await api.targets.add(username);
		await loadTargets();
		return null;
	} catch (e) {
		const msg = e instanceof Error ? e.message : 'Failed to add target';
		return msg;
	}
}

export async function removeTarget(username: string): Promise<string | null> {
	try {
		await api.targets.remove(username);
		targets.update(($t) => $t.filter((t) => t.username !== username));
		return null;
	} catch (e) {
		const msg = e instanceof Error ? e.message : 'Failed to remove target';
		error.set(msg);
		return msg;
	}
}

// --- WebSocket integration ---

let lastEventCount = 0;

wsEvents.subscribe(($events) => {
	if ($events.length === 0 || $events.length === lastEventCount) return;
	lastEventCount = $events.length;

	const latest = $events[0];

	// When a target reply action is performed, refresh the list to update counts.
	if (latest.type === 'ActionPerformed') {
		const target = latest.target as string | undefined;
		if (target && get(targets).some((t) => t.username === target)) {
			loadTargets();
		}
	}
});

// --- Auto-refresh ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 30_000) {
	stopAutoRefresh();
	refreshInterval = setInterval(() => {
		loadTargets();
	}, intervalMs);
}

export function stopAutoRefresh() {
	if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = null;
	}
}
