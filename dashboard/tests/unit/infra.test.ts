/**
 * infra.test.ts — Smoke tests to verify the test infrastructure itself is
 * wired correctly. These don't test application logic.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { fixtures } from '../helpers/fixtures';
import { mockApprovalStore, mockAnalyticsStore, resetAllStores } from '../helpers/mockStores';
import { mockApi, resetMockApi } from '../helpers/mockApi';
import { assertStoreValue, apiError } from '../helpers/testHelpers';

beforeEach(() => {
	resetAllStores();
	resetMockApi();
});

describe('Test infrastructure', () => {
	it('fixtures are defined and have expected shapes', () => {
		expect(fixtures.approvalItems).toHaveLength(4);
		expect(fixtures.approvalItems[0].status).toBe('pending');
		expect(fixtures.approvalStats.pending).toBe(2);
		expect(fixtures.analyticsSummary.followers.current).toBe(1250);
		expect(fixtures.targets).toHaveLength(3);
	});

	it('approvalItem factory applies overrides', () => {
		const item = fixtures.approvalItem({ status: 'approved', score: 0.99 });
		expect(item.status).toBe('approved');
		expect(item.score).toBe(0.99);
	});

	it('targetAccount factory applies overrides', () => {
		const t = fixtures.targetAccount({ username: 'custom_user', follower_count: 99 });
		expect(t.username).toBe('custom_user');
		expect(t.follower_count).toBe(99);
	});

	it('mockApprovalStore initialises with fixture data', () => {
		assertStoreValue(mockApprovalStore.loading, false);
		assertStoreValue(mockApprovalStore.error, null);
		const items = get(mockApprovalStore.items);
		expect(items).toHaveLength(4);
	});

	it('mockAnalyticsStore initialises correctly', () => {
		assertStoreValue(mockAnalyticsStore.loading, false);
		assertStoreValue(mockAnalyticsStore.error, null);
	});

	it('resetAllStores restores default state', () => {
		mockApprovalStore.loading.set(true);
		mockApprovalStore.error.set('boom');
		mockApprovalStore.focusedIndex.set(3);
		resetAllStores();
		assertStoreValue(mockApprovalStore.loading, false);
		assertStoreValue(mockApprovalStore.error, null);
		assertStoreValue(mockApprovalStore.focusedIndex, 0);
	});

	it('mockApi returns fixture data', async () => {
		const items = await mockApi.approval.list();
		expect(items).toHaveLength(4);

		const stats = await mockApi.approval.stats();
		expect(stats.pending).toBe(2);

		const summary = await mockApi.analytics.summary();
		expect(summary.followers.current).toBe(1250);
	});

	it('mockApi functions are vi.fn() spies', () => {
		expect(vi.isMockFunction(mockApi.approval.list)).toBe(true);
		expect(vi.isMockFunction(mockApi.approval.approve)).toBe(true);
		expect(vi.isMockFunction(mockApi.analytics.summary)).toBe(true);
	});

	it('mockApi can simulate errors', async () => {
		mockApi.approval.list.mockRejectedValueOnce(apiError('Network failure', 503));
		await expect(mockApi.approval.list()).rejects.toThrow('Network failure');
		// Next call returns fixture data again
		const items = await mockApi.approval.list();
		expect(items).toHaveLength(4);
	});

	it('resetMockApi clears call counts', async () => {
		await mockApi.health();
		await mockApi.health();
		expect(mockApi.health).toHaveBeenCalledTimes(2);
		resetMockApi();
		expect(mockApi.health).toHaveBeenCalledTimes(0);
	});
});
