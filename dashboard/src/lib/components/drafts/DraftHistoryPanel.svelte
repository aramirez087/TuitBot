<script lang="ts">
	import type { ContentRevision, ContentActivity } from '$lib/api/types';
	import { X } from 'lucide-svelte';
	import DraftRevisionList from './DraftRevisionList.svelte';
	import DraftActivityList from './DraftActivityList.svelte';

	let {
		revisions = [],
		activity = [],
		timezone = 'UTC',
		onrestore,
		onclose,
	}: {
		revisions: ContentRevision[];
		activity: ContentActivity[];
		timezone?: string;
		onrestore: (revisionId: number) => void;
		onclose: () => void;
	} = $props();

	let activeTab = $state<'revisions' | 'activity'>('revisions');
</script>

<div class="history-panel">
	<header class="panel-header">
		<span class="panel-title">History</span>
		<button type="button" class="close-btn" onclick={onclose} aria-label="Close history">
			<X size={14} />
		</button>
	</header>

	<div class="tab-bar" role="tablist">
		<button
			type="button"
			role="tab"
			class="tab-btn"
			class:active={activeTab === 'revisions'}
			aria-selected={activeTab === 'revisions'}
			onclick={() => (activeTab = 'revisions')}
		>
			Revisions
			{#if revisions.length > 0}
				<span class="tab-count">{revisions.length}</span>
			{/if}
		</button>
		<button
			type="button"
			role="tab"
			class="tab-btn"
			class:active={activeTab === 'activity'}
			aria-selected={activeTab === 'activity'}
			onclick={() => (activeTab = 'activity')}
		>
			Activity
			{#if activity.length > 0}
				<span class="tab-count">{activity.length}</span>
			{/if}
		</button>
	</div>

	<div class="panel-body">
		{#if activeTab === 'revisions'}
			<DraftRevisionList {revisions} {onrestore} />
		{:else}
			<DraftActivityList {activity} {timezone} />
		{/if}
	</div>
</div>

<style>
	.history-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-surface);
		border-left: 1px solid var(--color-border-subtle);
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.panel-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		letter-spacing: -0.01em;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
	}

	.close-btn:hover {
		background: var(--color-border-subtle);
		color: var(--color-text);
	}

	.tab-bar {
		display: flex;
		gap: 0;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.tab-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 5px;
		padding: 8px 0;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s;
	}

	.tab-btn:hover {
		color: var(--color-text);
	}

	.tab-btn.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.tab-count {
		font-size: 10px;
		padding: 1px 5px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		font-weight: 600;
	}

	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}
</style>
