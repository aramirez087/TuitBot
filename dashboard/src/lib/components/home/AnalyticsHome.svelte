<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Users, MessageSquare, FileText, TrendingUp } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import FollowerChart from '$lib/components/FollowerChart.svelte';
	import TopTopics from '$lib/components/TopTopics.svelte';
	import RecentPerformance from '$lib/components/RecentPerformance.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import {
		summary,
		recentPerformance,
		loading,
		error,
		followerCount,
		followerChange7d,
		repliesToday,
		tweetsToday,
		avgEngagement,
		loadDashboard,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/analytics';

	onMount(() => {
		loadDashboard();
		startAutoRefresh();
	});

	onDestroy(() => {
		stopAutoRefresh();
	});
</script>

<svelte:head>
	<title>Dashboard — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>Dashboard</h1>
	<p class="subtitle">Your autonomous growth overview</p>
</div>

{#if $error && !$summary}
	<ErrorState message={$error} onretry={() => loadDashboard()} />
{:else if $error}
	<div class="error-banner">
		<span>{$error}</span>
		<button onclick={() => loadDashboard()}>Retry</button>
	</div>
{/if}

{#if $loading && !$summary}
	<div class="stat-grid">
		{#each Array(4) as _}
			<div class="skeleton-card"></div>
		{/each}
	</div>
	<div class="skeleton-chart"></div>
{:else if !$summary && !$loading && !$error}
	<EmptyState
		title="No analytics data yet"
		description="Start the automation to begin tracking your growth."
		actionLabel="Refresh"
		onaction={() => loadDashboard()}
	/>
{:else}
	<div class="stat-grid">
		<StatCard label="Followers" value={$followerCount.toLocaleString()} change={$followerChange7d}>
			{#snippet icon()}<Users size={20} />{/snippet}
		</StatCard>
		<StatCard label="Replies Today" value={$repliesToday}>
			{#snippet icon()}<MessageSquare size={20} />{/snippet}
		</StatCard>
		<StatCard label="Tweets Today" value={$tweetsToday}>
			{#snippet icon()}<FileText size={20} />{/snippet}
		</StatCard>
		<StatCard label="Avg Engagement" value={$avgEngagement.toFixed(1)}>
			{#snippet icon()}<TrendingUp size={20} />{/snippet}
		</StatCard>
	</div>

	<div class="chart-section">
		<FollowerChart />
	</div>

	<div class="bottom-grid">
		<div class="bottom-left">
			<TopTopics topics={$summary?.top_topics ?? []} />
		</div>
		<div class="bottom-right">
			<RecentPerformance items={$recentPerformance} />
		</div>
	</div>
{/if}

<style>
	.page-header {
		margin-bottom: 24px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background-color: #f8514920;
		border: 1px solid var(--color-danger);
		border-radius: 8px;
		margin-bottom: 20px;
		color: var(--color-danger);
		font-size: 13px;
	}

	.error-banner button {
		padding: 4px 12px;
		border: 1px solid var(--color-danger);
		border-radius: 4px;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}

	.error-banner button:hover {
		background-color: #f8514920;
	}

	.stat-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 12px;
		margin-bottom: 24px;
	}

	.chart-section {
		margin-bottom: 24px;
	}

	.bottom-grid {
		display: grid;
		grid-template-columns: 1fr 2fr;
		gap: 16px;
	}

	@media (max-width: 800px) {
		.bottom-grid {
			grid-template-columns: 1fr;
		}
	}

	/* Skeleton placeholders */
	.skeleton-card {
		height: 80px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-chart {
		height: 300px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		margin-bottom: 24px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}
</style>
