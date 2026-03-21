<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import { canPost } from '$lib/stores/runtime';
	import { loadSchedule } from '$lib/stores/calendar';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import { api } from '$lib/api';
	import type { ProvenanceLink } from '$lib/api/types';
	import type { SyncStatus } from '$lib/utils/composerAutosave';
	import type { ComposeRequest } from '$lib/api';
	import { matchEvent } from '$lib/utils/shortcuts';
	import { events as wsEvents } from '$lib/stores/websocket';
	import { parseServerDraft } from '$lib/utils/draftStudioParse';
	import type { HydrationPayload } from '$lib/utils/draftStudioParse';
	import DraftStudioDrawer from './DraftStudioDrawer.svelte';
	import DraftStudioComposerZone from './DraftStudioComposerZone.svelte';
	import DraftStudioDetailsPane from './DraftStudioDetailsPane.svelte';

	let loadingDraft = $state(false);
	let syncStatus = $state<SyncStatus>('saved');
	let conflictDraftId = $state<number | null>(null);
	let detailsPanelOpen = $state(false);
	let activePanel = $state<'details' | 'history'>('details');
	let prefillSchedule = $state<string | null>(null);
	let drawerOpen = $state(false);
	let approvalMode = $state(true);
	let selectionSessionId = $state<string | null>(null);
	let lastConsumedSelectionId = $state<string | null>(null);
	let draftProvenance = $state<ProvenanceLink[]>([]);

	const publishEnabled = $derived($canPost && !approvalMode);

	let hydration = $state<HydrationPayload | null>(null);
	let hydrationDraftId = $state<number | null>(null);

	onMount(() => {
		studio.initFromUrl($page.url);

		api.assist.mode().then((res) => {
			approvalMode = res.approval_mode;
		}).catch(() => { /* default to approval_mode=true (safe fallback) */ });

		loadSchedule();

		const isNewDraft = $page.url.searchParams.get('new') === 'true';
		const hasExplicitId = $page.url.searchParams.has('id');

		studio.loadDrafts().then(() => {
			if (!isNewDraft && !hasExplicitId && studio.getSelectedId() === null) {
				const active = studio.getActiveDrafts();
				if (active.length > 0) studio.selectDraft(active[0].id);
			}
		});
		studio.loadTags();

		if ($page.url.searchParams.get('new') === 'true') {
			const prefillContent = $page.url.searchParams.get('prefill_content') || undefined;
			const url = new URL(window.location.href);
			url.searchParams.delete('new');
			url.searchParams.delete('prefill_content');
			history.replaceState(null, '', url.toString());
			studio.createDraft(prefillContent).then((newId) => {
				if (newId !== null) {
					console.info('[draft-studio]', { event: 'draft_created', id: newId, source: prefillContent ? 'onboarding' : 'cmd-n' });
				}
			});
		}

		const prefillParam = $page.url.searchParams.get('prefill_schedule');
		if (prefillParam) {
			const parsed = new Date(prefillParam);
			if (!isNaN(parsed.getTime())) prefillSchedule = prefillParam;
			const url = new URL(window.location.href);
			url.searchParams.delete('prefill_schedule');
			history.replaceState(null, '', url.toString());
		}

		const selectionParam = $page.url.searchParams.get('selection');
		if (selectionParam) {
			selectionSessionId = selectionParam;
			lastConsumedSelectionId = selectionParam;
			const url = new URL(window.location.href);
			url.searchParams.delete('selection');
			history.replaceState(null, '', url.toString());
		}

		const handler = () => {
			studio.reset();
			hydration = null;
			hydrationDraftId = null;
			studio.loadDrafts();
			studio.loadTags();
		};
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	$effect(() => {
		const id = studio.getSelectedId();
		if (id === null) {
			hydration = null;
			hydrationDraftId = null;
			draftProvenance = [];
			studio.setFullDraft(null);
			return;
		}
		if (id === hydrationDraftId) return;
		loadingDraft = true;
		syncStatus = 'saved';
		conflictDraftId = null;
		fetchDraft(id);
	});

	$effect(() => {
		void studio.getSelectedId();
		studio.loadSelectedDraftTags();
	});

	$effect(() => {
		void studio.getSelectedId();
		if (activePanel === 'history' && detailsPanelOpen) {
			studio.loadRevisions();
			studio.loadActivity();
		}
	});

	$effect(() => {
		const eventList = $wsEvents;
		if (eventList.length === 0) return;
		const latest = eventList[0];
		if (latest.type !== 'SelectionReceived') return;
		const sid = latest.session_id as string | undefined;
		if (!sid || sid === lastConsumedSelectionId) return;
		selectionSessionId = sid;
		lastConsumedSelectionId = sid;
	});

	async function fetchDraft(id: number) {
		try {
			const draft = await api.draftStudio.get(id);
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(draft);
			console.info('[draft-studio]', { event: 'draft_selected', id, source: 'fetch' });
			hydration = parseServerDraft(draft);
			hydrationDraftId = id;

			// Fetch provenance links (non-blocking — display is optional).
			api.draftStudio.provenance(id).then((links) => {
				if (studio.getSelectedId() === id) draftProvenance = links;
			}).catch(() => {
				if (studio.getSelectedId() === id) draftProvenance = [];
			});
		} catch {
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(null);
			hydration = null;
			hydrationDraftId = null;
			draftProvenance = [];
		} finally {
			if (studio.getSelectedId() === id) loadingDraft = false;
		}
	}

	async function handleCreate() {
		const newId = await studio.createDraft();
		if (newId !== null) {
			console.info('[draft-studio]', { event: 'draft_created', id: newId, source: 'rail-button' });
			drawerOpen = false;
		}
	}

	function handleSyncStatus(status: SyncStatus) {
		syncStatus = status;
		if (status === 'conflict') conflictDraftId = studio.getSelectedId();
		if (status === 'offline') {
			console.info('[draft-studio]', { event: 'save_failed', id: studio.getSelectedId(), syncStatus: status });
		}
	}

	async function handleConflictResolution(resolution: 'use-mine' | 'reload-server') {
		const id = studio.getSelectedId();
		if (id === null) return;
		if (resolution === 'reload-server') {
			hydration = null;
			hydrationDraftId = null;
			loadingDraft = true;
			syncStatus = 'saved';
			conflictDraftId = null;
			await fetchDraft(id);
		}
		if (resolution === 'use-mine') {
			try {
				const draft = await api.draftStudio.get(id);
				if (studio.getSelectedId() !== id) return;
				if (hydration) {
					hydration = { ...hydration, updatedAt: draft.updated_at };
					syncStatus = 'unsaved';
					conflictDraftId = null;
				}
			} catch {
				syncStatus = 'offline';
			}
		}
	}

	async function handleDraftSubmit(data: ComposeRequest) {
		const id = studio.getSelectedId();
		if (id === null) return;
		if (data.scheduled_for) {
			const success = await studio.scheduleDraft(id, data.scheduled_for);
			if (success) await fetchDraft(id);
		}
	}

	async function handleSchedule(scheduledFor: string) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.scheduleDraft(id, scheduledFor);
		if (success) {
			console.info('[draft-studio]', { event: 'transition', id, from: 'draft', to: 'scheduled' });
			await fetchDraft(id);
		}
	}

	async function handleUnschedule() {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.unscheduleDraft(id);
		if (success) {
			console.info('[draft-studio]', { event: 'transition', id, from: 'scheduled', to: 'draft' });
			await fetchDraft(id);
		}
	}

	async function handleReschedule(scheduledFor: string) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.rescheduleDraft(id, scheduledFor);
		if (success) await fetchDraft(id);
	}

	async function handleRestoreFromRevision(revisionId: number) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.restoreFromRevision(revisionId);
		if (success) {
			console.info('[draft-studio]', { event: 'restore_executed', id, revisionId });
			hydration = null;
			hydrationDraftId = null;
			loadingDraft = true;
			await fetchDraft(id);
		}
	}

	function handleDraftAction(actionId: string) {
		const id = studio.getSelectedId();
		switch (actionId) {
			case 'ds-new-draft': handleCreate(); break;
			case 'ds-duplicate': if (id !== null) studio.duplicateDraft(id); break;
			case 'ds-delete': if (id !== null) studio.deleteDraft(id); break;
		}
	}

	async function handleMetaUpdate(data: { title?: string; notes?: string }) {
		const id = studio.getSelectedId();
		if (id === null) return;
		await api.draftStudio.updateMeta(id, data);
		if (data.title !== undefined) studio.updateDraftInCollection(id, { title: data.title || null });
	}

	function handleShellKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (drawerOpen) { e.preventDefault(); drawerOpen = false; return; }
			const zone = document.querySelector('.composer-zone');
			if (zone?.contains(document.activeElement)) { e.preventDefault(); e.stopPropagation(); }
		}
		if (matchEvent(e, 'cmd+shift+d')) { e.preventDefault(); activePanel = 'details'; detailsPanelOpen = !detailsPanelOpen; }
		if (matchEvent(e, 'cmd+shift+h')) {
			e.preventDefault();
			if (activePanel === 'history' && detailsPanelOpen) { detailsPanelOpen = false; }
			else { activePanel = 'history'; detailsPanelOpen = true; studio.loadRevisions(); studio.loadActivity(); }
		}
		if (matchEvent(e, 'cmd+shift+o')) { e.preventDefault(); drawerOpen = !drawerOpen; }
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="studio-shell"
	class:details-open={detailsPanelOpen && studio.getSelectedId() !== null}
	onkeydown={handleShellKeydown}
>
	{#if drawerOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="drawer-backdrop" onclick={() => (drawerOpen = false)}></div>
	{/if}

	<DraftStudioDrawer
		open={drawerOpen}
		onSelect={(id) => { studio.selectDraft(id); drawerOpen = false; }}
		onCreate={handleCreate}
		onDelete={(id) => studio.deleteDraft(id)}
		onDuplicate={(id) => studio.duplicateDraft(id)}
		onRestore={(id) => studio.restoreDraft(id)}
	/>

	<DraftStudioComposerZone
		{drawerOpen}
		{hydration}
		{hydrationDraftId}
		{loadingDraft}
		{publishEnabled}
		{selectionSessionId}
		onToggleDrawer={() => (drawerOpen = !drawerOpen)}
		onSyncStatus={handleSyncStatus}
		onDraftAction={handleDraftAction}
		onDraftSubmit={handleDraftSubmit}
		onFetchDraft={fetchDraft}
		onCreate={handleCreate}
		onSelectionConsumed={() => { selectionSessionId = null; }}
	/>

	{#if detailsPanelOpen && studio.getSelectedId() !== null}
		<DraftStudioDetailsPane
			{activePanel}
			{prefillSchedule}
			provenance={draftProvenance}
			onActivePanel={(p) => (activePanel = p)}
			onUpdateMeta={handleMetaUpdate}
			onAssignTag={(id) => studio.assignTag(id)}
			onUnassignTag={(id) => studio.unassignTag(id)}
			onCreateTag={(name) => studio.createAndAssignTag(name)}
			onSchedule={handleSchedule}
			onUnschedule={handleUnschedule}
			onReschedule={handleReschedule}
			onDuplicate={() => { const id = studio.getSelectedId(); if (id !== null) studio.duplicateDraft(id); }}
			onRestoreFromRevision={handleRestoreFromRevision}
			onClose={() => (detailsPanelOpen = false)}
		/>
	{/if}
</div>

<style>
	.studio-shell {
		display: grid;
		grid-template-columns: 1fr;
		height: calc(100vh - 48px);
		margin: -24px -32px;
		overflow: hidden;
		position: relative;
	}

	.studio-shell.details-open {
		grid-template-columns: 1fr 280px;
	}

	.drawer-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.25);
		z-index: 10;
		backdrop-filter: blur(1px);
	}
</style>
