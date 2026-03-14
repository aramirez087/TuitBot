/**
 * mockStores.ts — Pre-configured Svelte store mocks for unit and component tests.
 *
 * Usage:
 *   import { mockApprovalStore, resetAllStores } from '../helpers/mockStores';
 *   beforeEach(() => resetAllStores());
 */

import { writable, get } from 'svelte/store';
import { vi } from 'vitest';
import type { ApprovalItem, ApprovalStats } from '../../src/lib/api/types';
import { fixtures } from './fixtures';

// ---------------------------------------------------------------------------
// approval store mock
// ---------------------------------------------------------------------------

export const mockApprovalStore = {
	items: writable<ApprovalItem[]>(fixtures.approvalItems),
	stats: writable<ApprovalStats | null>(fixtures.approvalStats),
	loading: writable(false),
	error: writable<string | null>(null),
	selectedStatus: writable('pending'),
	selectedType: writable('all'),
	reviewerFilter: writable(''),
	dateFilter: writable('all'),
	focusedIndex: writable(0),
	loadItems: vi.fn().mockResolvedValue(undefined),
	approveItem: vi.fn().mockResolvedValue(undefined),
	rejectItem: vi.fn().mockResolvedValue(undefined),
	editItem: vi.fn().mockResolvedValue(undefined)
};

// ---------------------------------------------------------------------------
// analytics store mock
// ---------------------------------------------------------------------------

export const mockAnalyticsStore = {
	summary: writable(fixtures.analyticsSummary),
	loading: writable(false),
	error: writable<string | null>(null),
	load: vi.fn().mockResolvedValue(undefined)
};

// ---------------------------------------------------------------------------
// settings store mock
// ---------------------------------------------------------------------------

export const mockSettingsStore = {
	config: writable(fixtures.config),
	loading: writable(false),
	saving: writable(false),
	error: writable<string | null>(null),
	load: vi.fn().mockResolvedValue(undefined),
	save: vi.fn().mockResolvedValue(undefined)
};

// ---------------------------------------------------------------------------
// composer store mock
// ---------------------------------------------------------------------------

export const mockComposerStore = {
	tweets: writable([{ id: 'tweet-1', text: '', media: [] }]),
	mode: writable<'single' | 'thread'>('single'),
	isOpen: writable(false),
	isSaving: writable(false),
	activeIndex: writable(0),
	open: vi.fn(),
	close: vi.fn(),
	addTweet: vi.fn(),
	removeTweet: vi.fn(),
	updateTweet: vi.fn(),
	submit: vi.fn().mockResolvedValue(undefined)
};

// ---------------------------------------------------------------------------
// discovery store mock
// ---------------------------------------------------------------------------

export const mockDiscoveryStore = {
	targets: writable(fixtures.targets),
	loading: writable(false),
	error: writable<string | null>(null),
	load: vi.fn().mockResolvedValue(undefined)
};

// ---------------------------------------------------------------------------
// Reset utility
// ---------------------------------------------------------------------------

export function resetAllStores(): void {
	mockApprovalStore.items.set(fixtures.approvalItems);
	mockApprovalStore.stats.set(fixtures.approvalStats);
	mockApprovalStore.loading.set(false);
	mockApprovalStore.error.set(null);
	mockApprovalStore.selectedStatus.set('pending');
	mockApprovalStore.selectedType.set('all');
	mockApprovalStore.focusedIndex.set(0);

	mockAnalyticsStore.loading.set(false);
	mockAnalyticsStore.error.set(null);

	mockSettingsStore.loading.set(false);
	mockSettingsStore.saving.set(false);
	mockSettingsStore.error.set(null);

	mockComposerStore.tweets.set([{ id: 'tweet-1', text: '', media: [] }]);
	mockComposerStore.isOpen.set(false);
	mockComposerStore.isSaving.set(false);
	mockComposerStore.activeIndex.set(0);

	mockDiscoveryStore.loading.set(false);
	mockDiscoveryStore.error.set(null);

	vi.clearAllMocks();
}
