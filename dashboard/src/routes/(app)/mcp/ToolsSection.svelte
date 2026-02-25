<script lang="ts">
	import { Activity } from 'lucide-svelte';
	import { metrics } from '$lib/stores/mcp';

	let { formatRate, formatLatency }: { formatRate: (n: number) => string; formatLatency: (n: number) => string } = $props();
</script>

{#if $metrics.length > 0}
	<section class="card">
		<h2>Tool Performance</h2>
		<div class="table-wrapper">
			<table>
				<thead>
					<tr>
						<th>Tool</th>
						<th>Category</th>
						<th class="right">Calls</th>
						<th class="right">Success</th>
						<th class="right">Failures</th>
						<th class="right">Rate</th>
						<th class="right">Avg</th>
						<th class="right">P50</th>
						<th class="right">P95</th>
					</tr>
				</thead>
				<tbody>
					{#each $metrics as tool}
						<tr>
							<td class="tool-name">{tool.tool_name}</td>
							<td>
								<span class="category-badge" class:mutation={tool.category === 'mutation'}>
									{tool.category}
								</span>
							</td>
							<td class="right">{tool.total_calls}</td>
							<td class="right text-success">{tool.success_count}</td>
							<td class="right">
								{#if tool.failure_count > 0}
									<span class="text-danger">{tool.failure_count}</span>
								{:else}
									<span class="text-muted">0</span>
								{/if}
							</td>
							<td class="right">
								<span
									class:text-success={tool.success_rate >= 0.95}
									class:text-warning={tool.success_rate >= 0.8 && tool.success_rate < 0.95}
									class:text-danger={tool.success_rate < 0.8}
								>
									{formatRate(tool.success_rate)}
								</span>
							</td>
							<td class="right">{formatLatency(tool.avg_latency_ms)}</td>
							<td class="right">{formatLatency(tool.p50_latency_ms)}</td>
							<td class="right">{formatLatency(tool.p95_latency_ms)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{:else}
	<div class="empty-state">
		<Activity size={32} />
		<p>No tool executions recorded in this time window.</p>
	</div>
{/if}

<style>
	.card { padding: 18px; background-color: var(--color-surface); border: 1px solid var(--color-border-subtle); border-radius: 8px; }
	h2 { font-size: 14px; font-weight: 600; color: var(--color-text); margin: 0 0 14px 0; }
	.table-wrapper { overflow-x: auto; }
	table { width: 100%; border-collapse: collapse; font-size: 13px; }
	th { text-align: left; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-subtle); padding: 8px 12px; border-bottom: 1px solid var(--color-border-subtle); }
	td { padding: 10px 12px; color: var(--color-text); border-bottom: 1px solid var(--color-border-subtle); }
	.right { text-align: right; }
	.tool-name { font-family: var(--font-mono, monospace); font-size: 12px; font-weight: 500; }
	.category-badge { display: inline-block; padding: 2px 8px; border-radius: 3px; font-size: 11px; font-weight: 600; background-color: var(--color-surface-active); color: var(--color-text-muted); }
	.category-badge.mutation { background: color-mix(in srgb, var(--color-warning) 12%, transparent); color: var(--color-warning); }
	.text-success { color: var(--color-success); }
	.text-warning { color: var(--color-warning); }
	.text-danger { color: var(--color-danger); }
	.text-muted { color: var(--color-text-subtle); }
	.empty-state { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 48px 24px; color: var(--color-text-muted); text-align: center; }
	.empty-state p { margin: 0; font-size: 14px; }
</style>
