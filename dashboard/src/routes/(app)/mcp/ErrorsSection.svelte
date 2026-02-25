<script lang="ts">
	import { CheckCircle } from 'lucide-svelte';
	import { errors } from '$lib/stores/mcp';

	let { formatTime, formatDate }: { formatTime: (s: string) => string; formatDate: (s: string) => string } = $props();
</script>

{#if $errors.length > 0}
	<section class="card">
		<h2>Error Breakdown</h2>
		<div class="table-wrapper">
			<table>
				<thead>
					<tr>
						<th>Tool</th>
						<th>Error Code</th>
						<th class="right">Count</th>
						<th class="right">Last Seen</th>
					</tr>
				</thead>
				<tbody>
					{#each $errors as err}
						<tr>
							<td class="tool-name">{err.tool_name}</td>
							<td><span class="error-badge">{err.error_code}</span></td>
							<td class="right">{err.count}</td>
							<td class="right text-muted">
								{formatDate(err.latest_at)} {formatTime(err.latest_at)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</section>
{:else}
	<div class="empty-state">
		<CheckCircle size={32} />
		<p>No errors recorded in this time window.</p>
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
	.error-badge { font-family: var(--font-mono, monospace); font-size: 11px; padding: 2px 8px; border-radius: 3px; background: color-mix(in srgb, var(--color-danger) 12%, transparent); color: var(--color-danger); }
	.text-muted { color: var(--color-text-subtle); }
	.empty-state { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 48px 24px; color: var(--color-text-muted); text-align: center; }
	.empty-state p { margin: 0; font-size: 14px; }
</style>
