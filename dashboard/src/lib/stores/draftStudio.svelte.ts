import { api } from '$lib/api';
import type { DraftSummary, ScheduledContentItem } from '$lib/api/types';

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function byUpdatedDesc(a: DraftSummary, b: DraftSummary): number {
	return b.updated_at.localeCompare(a.updated_at);
}

// ---------------------------------------------------------------------------
// Derived
// ---------------------------------------------------------------------------

export const activeDrafts = $derived(
	collection.filter((d) => d.status === 'draft').sort(byUpdatedDesc)
);

export const scheduledDrafts = $derived(
	collection.filter((d) => d.status === 'scheduled').sort(byUpdatedDesc)
);

export const postedDrafts = $derived(
	collection.filter((d) => d.status === 'posted').sort(byUpdatedDesc)
);

export const currentTabDrafts = $derived(
	tab === 'active'
		? activeDrafts
		: tab === 'scheduled'
			? scheduledDrafts
			: tab === 'posted'
				? postedDrafts
				: [...archivedCollection].sort(byUpdatedDesc)
);

export const selectedDraft = $derived(
	[...collection, ...archivedCollection].find((d) => d.id === selectedId) ?? null
);

export const tabCounts = $derived({
	active: activeDrafts.length,
	scheduled: scheduledDrafts.length,
	posted: postedDrafts.length,
	archive: archivedCollection.length
});

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

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

export async function loadDrafts(): Promise<void> {
	loading = true;
	error = null;
	try {
		collection = await api.draftStudio.list();
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
			created_at: result.updated_at
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
	collection = collection.map((d) =>
		d.id === id ? { ...d, ...updates } : d
	);
}

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
}

export function clearError(): void {
	error = null;
}

export function setSyncStatus(status: typeof syncStatus): void {
	syncStatus = status;
}
