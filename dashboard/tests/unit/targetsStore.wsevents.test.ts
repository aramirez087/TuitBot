/**
 * targetsStore.wsevents.test.ts — Tests for the module-level wsEvents
 * subscription branches in targets.ts (lines 63-71).
 *
 * Uses vi.hoisted() to capture the subscriber callback.
 * Same `lastEventCount` persistent-state issue as approvalStore — uses
 * auto-incrementing dispatch helper.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// --- Capture the wsEvents callback via vi.hoisted() -------------------------

const captured = vi.hoisted(() => ({ cb: null as ((v: unknown[]) => void) | null }));

const mockApi = vi.hoisted(() => ({
	targets: {
		list: vi.fn().mockResolvedValue([]),
		add: vi.fn().mockResolvedValue(undefined),
		remove: vi.fn().mockResolvedValue(undefined)
	}
}));

vi.mock('$lib/api', () => ({ api: mockApi }));

vi.mock('$lib/stores/websocket', () => ({
	events: {
		subscribe: vi.fn((cb: (v: unknown[]) => void) => {
			captured.cb = cb;
			cb([]); // initial: length=0, lastEventCount stays 0
			return () => {};
		})
	}
}));

// --- Imports after mocks ----------------------------------------------------

import * as store from '../../src/lib/stores/targets';

// --- Fixtures ----------------------------------------------------------------

function makeTarget(username: string) {
	return {
		account_id: `acc_${username}`,
		username,
		followed_at: null,
		first_engagement_at: null,
		total_replies_sent: 2,
		last_reply_at: null,
		status: 'active',
		interactions_today: 0
	};
}

// --- Dispatch helper ---------------------------------------------------------

let dispatchSeq = 0;

function dispatch(event: Record<string, unknown>) {
	dispatchSeq++;
	const arr: unknown[] = [event, ...Array(dispatchSeq - 1).fill({ type: '_pad' })];
	captured.cb?.(arr);
}

// ---------------------------------------------------------------------------
// WebSocket event branches
// ---------------------------------------------------------------------------

describe('wsEvents subscription branches', () => {
	beforeEach(() => {
		mockApi.targets.list.mockClear();
		store.targets.set([]);
		store.loading.set(false);
		store.error.set(null);
		// dispatchSeq stays in sync with module's lastEventCount
	});

	it('reloads when ActionPerformed target IS in the targets store', () => {
		store.targets.set([makeTarget('alice')]);
		dispatch({ type: 'ActionPerformed', target: 'alice' });
		expect(mockApi.targets.list).toHaveBeenCalled();
	});

	it('does NOT reload when ActionPerformed target is NOT in store', () => {
		store.targets.set([makeTarget('alice')]);
		mockApi.targets.list.mockClear();
		dispatch({ type: 'ActionPerformed', target: 'bob' });
		expect(mockApi.targets.list).not.toHaveBeenCalled();
	});

	it('does NOT reload when ActionPerformed target field is undefined', () => {
		store.targets.set([makeTarget('alice')]);
		mockApi.targets.list.mockClear();
		dispatch({ type: 'ActionPerformed', target: undefined });
		expect(mockApi.targets.list).not.toHaveBeenCalled();
	});

	it('ignores unknown event types', () => {
		mockApi.targets.list.mockClear();
		dispatch({ type: 'SomeOtherEvent' });
		expect(mockApi.targets.list).not.toHaveBeenCalled();
	});
});
