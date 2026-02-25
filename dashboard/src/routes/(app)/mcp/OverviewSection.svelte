<script lang="ts">
	import { Hash, CheckCircle, Gauge, Activity, Shield } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import { summary } from '$lib/stores/mcp';

	let { formatRate, formatLatency }: { formatRate: (n: number) => string; formatLatency: (n: number) => string } = $props();

	const policyDecisionCounts = $derived(() => {
		if (!$summary) return [];
		return Object.entries($summary.policy_decisions)
			.sort((a, b) => b[1] - a[1])
			.map(([decision, count]) => ({ decision, count }));
	});
</script>

<div class="stat-grid">
	<StatCard label="Total Calls" value={$summary?.total_calls ?? 0}>
		{#snippet icon()}<Hash size={18} />{/snippet}
	</StatCard>
	<StatCard label="Success Rate" value={formatRate($summary?.overall_success_rate ?? 0)}>
		{#snippet icon()}<CheckCircle size={18} />{/snippet}
	</StatCard>
	<StatCard label="Avg Latency" value={formatLatency($summary?.avg_latency_ms ?? 0)}>
		{#snippet icon()}<Gauge size={18} />{/snippet}
	</StatCard>
	<StatCard label="Unique Tools" value={$summary?.unique_tools ?? 0}>
		{#snippet icon()}<Activity size={18} />{/snippet}
	</StatCard>
</div>

{#if policyDecisionCounts().length > 0}
	<section class="card">
		<h2>
			<Shield size={16} />
			Policy Decisions
		</h2>
		<div class="decision-grid">
			{#each policyDecisionCounts() as { decision, count }}
				<div class="decision-item">
					<span class="decision-label">{decision}</span>
					<span class="decision-count">{count}</span>
				</div>
			{/each}
		</div>
	</section>
{/if}

<style>
	.stat-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
	}

	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	h2 {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 14px 0;
	}

	.decision-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
		gap: 8px;
	}

	.decision-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--color-surface-active);
		border-radius: 6px;
	}

	.decision-label {
		font-size: 12px;
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	.decision-count {
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text);
		font-family: var(--font-mono, monospace);
	}

	@media (max-width: 800px) {
		.stat-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>
