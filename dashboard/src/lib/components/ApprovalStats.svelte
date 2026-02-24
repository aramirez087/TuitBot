<script lang="ts">
	import type { ApprovalStats } from '$lib/api';

	interface Props {
		stats: ApprovalStats | null;
	}

	let { stats }: Props = $props();
</script>

{#if stats}
	<div class="stats-bar">
		<span class="stat pending">{stats.pending} pending</span>
		<span class="stat-separator">&middot;</span>
		<span class="stat approved">{stats.approved} approved</span>
		<span class="stat-separator">&middot;</span>
		<span class="stat rejected">{stats.rejected} rejected</span>
	</div>
{/if}

<style>
	.stats-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.stat {
		font-weight: 600;
	}

	.stat.pending {
		color: var(--color-warning);
	}

	.stat.approved {
		color: var(--color-success);
	}

	.stat.rejected {
		color: var(--color-danger);
	}

	.stat-separator {
		color: var(--color-text-subtle);
	}
</style>
