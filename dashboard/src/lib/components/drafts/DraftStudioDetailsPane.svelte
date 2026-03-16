<script lang="ts">
	import { schedule as scheduleStore } from '$lib/stores/calendar';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import DraftDetailsPanel from './DraftDetailsPanel.svelte';
	import DraftHistoryPanel from './DraftHistoryPanel.svelte';

	interface Props {
		activePanel: 'details' | 'history';
		prefillSchedule: string | null;
		onActivePanel: (panel: 'details' | 'history') => void;
		onUpdateMeta: (data: { title?: string; notes?: string }) => void;
		onAssignTag: (tagId: number) => void;
		onUnassignTag: (tagId: number) => void;
		onCreateTag: (name: string) => void;
		onSchedule: (scheduledFor: string) => void;
		onUnschedule: () => void;
		onReschedule: (scheduledFor: string) => void;
		onDuplicate: () => void;
		onRestoreFromRevision: (revisionId: number) => void;
		onClose: () => void;
	}

	const {
		activePanel,
		prefillSchedule,
		onActivePanel,
		onUpdateMeta,
		onAssignTag,
		onUnassignTag,
		onCreateTag,
		onSchedule,
		onUnschedule,
		onReschedule,
		onDuplicate,
		onRestoreFromRevision,
		onClose,
	}: Props = $props();

	function switchToHistory() {
		onActivePanel('history');
		studio.loadRevisions();
		studio.loadActivity();
	}
</script>

<div class="details-zone">
	<div class="panel-switcher">
		<button
			type="button"
			class="panel-tab"
			class:active={activePanel === 'details'}
			onclick={() => onActivePanel('details')}
		>
			Details
		</button>
		<button
			type="button"
			class="panel-tab"
			class:active={activePanel === 'history'}
			onclick={switchToHistory}
		>
			History
		</button>
	</div>

	{#if activePanel === 'details'}
		<DraftDetailsPanel
			draft={studio.getFullDraft()}
			draftSummary={studio.getSelectedDraft()}
			tags={studio.getSelectedDraftTags()}
			allTags={studio.getAccountTags()}
			{prefillSchedule}
			timezone={$scheduleStore?.timezone ?? 'UTC'}
			preferredTimes={$scheduleStore?.preferred_times ?? []}
			onupdatemeta={onUpdateMeta}
			onassigntag={onAssignTag}
			onunassigntag={onUnassignTag}
			oncreatetag={onCreateTag}
			onschedule={onSchedule}
			onunschedule={onUnschedule}
			onreschedule={onReschedule}
			onduplicate={onDuplicate}
			onclose={onClose}
		/>
	{:else}
		<DraftHistoryPanel
			revisions={studio.getRevisions()}
			activity={studio.getActivity()}
			timezone={$scheduleStore?.timezone ?? 'UTC'}
			onrestore={onRestoreFromRevision}
			onclose={onClose}
		/>
	{/if}
</div>

<style>
	.details-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
		border-left: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
	}

	.panel-switcher {
		display: flex;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
		background: var(--color-surface);
	}

	.panel-tab {
		flex: 1;
		padding: 8px 0;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.panel-tab:hover {
		color: var(--color-text);
	}

	.panel-tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}
</style>
