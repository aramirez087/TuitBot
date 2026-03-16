<script lang="ts">
	import { DollarSign, BarChart3, Globe, Hash } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import { xApiSummary, xApiDailyCalls, xApiEndpoints } from '$lib/stores/costs';
	import { formatCost, formatNumber } from '$lib/utils/costFormatters';

	const maxDailyXApiCost = $derived(
		$xApiDailyCalls.length > 0 ? Math.max(...$xApiDailyCalls.map((d) => d.cost), 0.001) : 1,
	);
</script>

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
				<div
					class="chart-bar-group"
					title="{day.date}: {formatCost(day.cost)} ({day.calls} calls)"
				>
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
							<td
								><span class="method-badge" class:post={row.method === 'POST'}
									>{row.method}</span
								></td
							>
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
		Costs estimated using X API pay-per-use pricing: $0.005/post read, $0.010/user lookup,
		$0.010/post write. Failed requests (4xx/5xx) are tracked but not billed.
	</p>
</section>

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
		border-radius: 3px 3px 0 0;
		transition: height 0.3s ease;
		min-height: 2px;
		background-color: var(--color-accent);
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

	@media (prefers-reduced-motion: reduce) {
		.chart-bar {
			transition: none;
		}
	}
</style>
