<script lang="ts">
	import { Plus, Copy, Trash2, Files } from 'lucide-svelte';
	import { schedule as scheduleStore } from '$lib/stores/calendar';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import ComposeWorkspace from '$lib/components/composer/ComposeWorkspace.svelte';
	import DraftEmptyState from './DraftEmptyState.svelte';
	import type { SyncStatus } from '$lib/utils/composerAutosave';
	import type { ComposeRequest } from '$lib/api';
	import type { PaletteAction } from '$lib/components/CommandPalette.svelte';
	import type { HydrationPayload } from '$lib/utils/draftStudioParse';

	interface Props {
		drawerOpen: boolean;
		hydration: HydrationPayload | null;
		hydrationDraftId: number | null;
		loadingDraft: boolean;
		publishEnabled: boolean;
		selectionSessionId?: string | null;
		onToggleDrawer: () => void;
		onSyncStatus: (status: SyncStatus) => void;
		onDraftAction: (actionId: string) => void;
		onDraftSubmit: (data: ComposeRequest) => void;
		onFetchDraft: (id: number) => void;
		onCreate: () => void;
		onSelectionConsumed?: () => void;
	}

	const {
		drawerOpen,
		hydration,
		hydrationDraftId,
		loadingDraft,
		publishEnabled,
		selectionSessionId = null,
		onToggleDrawer,
		onSyncStatus,
		onDraftAction,
		onDraftSubmit,
		onFetchDraft,
		onCreate,
		onSelectionConsumed,
	}: Props = $props();

	const paletteActions: PaletteAction[] = [
		{
			id: 'ds-new-draft',
			label: 'New draft',
			icon: Plus,
			category: 'DraftStudio',
			shortcut: 'n',
			when: 'always',
		},
		{
			id: 'ds-duplicate',
			label: 'Duplicate current draft',
			icon: Copy,
			category: 'DraftStudio',
			shortcut: 'd',
			when: 'always',
		},
		{
			id: 'ds-delete',
			label: 'Delete current draft',
			icon: Trash2,
			category: 'DraftStudio',
			shortcut: 'backspace',
			when: 'always',
		},
	];
</script>

<div class="composer-zone">
	{#snippet headerLeftControls()}
		<button
			class="drafts-toggle-btn"
			type="button"
			onclick={onToggleDrawer}
			title="Toggle drafts (⌘⇧O)"
			class:active={drawerOpen}
		>
			<Files size={15} />
			<span>Drafts</span>
			{#if studio.getTabCounts().active > 0}
				<span class="draft-count">{studio.getTabCounts().active}</span>
			{/if}
		</button>

		<button
			class="bar-action"
			type="button"
			onclick={onCreate}
			title="New draft"
		>
			<Plus size={14} />
			<span>New</span>
		</button>
	{/snippet}

	{#if studio.getSelectedId() === null || (loadingDraft && !hydration)}
		<div class="composer-bar">
			{@render headerLeftControls()}
		</div>
	{/if}

	{#if studio.getError()}
		<div class="error-banner">
			<span>{studio.getError()}</span>
			<button type="button" onclick={() => studio.clearError()}>Dismiss</button>
		</div>
	{/if}

	{#if studio.isLoading() && !studio.getSelectedDraft()}
		<div class="zone-loading">
			<div class="zone-spinner"></div>
		</div>
	{:else if studio.getSelectedId() !== null}
		{#if loadingDraft}
			<div class="zone-loading">
				<div class="zone-spinner"></div>
			</div>
		{:else if hydration && hydrationDraftId !== null}
			{#key hydrationDraftId}
				<ComposeWorkspace
					draftId={hydrationDraftId}
					initialContent={hydration}
					embedded={true}
					schedule={$scheduleStore}
					canPublish={publishEnabled}
					{selectionSessionId}
					onsubmit={onDraftSubmit}
					onsyncstatus={onSyncStatus}
					extraPaletteActions={paletteActions}
					ondraftaction={onDraftAction}
					headerLeft={headerLeftControls}
					onSelectionConsumed={onSelectionConsumed}
				/>
			{/key}
		{:else}
			<div class="zone-error">
				<p>Failed to load draft content.</p>
				<button
					type="button"
					onclick={() => {
						const id = studio.getSelectedId();
						if (id !== null) onFetchDraft(id);
					}}
				>
					Retry
				</button>
			</div>
		{/if}
	{:else if studio.getTabCounts().active === 0 && studio.getTabCounts().scheduled === 0}
		<DraftEmptyState variant="no-drafts" oncreate={onCreate} />
	{:else}
		<DraftEmptyState variant="no-selection" oncreate={onCreate} />
	{/if}
</div>

<style>
	.composer-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow-y: auto;
		background: var(--color-base);
		grid-column: 1;
	}

	.composer-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
		flex-shrink: 0;
	}

	.drafts-toggle-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.drafts-toggle-btn:hover,
	.drafts-toggle-btn.active {
		background: var(--color-surface-active);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.draft-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 16px;
		height: 16px;
		padding: 0 4px;
		border-radius: 8px;
		background: var(--color-accent);
		color: #fff;
		font-size: 10px;
		font-weight: 700;
		line-height: 1;
	}

	.bar-action {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.bar-action:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
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
		border-bottom: 1px solid
			color-mix(in srgb, var(--color-danger) 20%, transparent);
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
		to {
			transform: rotate(360deg);
		}
	}
</style>
