<script lang="ts">
	import { Plus, Loader2 } from 'lucide-svelte';
	import type { DraftSummary } from '$lib/api/types';
	import DraftRailEntry from './DraftRailEntry.svelte';

	let {
		drafts,
		selectedId,
		tab,
		tabCounts,
		loading,
		onselect,
		ontabchange,
		oncreate
	}: {
		drafts: DraftSummary[];
		selectedId: number | null;
		tab: 'active' | 'scheduled' | 'archive';
		tabCounts: { active: number; scheduled: number; archive: number };
		loading: boolean;
		onselect: (id: number) => void;
		ontabchange: (tab: 'active' | 'scheduled' | 'archive') => void;
		oncreate: () => void;
	} = $props();

	const tabs: Array<{ key: 'active' | 'scheduled' | 'archive'; label: string }> = [
		{ key: 'active', label: 'Active' },
		{ key: 'scheduled', label: 'Scheduled' },
		{ key: 'archive', label: 'Archive' }
	];
</script>

<div class="rail">
	<div class="rail-tabs">
		{#each tabs as t}
			<button
				class="tab-btn"
				class:active={tab === t.key}
				type="button"
				onclick={() => ontabchange(t.key)}
			>
				{t.label}
				<span class="tab-count">{tabCounts[t.key]}</span>
			</button>
		{/each}
	</div>

	<div class="rail-list">
		{#if loading}
			<div class="rail-loading">
				<Loader2 size={16} class="spinner" />
			</div>
		{:else if drafts.length === 0}
			<div class="rail-empty">
				<span class="rail-empty-text">
					{#if tab === 'archive'}
						No archived drafts
					{:else if tab === 'scheduled'}
						No scheduled drafts
					{:else}
						No drafts yet
					{/if}
				</span>
			</div>
		{:else}
			{#each drafts as draft (draft.id)}
				<DraftRailEntry
					{draft}
					selected={selectedId === draft.id}
					onclick={() => onselect(draft.id)}
				/>
			{/each}
		{/if}
	</div>

	<div class="rail-footer">
		<button class="new-draft-btn" type="button" onclick={oncreate}>
			<Plus size={16} />
			<span>New Draft</span>
		</button>
	</div>
</div>

<style>
	.rail {
		display: flex;
		flex-direction: column;
		height: 100%;
		border-right: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
	}

	.rail-tabs {
		display: flex;
		flex-shrink: 0;
		border-bottom: 1px solid var(--color-border-subtle);
		padding: 0 4px;
	}

	.tab-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 10px 4px;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
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
		font-weight: 700;
		font-family: var(--font-mono);
		opacity: 0.7;
	}

	.rail-list {
		flex: 1;
		overflow-y: auto;
		padding: 4px;
	}

	.rail-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 40px 0;
		color: var(--color-text-subtle);
	}

	.rail-loading :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	.rail-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 40px 12px;
	}

	.rail-empty-text {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.rail-footer {
		flex-shrink: 0;
		padding: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.new-draft-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		width: 100%;
		padding: 8px 12px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.new-draft-btn:hover {
		background: var(--color-accent-hover);
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
