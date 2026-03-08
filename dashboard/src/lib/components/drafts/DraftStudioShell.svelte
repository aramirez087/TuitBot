<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import { api, type ThreadBlock } from '$lib/api';
	import type { ScheduledContentItem } from '$lib/api/types';
	import type { SyncStatus } from '$lib/utils/composerAutosave';
	import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';
	import DraftRail from './DraftRail.svelte';
	import DraftEmptyState from './DraftEmptyState.svelte';
	import DraftSyncBadge from './DraftSyncBadge.svelte';
	import ComposeWorkspace from '$lib/components/composer/ComposeWorkspace.svelte';

	let loadingDraft = $state(false);
	let syncStatus = $state<SyncStatus>('saved');
	let conflictDraftId = $state<number | null>(null);

	interface HydrationPayload {
		mode: 'tweet' | 'thread';
		tweetText: string;
		threadBlocks: ThreadBlock[];
		attachedMedia: AttachedMedia[];
		updatedAt: string;
	}

	let hydration = $state<HydrationPayload | null>(null);
	let hydrationDraftId = $state<number | null>(null);

	onMount(() => {
		studio.initFromUrl($page.url);
		studio.loadDrafts();

		const handler = () => {
			studio.reset();
			hydration = null;
			hydrationDraftId = null;
			studio.loadDrafts();
		};
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	// Fetch full draft content when selection changes
	$effect(() => {
		const id = studio.getSelectedId();
		if (id === null) {
			hydration = null;
			hydrationDraftId = null;
			studio.setFullDraft(null);
			return;
		}
		if (id === hydrationDraftId) return;

		loadingDraft = true;
		syncStatus = 'saved';
		conflictDraftId = null;
		fetchDraft(id);
	});

	async function fetchDraft(id: number) {
		try {
			const draft = await api.draftStudio.get(id);
			if (studio.getSelectedId() !== id) return; // selection changed
			studio.setFullDraft(draft);
			hydration = parseServerDraft(draft);
			hydrationDraftId = id;
		} catch (e) {
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(null);
			hydration = null;
			hydrationDraftId = null;
		} finally {
			if (studio.getSelectedId() === id) loadingDraft = false;
		}
	}

	function parseServerDraft(draft: ScheduledContentItem): HydrationPayload {
		if (draft.content_type === 'thread') {
			let texts: string[] = [];
			try {
				const parsed = JSON.parse(draft.content || '[]');
				texts = Array.isArray(parsed) ? parsed.filter((t): t is string => typeof t === 'string') : [];
			} catch {
				texts = draft.content ? [draft.content] : [];
			}
			return {
				mode: 'thread',
				tweetText: '',
				threadBlocks: texts.length > 0
					? texts.map((text, i) => ({
						id: crypto.randomUUID(), text, media_paths: [], order: i
					}))
					: [
						{ id: crypto.randomUUID(), text: '', media_paths: [], order: 0 },
						{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 }
					],
				attachedMedia: [],
				updatedAt: draft.updated_at
			};
		}
		return {
			mode: 'tweet',
			tweetText: draft.content || '',
			threadBlocks: [],
			attachedMedia: [],
			updatedAt: draft.updated_at
		};
	}

	function handleCreate() {
		studio.createDraft();
	}

	function handleSyncStatus(status: SyncStatus) {
		syncStatus = status;
		if (status === 'conflict') {
			conflictDraftId = studio.getSelectedId();
		}
	}

	async function handleConflictResolution(resolution: 'use-mine' | 'reload-server') {
		const id = studio.getSelectedId();
		if (id === null) return;

		if (resolution === 'reload-server') {
			// Re-fetch and re-hydrate by clearing hydration (triggers {#key} remount)
			hydration = null;
			hydrationDraftId = null;
			loadingDraft = true;
			syncStatus = 'saved';
			conflictDraftId = null;
			await fetchDraft(id);
		}
		// "use-mine" is handled inside the DraftSaveManager:
		// The manager needs to re-fetch to get the new updated_at, then re-PATCH.
		// We re-fetch the draft to get the latest updated_at and re-hydrate the manager.
		if (resolution === 'use-mine') {
			try {
				const draft = await api.draftStudio.get(id);
				if (studio.getSelectedId() !== id) return;
				// Update the hydration's updatedAt so the ComposeWorkspace's manager
				// can force-save with the new timestamp. We force a remount with new updatedAt.
				if (hydration) {
					hydration = { ...hydration, updatedAt: draft.updated_at };
					// Force {#key} to remount is too aggressive here — the user would lose
					// their local text. Instead, bump hydrationDraftId to trigger re-key.
					// Actually, we can't do this without losing state. Instead, we'll
					// just update the sync status. The DraftSaveManager in the ComposeWorkspace
					// will retry on the next edit.
					syncStatus = 'unsaved';
					conflictDraftId = null;
				}
			} catch {
				syncStatus = 'offline';
			}
		}
	}

	function handleDraftSubmit() {
		// For now, draft studio submit is a no-op placeholder.
		// Schedule/publish flows are in Session 08.
	}
</script>

<div class="studio-shell">
	<div class="rail-zone">
		<DraftRail
			drafts={studio.currentTabDrafts}
			selectedId={studio.getSelectedId()}
			tab={studio.getTab()}
			tabCounts={studio.tabCounts}
			loading={studio.isLoading()}
			onselect={(id) => studio.selectDraft(id)}
			ontabchange={(t) => studio.setTab(t)}
			oncreate={handleCreate}
		/>
	</div>

	<div class="composer-zone">
		{#if studio.getError()}
			<div class="error-banner">
				<span>{studio.getError()}</span>
				<button type="button" onclick={() => studio.clearError()}>Dismiss</button>
			</div>
		{/if}

		{#if studio.isLoading() && !studio.selectedDraft}
			<div class="zone-loading">
				<div class="zone-spinner"></div>
			</div>
		{:else if studio.getSelectedId() !== null}
			{#if loadingDraft}
				<div class="zone-loading">
					<div class="zone-spinner"></div>
				</div>
			{:else if hydration && hydrationDraftId !== null}
				<div class="composer-header">
					<DraftSyncBadge
						status={syncStatus}
						onresolveconflict={handleConflictResolution}
					/>
				</div>
				{#key hydrationDraftId}
					<ComposeWorkspace
						draftId={hydrationDraftId}
						initialContent={hydration}
						embedded={true}
						schedule={null}
						canPublish={false}
						onsubmit={handleDraftSubmit}
						onsyncstatus={handleSyncStatus}
					/>
				{/key}
			{:else}
				<div class="zone-error">
					<p>Failed to load draft content.</p>
					<button type="button" onclick={() => { if (studio.getSelectedId() !== null) fetchDraft(studio.getSelectedId()!); }}>
						Retry
					</button>
				</div>
			{/if}
		{:else if studio.tabCounts.active === 0 && studio.tabCounts.scheduled === 0}
			<DraftEmptyState variant="no-drafts" oncreate={handleCreate} />
		{:else}
			<DraftEmptyState variant="no-selection" oncreate={handleCreate} />
		{/if}
	</div>
</div>

<style>
	.studio-shell {
		display: grid;
		grid-template-columns: 260px 1fr;
		height: calc(100vh - 48px);
		margin: -24px -32px;
		overflow: hidden;
	}

	.rail-zone {
		min-width: 0;
		overflow: hidden;
	}

	.composer-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow-y: auto;
		background: var(--color-base);
	}

	.composer-header {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		padding: 8px 16px 0;
		flex-shrink: 0;
	}

	.zone-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	.zone-spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border-subtle);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	.zone-error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	.zone-error button {
		padding: 6px 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
	}

	.zone-error button:hover {
		background: var(--color-surface);
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 13px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-danger) 20%, transparent);
		flex-shrink: 0;
	}

	.error-banner button {
		border: none;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		text-decoration: underline;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
