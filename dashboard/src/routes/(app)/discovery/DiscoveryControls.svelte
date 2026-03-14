<script lang="ts">
	import { Search } from 'lucide-svelte';

	interface Props {
		minScore: number;
		maxScore: number | undefined;
		keyword: string;
		keywords: string[];
		limit: number;
		loading: boolean;
		onRefresh: () => void;
	}

	let {
		minScore = $bindable(),
		maxScore = $bindable(),
		keyword = $bindable(),
		keywords,
		limit = $bindable(),
		loading,
		onRefresh,
	}: Props = $props();
</script>

<div class="controls">
	<label class="score-control">
		<span class="control-label">Min score</span>
		<input
			type="number"
			class="score-input"
			bind:value={minScore}
			min="0"
			max="100"
			step="5"
		/>
	</label>
	<label class="score-control">
		<span class="control-label">Max score</span>
		<input
			type="number"
			class="score-input"
			bind:value={maxScore}
			min="0"
			max="100"
			step="5"
			placeholder="—"
		/>
	</label>
	{#if keywords.length > 0}
		<label class="score-control">
			<span class="control-label">Keyword</span>
			<select class="keyword-select" bind:value={keyword}>
				<option value="">All</option>
				{#each keywords as kw}
					<option value={kw}>{kw}</option>
				{/each}
			</select>
		</label>
	{/if}
	<label class="score-control">
		<span class="control-label">Limit</span>
		<input
			type="number"
			class="score-input"
			bind:value={limit}
			min="5"
			max="100"
			step="5"
		/>
	</label>
	<button class="refresh-btn" onclick={onRefresh} disabled={loading}>
		<Search size={14} />
		Refresh
	</button>
</div>

<style>
	.controls {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.score-control {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.control-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.score-input {
		width: 64px;
		padding: 6px 8px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
		text-align: center;
	}

	.keyword-select {
		padding: 6px 8px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
		min-width: 100px;
	}

	.refresh-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.refresh-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
	}

	.refresh-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
