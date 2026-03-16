<script lang="ts">
	import { DollarSign, BarChart3, Hash } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import { summary, dailyCosts, modelBreakdown, typeBreakdown } from '$lib/stores/costs';
	import { formatCost, formatTokens } from '$lib/utils/costFormatters';

	const maxDailyCost = $derived(
		$dailyCosts.length > 0 ? Math.max(...$dailyCosts.map((d) => d.cost), 0.001) : 1,
	);
</script>

<div class="stat-grid">
	<StatCard label="Cost Today" value={formatCost($summary?.cost_today ?? 0)}>
		{#snippet icon()}<DollarSign size={18} />{/snippet}
	</StatCard>
	<StatCard label="Cost (7 days)" value={formatCost($summary?.cost_7d ?? 0)}>
		{#snippet icon()}<BarChart3 size={18} />{/snippet}
	</StatCard>
	<StatCard label="Cost (30 days)" value={formatCost($summary?.cost_30d ?? 0)}>
		{#snippet icon()}<DollarSign size={18} />{/snippet}
	</StatCard>
	<StatCard label="Total API Calls" value={$summary?.calls_all_time ?? 0}>
		{#snippet icon()}<Hash size={18} />{/snippet}
	</StatCard>
</div>

{#if $dailyCosts.length > 0}
	<section class="card">
		<h2>Daily Cost</h2>
		<div class="chart">
			{#each $dailyCosts as day}
				<div
					class="chart-bar-group"
					title="{day.date}: {formatCost(day.cost)} ({day.calls} calls)"
				>
					<div
						class="chart-bar"
						style="height: {Math.max((day.cost / maxDailyCost) * 100, 2)}%"
					></div>
					<span class="chart-label">{day.date.slice(5)}</span>
				</div>
			{/each}
		</div>
	</section>
{/if}

{#if $typeBreakdown.length > 0}
	<section class="card">
		<h2>By Generation Type</h2>
		<div class="table-wrapper">
			<table>
				<thead>
					<tr>
						<th>Type</th>
						<th class="right">Calls</th>
						<th class="right">Total Cost</th>
						<th class="right">Avg Cost / Call</th>
					</tr>
				</thead>
				<tbody>
					{#each $typeBreakdown as row}
						<tr>
							<td class="type-label">{row.generation_type}</td>
							<td class="right">{row.calls}</td>
							<td class="right">{formatCost(row.cost)}</td>
							<td class="right">{formatCost(row.avg_cost)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{/if}

{#if $modelBreakdown.length > 0}
	<section class="card">
		<h2>By Model</h2>
		<div class="table-wrapper">
			<table>
				<thead>
					<tr>
						<th>Provider</th>
						<th>Model</th>
						<th class="right">Calls</th>
						<th class="right">Input Tokens</th>
						<th class="right">Output Tokens</th>
						<th class="right">Total Cost</th>
					</tr>
				</thead>
				<tbody>
					{#each $modelBreakdown as row}
						<tr>
							<td>{row.provider}</td>
							<td class="model-name">{row.model}</td>
							<td class="right">{row.calls}</td>
							<td class="right">{formatTokens(row.input_tokens)}</td>
							<td class="right">{formatTokens(row.output_tokens)}</td>
							<td class="right">{formatCost(row.cost)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{/if}

<style>
	.stat-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
	}

	@media (max-width: 800px) {
		.stat-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	h2 {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 14px 0;
	}

	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.chart {
		display: flex;
		align-items: flex-end;
		gap: 3px;
		height: 140px;
		padding-top: 8px;
	}

	.chart-bar-group {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		height: 100%;
		justify-content: flex-end;
		min-width: 0;
	}

	.chart-bar {
		width: 100%;
		max-width: 20px;
		background-color: var(--color-accent);
		border-radius: 3px 3px 0 0;
		transition: height 0.3s ease;
		min-height: 2px;
	}

	.chart-label {
		font-size: 9px;
		color: var(--color-text-subtle);
		margin-top: 4px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100%;
	}

	.table-wrapper {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	th {
		text-align: left;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	td {
		padding: 10px 12px;
		color: var(--color-text);
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.right {
		text-align: right;
	}

	.type-label {
		text-transform: capitalize;
		font-weight: 500;
	}

	.model-name {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
	}

	@media (prefers-reduced-motion: reduce) {
		.chart-bar {
			transition: none;
		}
	}
</style>
