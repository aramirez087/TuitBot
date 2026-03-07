<script lang="ts">
	import { onMount } from 'svelte';
	import { DollarSign, BarChart3, Cpu, Hash, Globe, AlertTriangle } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import {
		summary,
		dailyCosts,
		modelBreakdown,
		typeBreakdown,
		xApiSummary,
		xApiDailyCalls,
		xApiEndpoints,
		loading,
		error,
		loadCosts
	} from '$lib/stores/costs';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

	let days = $state(30);
	let activeTab = $state<'llm' | 'xapi'>('llm');

	onMount(() => {
		loadCosts(days);
		const handler = () => loadCosts(days);
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
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

	function formatNumber(value: number): string {
		if (value >= 1_000_000) return (value / 1_000_000).toFixed(1) + 'M';
		if (value >= 1_000) return (value / 1_000).toFixed(1) + 'K';
		return value.toLocaleString();
	}

	const maxDailyCost = $derived(
		$dailyCosts.length > 0 ? Math.max(...$dailyCosts.map((d) => d.cost), 0.001) : 1
	);

	const maxDailyXApiCost = $derived(
		$xApiDailyCalls.length > 0 ? Math.max(...$xApiDailyCalls.map((d) => d.cost), 0.001) : 1
	);
</script>

<div class="page">
	<div class="page-header">
		<h1>Costs</h1>
		<div class="header-controls">
			<div class="tab-switcher">
				<button
					class="tab-btn"
					class:active={activeTab === 'llm'}
					onclick={() => (activeTab = 'llm')}
				>LLM</button>
				<button
					class="tab-btn"
					class:active={activeTab === 'xapi'}
					onclick={() => (activeTab = 'xapi')}
				>X API</button>
			</div>
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
	</div>

	{#if $error}
		<ErrorState message={$error} onretry={() => loadCosts(days)} />
	{:else if $loading}
		<div class="loading">Loading cost data...</div>
	{:else if activeTab === 'llm'}
		<!-- LLM Tab -->
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
	{:else}
		<!-- X API Tab -->
		<div class="stat-grid">
			<StatCard label="Cost Today" value={formatCost($xApiSummary?.cost_today ?? 0)}>
				{#snippet icon()}<DollarSign size={18} />{/snippet}
			</StatCard>
			<StatCard label="Cost (7 days)" value={formatCost($xApiSummary?.cost_7d ?? 0)}>
				{#snippet icon()}<BarChart3 size={18} />{/snippet}
			</StatCard>
			<StatCard label="Calls (30 days)" value={formatNumber($xApiSummary?.calls_30d ?? 0)}>
				{#snippet icon()}<Globe size={18} />{/snippet}
			</StatCard>
			<StatCard label="Total Calls" value={formatNumber($xApiSummary?.calls_all_time ?? 0)}>
				{#snippet icon()}<Hash size={18} />{/snippet}
			</StatCard>
		</div>

		{#if $xApiDailyCalls.length > 0}
			<section class="card">
				<h2>Daily Cost</h2>
				<div class="chart">
					{#each $xApiDailyCalls as day}
						<div class="chart-bar-group" title="{day.date}: {formatCost(day.cost)} ({day.calls} calls)">
							<div
								class="chart-bar xapi"
								style="height: {Math.max((day.cost / maxDailyXApiCost) * 100, 2)}%"
							></div>
							<span class="chart-label">{day.date.slice(5)}</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		{#if $xApiEndpoints.length > 0}
			<section class="card">
				<h2>By Endpoint</h2>
				<div class="table-wrapper">
					<table>
						<thead>
							<tr>
								<th>Endpoint</th>
								<th>Method</th>
								<th class="right">Calls</th>
								<th class="right">Cost</th>
								<th class="right">Errors</th>
							</tr>
						</thead>
						<tbody>
							{#each $xApiEndpoints as row}
								<tr>
									<td class="endpoint-name">{row.endpoint}</td>
									<td><span class="method-badge" class:post={row.method === 'POST'}>{row.method}</span></td>
									<td class="right">{formatNumber(row.calls)}</td>
									<td class="right">{formatCost(row.cost)}</td>
									<td class="right">
										{#if row.error_count > 0}
											<span class="error-count">{row.error_count}</span>
										{:else}
											<span class="text-muted">0</span>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{/if}

		<section class="card info-card">
			<p class="info-text">
				Costs estimated using X API pay-per-use pricing: $0.005/post read, $0.010/user lookup, $0.010/post write. Failed requests (4xx/5xx) are tracked but not billed.
			</p>
		</section>
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

	.header-controls {
		display: flex;
		gap: 8px;
		align-items: center;
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

		.page-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 12px;
		}
	}

	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.tab-switcher {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.tab-btn {
		padding: 4px 14px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.tab-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.tab-btn:hover:not(.active) {
		color: var(--color-text);
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

	.chart-bar.xapi {
		background-color: var(--color-info, #3b82f6);
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

	.model-name,
	.endpoint-name {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
	}

	.method-badge {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 3px;
		font-size: 11px;
		font-weight: 600;
		font-family: var(--font-mono, monospace);
		background-color: var(--color-surface-active);
		color: var(--color-text-muted);
	}

	.method-badge.post {
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.error-count {
		color: var(--color-error, #ef4444);
		font-weight: 600;
	}

	.text-muted {
		color: var(--color-text-subtle);
	}

	.info-card {
		background-color: color-mix(in srgb, var(--color-info, #3b82f6) 5%, var(--color-surface));
		border-color: color-mix(in srgb, var(--color-info, #3b82f6) 20%, var(--color-border-subtle));
	}

	.info-text {
		margin: 0;
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.loading {
		text-align: center;
		padding: 48px 0;
		color: var(--color-text-muted);
		font-size: 14px;
	}
</style>
