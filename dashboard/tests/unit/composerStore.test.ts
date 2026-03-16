/**
 * composerStore.test.ts — Unit tests for src/lib/stores/draftStudio.svelte.ts
 *
 * draftStudio uses Svelte 5 `$state` / `$derived` runes compiled by
 * @sveltejs/vite-plugin-svelte.  Getter functions expose state values
 * so we test the exported API surface: getters, setters, and async actions.
 *
 * Note: module-level rune state is shared across tests in the same run
 * because ES modules are singletons.  We reset state via selectDraft(null),
 * setTab('active'), and setSearchQuery('') between tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		draftStudio: {
			list: vi.fn(),
			get: vi.fn(),
			create: vi.fn(),
			autosave: vi.fn(),
			archive: vi.fn(),
			restore: vi.fn(),
			delete: vi.fn(),
			schedule: vi.fn(),
			reschedule: vi.fn(),
			unschedule: vi.fn(),
			tags: vi.fn(),
			revisions: vi.fn(),
			activity: vi.fn(),
			restoreRevision: vi.fn(),
			duplicate: vi.fn()
		}
	}
}));

// --- Imports after mocks ----------------------------------------------------

import { api } from '$lib/api';
import * as composer from '../../src/lib/stores/draftStudio.svelte';
import type { DraftSummary, ScheduledContentItem } from '../../src/lib/api/types';

// --- Fixtures ---------------------------------------------------------------

const makeDraftSummary = (id: number, status: 'draft' | 'scheduled' | 'posted' = 'draft'): DraftSummary => ({
	id,
	status,
	content_type: 'tweet',
	content_preview: `Draft preview ${id}`,
	title: `Draft ${id}`,
	created_at: '2026-03-14T00:00:00.000Z',
	updated_at: '2026-03-14T01:00:00.000Z',
	scheduled_for: status === 'scheduled' ? '2026-03-20T10:00:00.000Z' : null,
	archived_at: null,
	source: 'manual'
});

const DRAFT_SUMMARIES: DraftSummary[] = [
	makeDraftSummary(1, 'draft'),
	makeDraftSummary(2, 'draft'),
	makeDraftSummary(3, 'scheduled'),
	makeDraftSummary(4, 'posted')
];

const makeFull = (id: number): ScheduledContentItem => ({
	id,
	status: 'draft',
	content_type: 'tweet',
	content: `Full content ${id}`,
	created_at: '2026-03-14T00:00:00.000Z',
	updated_at: '2026-03-14T01:00:00.000Z',
	source: 'manual',
	blocks: []
} as unknown as ScheduledContentItem);

// --- Reset helper -----------------------------------------------------------

function resetComposerState() {
	composer.selectDraft(null);
	composer.setTab('active');
	composer.setSearchQuery('');
	composer.setSortBy('updated');
	composer.setFullDraft(null);
}

// --- Tests ------------------------------------------------------------------

beforeEach(() => {
	resetComposerState();
	vi.clearAllMocks();
	(api.draftStudio.list as ReturnType<typeof vi.fn>).mockResolvedValue(DRAFT_SUMMARIES);
	(api.draftStudio.get as ReturnType<typeof vi.fn>).mockResolvedValue(makeFull(1));
	(api.draftStudio.create as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 99, updated_at: '2026-03-14T02:00:00.000Z' });
	(api.draftStudio.archive as ReturnType<typeof vi.fn>).mockResolvedValue({});
	(api.draftStudio.restore as ReturnType<typeof vi.fn>).mockResolvedValue({});
	(api.draftStudio.delete as ReturnType<typeof vi.fn>).mockResolvedValue({});
	(api.draftStudio.schedule as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 1, status: 'scheduled', scheduled_for: '2026-03-20T10:00:00.000Z' });
	(api.draftStudio.reschedule as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 1, status: 'scheduled', scheduled_for: '2026-03-21T10:00:00.000Z' });
	(api.draftStudio.unschedule as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 1, status: 'draft' });
	(api.draftStudio.tags as ReturnType<typeof vi.fn>).mockResolvedValue([]);
	(api.draftStudio.revisions as ReturnType<typeof vi.fn>).mockResolvedValue([]);
	(api.draftStudio.activity as ReturnType<typeof vi.fn>).mockResolvedValue([]);
	(api.draftStudio.duplicate as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 100, updated_at: '2026-03-14T02:00:00.000Z' });
});

// ---------------------------------------------------------------------------
// loadDrafts
// ---------------------------------------------------------------------------

describe('loadDrafts', () => {
	it('calls draftStudio.list', async () => {
		await composer.loadDrafts();
		expect(api.draftStudio.list).toHaveBeenCalled();
	});

	it('isLoading() returns false after load', async () => {
		await composer.loadDrafts();
		expect(composer.isLoading()).toBe(false);
	});

	it('getError() is null after successful load', async () => {
		await composer.loadDrafts();
		expect(composer.getError()).toBeNull();
	});

	it('getActiveDrafts() returns only draft-status items', async () => {
		await composer.loadDrafts();
		const active = composer.getActiveDrafts();
		expect(active.every((d) => d.status === 'draft')).toBe(true);
		expect(active).toHaveLength(2);
	});

	it('getScheduledDrafts() returns only scheduled-status items', async () => {
		await composer.loadDrafts();
		const scheduled = composer.getScheduledDrafts();
		expect(scheduled.every((d) => d.status === 'scheduled')).toBe(true);
		expect(scheduled).toHaveLength(1);
	});

	it('getPostedDrafts() returns only posted-status items', async () => {
		await composer.loadDrafts();
		const posted = composer.getPostedDrafts();
		expect(posted.every((d) => d.status === 'posted')).toBe(true);
		expect(posted).toHaveLength(1);
	});

	it('sets error on API failure', async () => {
		(api.draftStudio.list as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Server error')
		);
		await composer.loadDrafts();
		expect(composer.getError()).toBe('Server error');
	});
});

// ---------------------------------------------------------------------------
// selectDraft
// ---------------------------------------------------------------------------

describe('selectDraft', () => {
	it('sets selectedId', () => {
		composer.selectDraft(42);
		expect(composer.getSelectedId()).toBe(42);
	});

	it('clears selectedId when null', () => {
		composer.selectDraft(42);
		composer.selectDraft(null);
		expect(composer.getSelectedId()).toBeNull();
	});

	it('getSelectedDraft() returns the matching draft summary after load', async () => {
		await composer.loadDrafts();
		composer.selectDraft(1);
		expect(composer.getSelectedDraft()?.id).toBe(1);
	});

	it('getSelectedDraft() returns null when nothing is selected', async () => {
		await composer.loadDrafts();
		composer.selectDraft(null);
		expect(composer.getSelectedDraft()).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// setTab
// ---------------------------------------------------------------------------

describe('setTab', () => {
	it('getTab() reflects set value', () => {
		composer.setTab('scheduled');
		expect(composer.getTab()).toBe('scheduled');
	});

	it('switching tabs is idempotent', () => {
		composer.setTab('active');
		composer.setTab('active');
		expect(composer.getTab()).toBe('active');
	});

	it('getCurrentTabDrafts() matches the active tab after load', async () => {
		await composer.loadDrafts();
		composer.setTab('active');
		const active = composer.getCurrentTabDrafts();
		expect(active.every((d) => d.status === 'draft')).toBe(true);
	});
});

// ---------------------------------------------------------------------------
// setSearchQuery / setSortBy
// ---------------------------------------------------------------------------

describe('setSearchQuery', () => {
	it('getSearchQuery() reflects the set value', () => {
		composer.setSearchQuery('hello');
		expect(composer.getSearchQuery()).toBe('hello');
	});

	it('empty string clears the search', () => {
		composer.setSearchQuery('hello');
		composer.setSearchQuery('');
		expect(composer.getSearchQuery()).toBe('');
	});
});

describe('setSortBy', () => {
	it('getSortBy() reflects the set value', () => {
		composer.setSortBy('created');
		expect(composer.getSortBy()).toBe('created');
	});

	it('reverts to updated when explicitly set', () => {
		composer.setSortBy('title');
		composer.setSortBy('updated');
		expect(composer.getSortBy()).toBe('updated');
	});
});

// ---------------------------------------------------------------------------
// getTabCounts
// ---------------------------------------------------------------------------

describe('getTabCounts', () => {
	it('returns correct counts after load', async () => {
		await composer.loadDrafts();
		const counts = composer.getTabCounts();
		expect(counts.active).toBe(2);
		expect(counts.scheduled).toBe(1);
		expect(counts.posted).toBe(1);
	});

	it('returns zero counts when list is empty', async () => {
		// Svelte 5 rune state persists in the module — reload with an empty list
		(api.draftStudio.list as ReturnType<typeof vi.fn>).mockResolvedValueOnce([]);
		await composer.loadDrafts();
		const counts = composer.getTabCounts();
		expect(counts.active).toBe(0);
		expect(counts.scheduled).toBe(0);
	});
});

// ---------------------------------------------------------------------------
// createDraft
// ---------------------------------------------------------------------------

describe('createDraft', () => {
	it('calls draftStudio.create and returns the new id', async () => {
		await composer.loadDrafts();
		const newId = await composer.createDraft('Initial content');
		expect(api.draftStudio.create).toHaveBeenCalled();
		expect(newId).toBe(99);
	});

	it('returns null on API failure', async () => {
		(api.draftStudio.create as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Create failed')
		);
		const result = await composer.createDraft();
		expect(result).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// scheduleDraft / unscheduleDraft / rescheduleDraft
// ---------------------------------------------------------------------------

describe('scheduleDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.schedule with id and time', async () => {
		const result = await composer.scheduleDraft(1, '2026-03-20T10:00:00.000Z');
		expect(api.draftStudio.schedule).toHaveBeenCalledWith(1, '2026-03-20T10:00:00.000Z');
		expect(result).toBe(true);
	});

	it('returns false on API failure', async () => {
		(api.draftStudio.schedule as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Schedule failed')
		);
		const result = await composer.scheduleDraft(1, '2026-03-20T10:00:00.000Z');
		expect(result).toBe(false);
	});
});

describe('unscheduleDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.unschedule with id', async () => {
		const result = await composer.unscheduleDraft(3);
		expect(api.draftStudio.unschedule).toHaveBeenCalledWith(3);
		expect(result).toBe(true);
	});
});

describe('rescheduleDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.reschedule with id and new time', async () => {
		const result = await composer.rescheduleDraft(3, '2026-03-21T10:00:00.000Z');
		expect(api.draftStudio.reschedule).toHaveBeenCalledWith(3, '2026-03-21T10:00:00.000Z');
		expect(result).toBe(true);
	});
});

// ---------------------------------------------------------------------------
// archiveDraft / restoreDraft / deleteDraft
// ---------------------------------------------------------------------------

describe('archiveDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.archive with id', async () => {
		await composer.archiveDraft(1);
		expect(api.draftStudio.archive).toHaveBeenCalledWith(1);
	});
});

describe('restoreDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.restore with id', async () => {
		await composer.restoreDraft(1);
		expect(api.draftStudio.restore).toHaveBeenCalledWith(1);
	});
});

describe('deleteDraft', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('calls draftStudio.delete and removes item from collection', async () => {
		await composer.deleteDraft(1);
		expect(api.draftStudio.delete).toHaveBeenCalledWith(1);
		// Item should be removed from active drafts
		const active = composer.getActiveDrafts();
		expect(active.find((d) => d.id === 1)).toBeUndefined();
	});
});

// ---------------------------------------------------------------------------
// setFullDraft / getFullDraft
// ---------------------------------------------------------------------------

describe('fullDraft', () => {
	it('setFullDraft stores a full draft and getFullDraft retrieves it', () => {
		const full = makeFull(5);
		composer.setFullDraft(full);
		expect(composer.getFullDraft()?.id).toBe(5);
	});

	it('getFullDraft returns null after reset', () => {
		composer.setFullDraft(makeFull(5));
		composer.setFullDraft(null);
		expect(composer.getFullDraft()).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// updateDraftInCollection
// ---------------------------------------------------------------------------

describe('updateDraftInCollection', () => {
	beforeEach(async () => {
		await composer.loadDrafts();
	});

	it('updates a field on a specific draft in the collection', () => {
		composer.updateDraftInCollection(1, { title: 'Updated Title' });
		const updated = composer.getActiveDrafts().find((d) => d.id === 1);
		expect(updated?.title).toBe('Updated Title');
	});

	it('does not affect other drafts', () => {
		composer.updateDraftInCollection(1, { title: 'Only Draft 1 Changed' });
		const draft2 = composer.getActiveDrafts().find((d) => d.id === 2);
		expect(draft2?.title).toBe('Draft 2');
	});
});
