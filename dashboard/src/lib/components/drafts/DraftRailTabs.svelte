<script lang="ts">
	type TabKey = 'active' | 'scheduled' | 'posted' | 'archive';

	interface Tab {
		key: TabKey;
		label: string;
	}

	interface Props {
		tabs: Tab[];
		tab: TabKey;
		tabCounts: { active: number; scheduled: number; posted: number; archive: number };
		ontabchange: (tab: TabKey) => void;
	}

	const { tabs, tab, tabCounts, ontabchange }: Props = $props();
</script>

<div class="rail-tabs" role="tablist" aria-label="Draft tabs">
	{#each tabs as t}
		<button
			class="tab-btn"
			class:active={tab === t.key}
			type="button"
			role="tab"
			aria-selected={tab === t.key}
			onclick={() => ontabchange(t.key)}
		>
			{t.label}
			<span class="tab-count">{tabCounts[t.key]}</span>
		</button>
	{/each}
</div>

<style>
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
</style>
