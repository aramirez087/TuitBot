<script lang="ts">
	import { Clock, CheckCircle, XCircle } from 'lucide-svelte';
	import { recentExecutions } from '$lib/stores/mcp';

	let { formatTime, formatDate, formatLatency }: {
		formatTime: (s: string) => string;
		formatDate: (s: string) => string;
		formatLatency: (n: number) => string;
	} = $props();
</script>

{#if $recentExecutions.length > 0}
	<section class="card">
		<h2>Recent Executions</h2>
		<div class="table-wrapper">
			<table>
				<thead>
					<tr>
						<th>Time</th>
						<th>Tool</th>
						<th>Category</th>
						<th>Status</th>
						<th>Policy</th>
						<th class="right">Latency</th>
					</tr>
				</thead>
				<tbody>
					{#each $recentExecutions as entry}
						<tr>
							<td class="text-muted">
								{formatDate(entry.created_at)}
								{formatTime(entry.created_at)}
							</td>
							<td class="tool-name">{entry.tool_name}</td>
							<td>
								<span class="category-badge" class:mutation={entry.category === 'mutation'}>
									{entry.category}
								</span>
							</td>
							<td>
								{#if entry.success}
									<span class="status-badge success">
										<CheckCircle size={12} />
										OK
									</span>
								{:else}
									<span class="status-badge failure">
										<XCircle size={12} />
										{entry.error_code ?? 'Error'}
									</span>
								{/if}
							</td>
							<td>
								{#if entry.policy_decision}
									<span
										class="policy-badge"
										class:allow={entry.policy_decision === 'allow'}
										class:deny={entry.policy_decision === 'deny'}
										class:dry-run={entry.policy_decision === 'dry_run'}
										class:approval={entry.policy_decision === 'route_to_approval'}
									>
										{entry.policy_decision}
									</span>
								{:else}
									<span class="text-muted">-</span>
								{/if}
							</td>
							<td class="right">{formatLatency(entry.latency_ms)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{:else}
	<div class="empty-state">
		<Clock size={32} />
		<p>No recent executions recorded.</p>
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
	.status-badge { display: inline-flex; align-items: center; gap: 4px; font-size: 12px; font-weight: 500; }
	.status-badge.success { color: var(--color-success); }
	.status-badge.failure { color: var(--color-danger); }
	.policy-badge { display: inline-block; padding: 2px 8px; border-radius: 3px; font-size: 11px; font-weight: 600; background: var(--color-surface-active); color: var(--color-text-muted); }
	.policy-badge.allow { background: color-mix(in srgb, var(--color-success) 12%, transparent); color: var(--color-success); }
	.policy-badge.deny { background: color-mix(in srgb, var(--color-danger) 12%, transparent); color: var(--color-danger); }
	.policy-badge.dry-run { background: color-mix(in srgb, var(--color-warning) 12%, transparent); color: var(--color-warning); }
	.policy-badge.approval { background: color-mix(in srgb, var(--color-accent) 12%, transparent); color: var(--color-accent); }
	.text-muted { color: var(--color-text-subtle); }
	.empty-state { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 48px 24px; color: var(--color-text-muted); text-align: center; }
	.empty-state p { margin: 0; font-size: 14px; }
</style>
