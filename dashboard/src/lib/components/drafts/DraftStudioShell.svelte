<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import { api, type ThreadBlock } from '$lib/api';
	import type { ScheduledContentItem } from '$lib/api/types';
	import type { SyncStatus } from '$lib/utils/composerAutosave';
	import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';
	import type { PaletteAction } from '$lib/components/CommandPalette.svelte';
	import { Plus, Copy, Archive, ArrowLeft } from 'lucide-svelte';
	import DraftRail from './DraftRail.svelte';
	import DraftEmptyState from './DraftEmptyState.svelte';
	import DraftSyncBadge from './DraftSyncBadge.svelte';
	import ComposeWorkspace from '$lib/components/composer/ComposeWorkspace.svelte';

	let loadingDraft = $state(false);
	let syncStatus = $state<SyncStatus>('saved');
	let conflictDraftId = $state<number | null>(null);
	let railComponent: DraftRail | undefined = $state();
	let composerZoneEl: HTMLDivElement | undefined = $state();

	interface HydrationPayload {
		mode: 'tweet' | 'thread';
		tweetText: string;
		threadBlocks: ThreadBlock[];
		attachedMedia: AttachedMedia[];
		updatedAt: string;
	}

	let hydration = $state<HydrationPayload | null>(null);
	let hydrationDraftId = $state<number | null>(null);

	const draftStudioPaletteActions: PaletteAction[] = [
		{ id: 'ds-new-draft', label: 'New draft', icon: Plus, category: 'DraftStudio', shortcut: 'n', when: 'always' },
		{ id: 'ds-duplicate', label: 'Duplicate current draft', icon: Copy, category: 'DraftStudio', shortcut: 'd', when: 'always' },
		{ id: 'ds-archive', label: 'Archive current draft', icon: Archive, category: 'DraftStudio', shortcut: 'backspace', when: 'always' },
		{ id: 'ds-jump-rail', label: 'Jump to rail', icon: ArrowLeft, category: 'DraftStudio', shortcut: 'escape', when: 'always' }
	];

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
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(draft);
			hydration = parseServerDraft(draft);
			hydrationDraftId = id;
		} catch {
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

	function handleArchive(id: number) {
		studio.archiveDraft(id);
	}

	function handleDuplicate(id: number) {
		studio.duplicateDraft(id);
	}

	function handleRestore(id: number) {
		studio.restoreDraft(id);
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

	function handleDraftSubmit() {
		// Schedule/publish flows are in Session 08.
	}

	function handleDraftAction(actionId: string) {
		const id = studio.getSelectedId();
		switch (actionId) {
			case 'ds-new-draft':
				handleCreate();
				break;
			case 'ds-duplicate':
				if (id !== null) handleDuplicate(id);
				break;
			case 'ds-archive':
				if (id !== null) handleArchive(id);
				break;
			case 'ds-jump-rail':
				railComponent?.focus();
				break;
		}
	}

	function handleShellKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && composerZoneEl?.contains(document.activeElement)) {
			e.preventDefault();
			e.stopPropagation();
			railComponent?.focus();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="studio-shell" onkeydown={handleShellKeydown}>
	<div class="rail-zone">
		<DraftRail
			bind:this={railComponent}
			drafts={studio.currentTabDrafts}
			selectedId={studio.getSelectedId()}
			tab={studio.getTab()}
			tabCounts={studio.tabCounts}
			loading={studio.isLoading()}
			onselect={(id) => studio.selectDraft(id)}
			ontabchange={(t) => studio.setTab(t)}
			oncreate={handleCreate}
			onarchive={handleArchive}
			onduplicate={handleDuplicate}
			onrestore={handleRestore}
		/>
	</div>

	<div class="composer-zone" bind:this={composerZoneEl}>
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
						extraPaletteActions={draftStudioPaletteActions}
						ondraftaction={handleDraftAction}
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
