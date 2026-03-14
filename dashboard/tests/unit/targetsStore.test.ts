/**
 * targetsStore.test.ts — Unit tests for src/lib/stores/targets.ts
 *
 * Covers: loadTargets, addTarget, removeTarget, startAutoRefresh,
 * stopAutoRefresh, and derived stores (targetCount, repliesToday).
 * Branch coverage target: ≥70%.
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

// --- Fixtures ----------------------------------------------------------------

function makeTarget(username: string, interactionsToday = 0) {
	return {
		account_id: `acc_${username}`,
		username,
		followed_at: null,
		first_engagement_at: null,
		total_replies_sent: 2,
		last_reply_at: null,
		status: 'active',
		interactions_today: interactionsToday
	};
}

const TARGETS = [makeTarget('alice', 3), makeTarget('bob', 1)];

// --- beforeEach reset --------------------------------------------------------

beforeEach(() => {
	vi.clearAllMocks();
	store.targets.set([]);
	store.loading.set(true);
	store.error.set(null);
	(api.targets.list as ReturnType<typeof vi.fn>).mockResolvedValue(TARGETS);
});

// ---------------------------------------------------------------------------
// loadTargets
// ---------------------------------------------------------------------------

describe('loadTargets', () => {
	it('sets targets on success', async () => {
		await store.loadTargets();
		expect(get(store.targets)).toEqual(TARGETS);
	});

	it('sets loading=false on success', async () => {
		await store.loadTargets();
		expect(get(store.loading)).toBe(false);
	});

	it('clears error on success', async () => {
		store.error.set('old error');
		await store.loadTargets();
		expect(get(store.error)).toBeNull();
	});

	it('sets error and loading=false on failure', async () => {
		(api.targets.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Network error')
		);
		await store.loadTargets();
		expect(get(store.error)).toMatch(/Network error/);
		expect(get(store.loading)).toBe(false);
	});

	it('sets generic error message when thrown value is not an Error', async () => {
		(api.targets.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce('string error');
		await store.loadTargets();
		expect(get(store.error)).toBe('Failed to load targets');
	});
});

// ---------------------------------------------------------------------------
// addTarget
// ---------------------------------------------------------------------------

describe('addTarget', () => {
	it('returns null on success and reloads targets', async () => {
		(api.targets.add as ReturnType<typeof vi.fn>).mockResolvedValueOnce(undefined);
		const result = await store.addTarget('charlie');
		expect(result).toBeNull();
		expect(api.targets.list).toHaveBeenCalled();
	});

	it('returns error message on failure', async () => {
		(api.targets.add as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Already tracking')
		);
		const result = await store.addTarget('dave');
		expect(result).toBe('Already tracking');
	});

	it('returns generic message when thrown value is not an Error', async () => {
		(api.targets.add as ReturnType<typeof vi.fn>).mockRejectedValueOnce('bad');
		const result = await store.addTarget('eve');
		expect(result).toBe('Failed to add target');
	});
});

// ---------------------------------------------------------------------------
// removeTarget
// ---------------------------------------------------------------------------

describe('removeTarget', () => {
	beforeEach(async () => {
		await store.loadTargets();
	});

	it('removes target from store optimistically', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockResolvedValueOnce(undefined);
		await store.removeTarget('alice');
		expect(get(store.targets).map((t) => t.username)).not.toContain('alice');
	});

	it('returns null on success', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockResolvedValueOnce(undefined);
		const result = await store.removeTarget('alice');
		expect(result).toBeNull();
	});

	it('sets error and returns message on failure', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Remove failed')
		);
		const result = await store.removeTarget('alice');
		expect(result).toBe('Remove failed');
		expect(get(store.error)).toBe('Remove failed');
	});

	it('returns generic message when thrown value is not an Error', async () => {
		(api.targets.remove as ReturnType<typeof vi.fn>).mockRejectedValueOnce('oops');
		const result = await store.removeTarget('bob');
		expect(result).toBe('Failed to remove target');
	});
});

// ---------------------------------------------------------------------------
// startAutoRefresh / stopAutoRefresh
// ---------------------------------------------------------------------------

describe('startAutoRefresh / stopAutoRefresh', () => {
	it('startAutoRefresh starts a timer without crashing', () => {
		vi.useFakeTimers();
		expect(() => store.startAutoRefresh(1000)).not.toThrow();
		vi.useRealTimers();
	});

	it('stopAutoRefresh clears the interval', () => {
		vi.useFakeTimers();
		store.startAutoRefresh(1000);
		expect(() => store.stopAutoRefresh()).not.toThrow();
		vi.useRealTimers();
	});

	it('stopAutoRefresh is safe to call when no interval running', () => {
		expect(() => store.stopAutoRefresh()).not.toThrow();
	});

	it('startAutoRefresh replaces existing interval (idempotent)', () => {
		vi.useFakeTimers();
		store.startAutoRefresh(500);
		expect(() => store.startAutoRefresh(1000)).not.toThrow();
		store.stopAutoRefresh();
		vi.useRealTimers();
	});
});

// ---------------------------------------------------------------------------
// Derived: targetCount
// ---------------------------------------------------------------------------

describe('targetCount', () => {
	it('returns 0 when targets is empty', () => {
		store.targets.set([]);
		expect(get(store.targetCount)).toBe(0);
	});

	it('returns count of current targets', () => {
		store.targets.set(TARGETS);
		expect(get(store.targetCount)).toBe(2);
	});

	it('updates when targets changes', async () => {
		await store.loadTargets();
		expect(get(store.targetCount)).toBe(2);
	});
});

// ---------------------------------------------------------------------------
// Derived: repliesToday
// ---------------------------------------------------------------------------

describe('repliesToday', () => {
	it('returns 0 when targets is empty', () => {
		store.targets.set([]);
		expect(get(store.repliesToday)).toBe(0);
	});

	it('sums interactions_today across all targets', () => {
		store.targets.set(TARGETS); // alice=3, bob=1
		expect(get(store.repliesToday)).toBe(4);
	});

	it('updates reactively when targets change', async () => {
		await store.loadTargets();
		expect(get(store.repliesToday)).toBe(4);
	});

	it('handles single target', () => {
		store.targets.set([makeTarget('alice', 7)]);
		expect(get(store.repliesToday)).toBe(7);
	});
});
