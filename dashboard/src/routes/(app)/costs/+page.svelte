<script lang="ts">
	import { onMount } from 'svelte';
	import { DollarSign, BarChart3, Cpu, Hash } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import {
		summary,
		dailyCosts,
		modelBreakdown,
		typeBreakdown,
		loading,
		error,
		loadCosts
	} from '$lib/stores/costs';

	let days = $state(30);

	onMount(() => {
		loadCosts(days);
	});

	function handleDaysChange(newDays: number) {
		days = newDays;
		loadCosts(days);
	}

	function formatCost(value: number): string {
		if (value < 0.01) return '$' + value.toFixed(4);
		if (value < 1) return '$' + value.toFixed(3);
		return '$' + value.toFixed(2);
	}

	function formatTokens(value: number): string {
		if (value >= 1_000_000) return (value / 1_000_000).toFixed(1) + 'M';
		if (value >= 1_000) return (value / 1_000).toFixed(1) + 'K';
		return value.toString();
	}

	const maxDailyCost = $derived(
		$dailyCosts.length > 0 ? Math.max(...$dailyCosts.map((d) => d.cost), 0.001) : 1
	);
</script>

<div class="page">
	<div class="page-header">
		<h1>LLM Costs</h1>
		<div class="period-selector">
			<button
				class="period-btn"
				class:active={days === 7}
				onclick={() => handleDaysChange(7)}
			>7d</button>
			<button
				class="period-btn"
				class:active={days === 30}
				onclick={() => handleDaysChange(30)}
			>30d</button>
			<button
				class="period-btn"
				class:active={days === 90}
				onclick={() => handleDaysChange(90)}
			>90d</button>
		</div>
	</div>

	{#if $error}
		<ErrorState message={$error} onretry={() => loadCosts(days)} />
	{:else if $loading}
		<div class="loading">Loading cost data...</div>
	{:else}
		<!-- Summary Cards -->
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

		<!-- Daily Cost Chart -->
		{#if $dailyCosts.length > 0}
			<section class="card">
				<h2>Daily Cost</h2>
				<div class="chart">
					{#each $dailyCosts as day}
						<div class="chart-bar-group" title="{day.date}: {formatCost(day.cost)} ({day.calls} calls)">
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

		<!-- By Type Table -->
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

		<!-- By Model Table -->
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
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 1000px;
	}

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	h1 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	h2 {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 14px 0;
	}

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

	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.period-selector {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.period-btn {
		padding: 4px 12px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.period-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.period-btn:hover:not(.active) {
		color: var(--color-text);
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

	.loading {
		text-align: center;
		padding: 48px 0;
		color: var(--color-text-muted);
		font-size: 14px;
	}
</style>
