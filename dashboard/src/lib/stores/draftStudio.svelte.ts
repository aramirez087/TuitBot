import { api } from '$lib/api';
import type {
	DraftSummary,
	ScheduledContentItem,
	ContentTag,
	ContentRevision,
	ContentActivity
} from '$lib/api/types';

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

let collection = $state<DraftSummary[]>([]);
let archivedCollection = $state<DraftSummary[]>([]);
let selectedId = $state<number | null>(null);
let tab = $state<'active' | 'scheduled' | 'posted' | 'archive'>('active');
let loading = $state(true);
let archiveLoaded = $state(false);
let error = $state<string | null>(null);
let syncStatus = $state<'saved' | 'saving' | 'unsaved' | 'offline' | 'conflict'>('saved');
let fullDraft = $state<ScheduledContentItem | null>(null);

// Revision / activity state
let revisions = $state<ContentRevision[]>([]);
let activity = $state<ContentActivity[]>([]);

// Filter / sort / tag state
let searchQuery = $state('');
let sortBy = $state<'updated' | 'created' | 'title' | 'scheduled'>('updated');
let tagFilter = $state<number | null>(null);
let accountTags = $state<ContentTag[]>([]);
let selectedDraftTags = $state<ContentTag[]>([]);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function byUpdatedDesc(a: DraftSummary, b: DraftSummary): number {
	return b.updated_at.localeCompare(a.updated_at);
}

function sortDrafts(drafts: DraftSummary[], by: typeof sortBy): DraftSummary[] {
	switch (by) {
		case 'created':
			return [...drafts].sort((a, b) => b.created_at.localeCompare(a.created_at));
		case 'title':
			return [...drafts].sort((a, b) =>
				(a.title ?? a.content_preview).localeCompare(b.title ?? b.content_preview)
			);
		case 'scheduled':
			return [...drafts].sort((a, b) =>
				(a.scheduled_for ?? 'z').localeCompare(b.scheduled_for ?? 'z')
			);
		default:
			return [...drafts].sort(byUpdatedDesc);
	}
}

function filterBySearch(drafts: DraftSummary[], q: string): DraftSummary[] {
	if (!q.trim()) return drafts;
	const needle = q.toLowerCase();
	return drafts.filter(
		(d) =>
			(d.title ?? '').toLowerCase().includes(needle) ||
			d.content_preview.toLowerCase().includes(needle)
	);
}

// ---------------------------------------------------------------------------
// Derived
// ---------------------------------------------------------------------------

const activeDrafts = $derived(
	collection.filter((d) => d.status === 'draft').sort(byUpdatedDesc)
);

const scheduledDrafts = $derived(
	collection.filter((d) => d.status === 'scheduled').sort(byUpdatedDesc)
);

const postedDrafts = $derived(
	collection.filter((d) => d.status === 'posted').sort(byUpdatedDesc)
);

const rawTabDrafts = $derived(
	tab === 'active'
		? activeDrafts
		: tab === 'scheduled'
			? scheduledDrafts
			: tab === 'posted'
				? postedDrafts
				: [...archivedCollection].sort(byUpdatedDesc)
);

const currentTabDrafts = $derived(
	sortDrafts(filterBySearch(rawTabDrafts, searchQuery), sortBy)
);

const selectedDraft = $derived(
	[...collection, ...archivedCollection].find((d) => d.id === selectedId) ?? null
);

const tabCounts = $derived({
	active: activeDrafts.length,
	scheduled: scheduledDrafts.length,
	posted: postedDrafts.length,
	archive: archivedCollection.length
});

export function getActiveDrafts(): DraftSummary[] {
	return activeDrafts;
}

export function getScheduledDrafts(): DraftSummary[] {
	return scheduledDrafts;
}

export function getPostedDrafts(): DraftSummary[] {
	return postedDrafts;
}

export function getCurrentTabDrafts(): DraftSummary[] {
	return currentTabDrafts;
}

export function getSelectedDraft(): DraftSummary | null {
	return selectedDraft;
}

export function getTabCounts(): { active: number; scheduled: number; posted: number; archive: number } {
	return tabCounts;
}

// ---------------------------------------------------------------------------
// Getters (for non-derived reactive reads)
// ---------------------------------------------------------------------------

export function getSelectedId(): number | null {
	return selectedId;
}

export function getTab(): 'active' | 'scheduled' | 'posted' | 'archive' {
	return tab;
}

export function isLoading(): boolean {
	return loading;
}

export function getError(): string | null {
	return error;
}

export function getSyncStatus(): typeof syncStatus {
	return syncStatus;
}

export function getSearchQuery(): string {
	return searchQuery;
}

export function getSortBy(): typeof sortBy {
	return sortBy;
}

export function getTagFilter(): number | null {
	return tagFilter;
}

export function getAccountTags(): ContentTag[] {
	return accountTags;
}

export function getSelectedDraftTags(): ContentTag[] {
	return selectedDraftTags;
}

export function getRevisions(): ContentRevision[] {
	return revisions;
}

export function getActivity(): ContentActivity[] {
	return activity;
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

export async function loadDrafts(): Promise<void> {
	loading = true;
	error = null;
	try {
		const params: { tag?: number } = {};
		if (tagFilter !== null) params.tag = tagFilter;
		collection = await api.draftStudio.list(params);
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load drafts';
	} finally {
		loading = false;
	}
}

async function loadArchived(): Promise<void> {
	try {
		archivedCollection = await api.draftStudio.list({ archived: true });
		archiveLoaded = true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to load archived drafts';
	}
}

export function selectDraft(id: number | null): void {
	selectedId = id;
	const url = new URL(window.location.href);
	if (id !== null) {
		url.searchParams.set('id', String(id));
	} else {
		url.searchParams.delete('id');
	}
	history.replaceState(null, '', url.toString());
}

export function setTab(newTab: 'active' | 'scheduled' | 'posted' | 'archive'): void {
	tab = newTab;
	if (newTab === 'archive' && !archiveLoaded) {
		loadArchived();
	}
}

export function setSearchQuery(q: string): void {
	searchQuery = q;
}

export function setSortBy(by: typeof sortBy): void {
	sortBy = by;
}

export async function setTagFilter(tagId: number | null): Promise<void> {
	tagFilter = tagId;
	await loadDrafts();
}

export async function createDraft(): Promise<number | null> {
	try {
		const result = await api.draftStudio.create({ content_type: 'tweet' });
		const newDraft: DraftSummary = {
			id: result.id,
			title: null,
			content_type: 'tweet',
			content_preview: '',
			status: 'draft',
			scheduled_for: null,
			archived_at: null,
			updated_at: result.updated_at,
			created_at: result.updated_at,
			source: 'manual'
		};
		collection = [newDraft, ...collection];
		tab = 'active';
		selectDraft(result.id);
		return result.id;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to create draft';
		return null;
	}
}

export async function archiveDraft(id: number): Promise<void> {
	try {
		const result = await api.draftStudio.archive(id);
		const draft = collection.find((d) => d.id === id);
		if (draft) {
			collection = collection.filter((d) => d.id !== id);
			archivedCollection = [
				{ ...draft, archived_at: result.archived_at },
				...archivedCollection
			];
		}
		if (selectedId === id) {
			const remaining = currentTabDrafts;
			selectDraft(remaining[0]?.id ?? null);
		}
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to archive draft';
	}
}

export async function restoreDraft(id: number): Promise<void> {
	try {
		await api.draftStudio.restore(id);
		const draft = archivedCollection.find((d) => d.id === id);
		if (draft) {
			archivedCollection = archivedCollection.filter((d) => d.id !== id);
			collection = [{ ...draft, archived_at: null, status: 'draft' }, ...collection];
		}
		if (selectedId === id) {
			const remaining = currentTabDrafts;
			selectDraft(remaining[0]?.id ?? null);
		}
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to restore draft';
	}
}

export async function deleteDraft(id: number): Promise<void> {
	try {
		await api.draftStudio.delete(id);
		collection = collection.filter((d) => d.id !== id);
		archivedCollection = archivedCollection.filter((d) => d.id !== id);
		if (selectedId === id) {
			const remaining = currentTabDrafts.filter((d) => d.id !== id);
			selectDraft(remaining[0]?.id ?? null);
		}
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to delete draft';
	}
}

export async function scheduleDraft(id: number, scheduledFor: string): Promise<boolean> {
	try {
		const result = await api.draftStudio.schedule(id, scheduledFor);
		collection = collection.map((d) =>
			d.id === id
				? { ...d, status: result.status, scheduled_for: result.scheduled_for }
				: d
		);
		tab = 'scheduled';
		return true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to schedule draft';
		return false;
	}
}

export async function unscheduleDraft(id: number): Promise<boolean> {
	try {
		await api.draftStudio.unschedule(id);
		collection = collection.map((d) =>
			d.id === id ? { ...d, status: 'draft', scheduled_for: null } : d
		);
		tab = 'active';
		return true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to unschedule draft';
		return false;
	}
}

export async function rescheduleDraft(id: number, scheduledFor: string): Promise<boolean> {
	try {
		await api.draftStudio.unschedule(id);
		const result = await api.draftStudio.schedule(id, scheduledFor);
		collection = collection.map((d) =>
			d.id === id
				? { ...d, status: result.status, scheduled_for: result.scheduled_for }
				: d
		);
		return true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to reschedule draft';
		return false;
	}
}

export async function duplicateDraft(id: number): Promise<void> {
	try {
		const result = await api.draftStudio.duplicate(id);
		// Reload to get the new draft's summary
		await loadDrafts();
		selectDraft(result.id);
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to duplicate draft';
	}
}

export function initFromUrl(url: URL): void {
	const idParam = url.searchParams.get('id');
	if (idParam) {
		const parsed = parseInt(idParam, 10);
		if (!isNaN(parsed)) {
			selectedId = parsed;
		}
	}
}

export function getFullDraft(): ScheduledContentItem | null {
	return fullDraft;
}

export function setFullDraft(draft: ScheduledContentItem | null): void {
	fullDraft = draft;
}

export function updateDraftInCollection(id: number, updates: Partial<DraftSummary>): void {
	collection = collection.map((d) => (d.id === id ? { ...d, ...updates } : d));
}

// ---------------------------------------------------------------------------
// Revision / activity actions
// ---------------------------------------------------------------------------

export async function loadRevisions(): Promise<void> {
	if (selectedId === null) {
		revisions = [];
		return;
	}
	try {
		revisions = await api.draftStudio.revisions(selectedId);
	} catch {
		revisions = [];
	}
}

export async function loadActivity(): Promise<void> {
	if (selectedId === null) {
		activity = [];
		return;
	}
	try {
		activity = await api.draftStudio.activity(selectedId);
	} catch {
		activity = [];
	}
}

export async function restoreFromRevision(revisionId: number): Promise<boolean> {
	if (selectedId === null) return false;
	try {
		const updated = await api.draftStudio.restoreRevision(selectedId, revisionId);
		fullDraft = updated;
		updateDraftInCollection(selectedId, { updated_at: updated.updated_at });
		await loadRevisions();
		await loadActivity();
		return true;
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to restore revision';
		return false;
	}
}

// ---------------------------------------------------------------------------
// Tag actions
// ---------------------------------------------------------------------------

export async function loadTags(): Promise<void> {
	try {
		accountTags = await api.tags.list();
	} catch {
		// Non-critical — tags are optional
	}
}

export async function loadSelectedDraftTags(): Promise<void> {
	if (selectedId === null) {
		selectedDraftTags = [];
		return;
	}
	try {
		selectedDraftTags = await api.draftStudio.tags(selectedId);
	} catch {
		selectedDraftTags = [];
	}
}

export async function assignTag(tagId: number): Promise<void> {
	if (selectedId === null) return;
	try {
		await api.draftStudio.assignTag(selectedId, tagId);
		await loadSelectedDraftTags();
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to assign tag';
	}
}

export async function unassignTag(tagId: number): Promise<void> {
	if (selectedId === null) return;
	try {
		await api.draftStudio.unassignTag(selectedId, tagId);
		await loadSelectedDraftTags();
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to unassign tag';
	}
}

export async function createAndAssignTag(name: string): Promise<void> {
	if (selectedId === null) return;
	try {
		const result = await api.tags.create(name);
		await api.draftStudio.assignTag(selectedId, result.id);
		await loadTags();
		await loadSelectedDraftTags();
	} catch (e) {
		error = e instanceof Error ? e.message : 'Failed to create tag';
	}
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

export function reset(): void {
	collection = [];
	archivedCollection = [];
	selectedId = null;
	tab = 'active';
	loading = true;
	archiveLoaded = false;
	error = null;
	syncStatus = 'saved';
	fullDraft = null;
	revisions = [];
	activity = [];
	searchQuery = '';
	sortBy = 'updated';
	tagFilter = null;
	accountTags = [];
	selectedDraftTags = [];
}

export function clearError(): void {
	error = null;
}

export function setSyncStatus(status: typeof syncStatus): void {
	syncStatus = status;
}
