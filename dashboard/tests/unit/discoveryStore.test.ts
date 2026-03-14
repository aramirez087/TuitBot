/**
 * discoveryStore.test.ts — Unit tests for src/lib/stores/targets.ts
 *
 * Covers: loadTargets, addTarget, removeTarget (optimistic),
 * and derived stores (targetCount, repliesToday).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		targets: {
			list: vi.fn(),
			add: vi.fn(),
			remove: vi.fn()
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
import * as store from '../../src/lib/stores/targets';
import type { TargetAccount } from '../../src/lib/api/types';

// --- Fixtures ---------------------------------------------------------------

const makeTarget = (username: string, overrides: Partial<TargetAccount> = {}): TargetAccount => ({
	account_id: `id_${username}`,
	username,
	followed_at: '2026-01-01T00:00:00.000Z',
	first_engagement_at: '2026-01-02T00:00:00.000Z',
	total_replies_sent: 3,
	last_reply_at: '2026-03-01T00:00:00.000Z',
	status: 'active',
	interactions_today: 1,
	...overrides
});

const TARGETS: TargetAccount[] = [
	makeTarget('alice', { interactions_today: 2 }),
	makeTarget('bob', { interactions_today: 1 }),
	makeTarget('carol', { interactions_today: 0, status: 'inactive' })
];

// --- Reset helper -----------------------------------------------------------

function resetStores() {
	store.targets.set([]);
	store.loading.set(false);
	store.error.set(null);
}

// --- Tests ------------------------------------------------------------------

beforeEach(() => {
	resetStores();
	vi.clearAllMocks();
	(api.targets.list as ReturnType<typeof vi.fn>).mockResolvedValue(TARGETS);
	(api.targets.add as ReturnType<typeof vi.fn>).mockResolvedValue({ status: 'ok', username: 'newuser' });
	(api.targets.remove as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true });
});

// ---------------------------------------------------------------------------
// loadTargets
// ---------------------------------------------------------------------------

describe('loadTargets', () => {
	it('sets loading true then false', async () => {
		const states: boolean[] = [];
		const unsub = store.loading.subscribe((v) => states.push(v));
		await store.loadTargets();
		unsub();
		expect(states).toContain(true);
		expect(states[states.length - 1]).toBe(false);
	});

	it('populates targets on success', async () => {
		await store.loadTargets();
		expect(get(store.targets)).toHaveLength(3);
		expect(get(store.targets)[0].username).toBe('alice');
	});

	it('clears error before fetching', async () => {
		store.error.set('old error');
		await store.loadTargets();
		expect(get(store.error)).toBeNull();
	});

	it('sets error on API failure', async () => {
		(api.targets.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Connection refused')
		);
		await store.loadTargets();
		expect(get(store.error)).toBe('Connection refused');
		expect(get(store.loading)).toBe(false);
	});

	it('sets generic error for non-Error rejections', async () => {
		(api.targets.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce('timeout');
		await store.loadTargets();
		expect(get(store.error)).toBe('Failed to load targets');
	});

	it('always sets loading to false after API failure', async () => {
		(api.targets.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('oops'));
		await store.loadTargets();
		expect(get(store.loading)).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// addTarget
// ---------------------------------------------------------------------------

describe('addTarget', () => {
	beforeEach(async () => {
		await store.loadTargets();
	});

	it('calls targets.add with username', async () => {
		await store.addTarget('newuser');
		expect(api.targets.add).toHaveBeenCalledWith('newuser');
	});

	it('reloads targets after adding', async () => {
		await store.addTarget('newuser');
		// loadTargets called once during beforeEach, once after add
		expect(api.targets.list).toHaveBeenCalledTimes(2);
	});

	it('returns null on success', async () => {
		const result = await store.addTarget('newuser');
		expect(result).toBeNull();
	});

	it('returns error message on API failure', async () => {
		(api.targets.add as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Username not found')
		);
		const result = await store.addTarget('nonexistent');
		expect(result).toBe('Username not found');
	});

	it('returns generic message for non-Error rejections', async () => {
		(api.targets.add as ReturnType<typeof vi.fn>).mockRejectedValueOnce('oops');
		const result = await store.addTarget('x');
		expect(result).toBe('Failed to add target');
	});
});

// ---------------------------------------------------------------------------
// removeTarget
// ---------------------------------------------------------------------------

describe('removeTarget', () => {
	beforeEach(async () => {
		await store.loadTargets();
		vi.clearAllMocks(); // reset call counts after setup
		(api.targets.remove as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true });
	});

	it('removes the target from the list optimistically', async () => {
		await store.removeTarget('alice');
		expect(get(store.targets).find((t) => t.username === 'alice')).toBeUndefined();
	});

	it('calls targets.remove with the username', async () => {
		await store.removeTarget('bob');
		expect(api.targets.remove).toHaveBeenCalledWith('bob');
	});

	it('returns null on success', async () => {
		const result = await store.removeTarget('alice');
		expect(result).toBeNull();
	});

	it('returns error message and sets error store on failure', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Not found')
		);
		const result = await store.removeTarget('alice');
		expect(result).toBe('Not found');
		expect(get(store.error)).toBe('Not found');
	});

	it('returns generic message for non-Error rejections', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockRejectedValueOnce('nope');
		const result = await store.removeTarget('alice');
		expect(result).toBe('Failed to remove target');
	});

	it('does not affect other targets when removing one', async () => {
		await store.removeTarget('alice');
		const remaining = get(store.targets);
		expect(remaining.find((t) => t.username === 'bob')).toBeDefined();
		expect(remaining.find((t) => t.username === 'carol')).toBeDefined();
	});
});

// ---------------------------------------------------------------------------
// Derived: targetCount
// ---------------------------------------------------------------------------

describe('targetCount', () => {
	it('returns number of targets', () => {
		store.targets.set(TARGETS);
		expect(get(store.targetCount)).toBe(3);
	});

	it('returns 0 when empty', () => {
		store.targets.set([]);
		expect(get(store.targetCount)).toBe(0);
	});

	it('updates reactively as targets change', async () => {
		await store.loadTargets();
		expect(get(store.targetCount)).toBe(3);
		store.targets.update((t) => t.filter((x) => x.username !== 'alice'));
		expect(get(store.targetCount)).toBe(2);
	});
});

// ---------------------------------------------------------------------------
// Derived: repliesToday
// ---------------------------------------------------------------------------

describe('repliesToday', () => {
	it('sums interactions_today across all targets', () => {
		store.targets.set(TARGETS); // alice=2, bob=1, carol=0
		expect(get(store.repliesToday)).toBe(3);
	});

	it('returns 0 for empty list', () => {
		store.targets.set([]);
		expect(get(store.repliesToday)).toBe(0);
	});

	it('updates when targets change', () => {
		store.targets.set([makeTarget('user1', { interactions_today: 5 })]);
		expect(get(store.repliesToday)).toBe(5);
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
