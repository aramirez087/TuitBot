<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Users,
		TrendingUp,
		MessageSquare,
		BarChart3,
		Percent,
		Star,
		RefreshCw
	} from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import GrowthLoopDiagram from '$lib/components/GrowthLoopDiagram.svelte';
	import RecommendationCard from '$lib/components/RecommendationCard.svelte';
	import WeeklyTrendChart from '$lib/components/WeeklyTrendChart.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import {
		currentReport,
		reportHistory,
		inputs,
		loading,
		error,
		recommendations,
		loadStrategy,
		refreshReport
	} from '$lib/stores/strategy';

	let refreshing = $state(false);

	async function handleRefresh() {
		refreshing = true;
		await refreshReport();
		refreshing = false;
	}

	onMount(() => {
		loadStrategy();
	});

	const totalOutput = $derived(
		($currentReport?.replies_sent ?? 0) +
			($currentReport?.tweets_posted ?? 0) +
			($currentReport?.threads_posted ?? 0)
	);

	const avgScore = $derived(() => {
		if (!$currentReport) return 0;
		const scores = [$currentReport.avg_reply_score, $currentReport.avg_tweet_score].filter(
			(s) => s > 0
		);
		return scores.length > 0 ? scores.reduce((a, b) => a + b, 0) / scores.length : 0;
	});

	const topTopicName = $derived(
		$currentReport?.top_topics?.[0]?.topic ?? 'N/A'
	);
</script>

<svelte:head>
	<title>Strategy — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<div class="header-left">
		<h1>Strategy</h1>
		<p class="subtitle">Your weekly growth scorecard and recommendations</p>
	</div>
	<button class="refresh-btn" onclick={handleRefresh} disabled={refreshing}>
		<RefreshCw size={14} class={refreshing ? 'spinning' : ''} />
		{refreshing ? 'Refreshing...' : 'Refresh'}
	</button>
</div>

{#if $error && !$currentReport}
	<ErrorState message={$error} onretry={() => loadStrategy()} />
{:else if $error}
	<div class="error-banner">
		<span>{$error}</span>
		<button onclick={() => loadStrategy()}>Retry</button>
	</div>
{/if}

{#if $loading && !$currentReport}
	<div class="skeleton-card tall"></div>
	<div class="stat-grid">
		{#each Array(6) as _}
			<div class="skeleton-card"></div>
		{/each}
	</div>
{:else if !$currentReport && !$loading && !$error}
	<EmptyState
		title="No strategy data yet"
		description="Start the automation to begin collecting growth metrics."
		actionLabel="Refresh"
		onaction={() => loadStrategy()}
	/>
{:else}
	<!-- 1. Growth Loop Diagram -->
	<section class="section">
		<GrowthLoopDiagram report={$currentReport} inputs={$inputs} />
	</section>

	<!-- 2. Weekly Scorecard -->
	<section class="section">
		<div class="stat-grid">
			<StatCard
				label="Follower Growth"
				value={$currentReport?.follower_delta ?? 0 > 0 ? `+${$currentReport?.follower_delta}` : `${$currentReport?.follower_delta ?? 0}`}
				change={$currentReport?.follower_delta ?? null}
			>
				{#snippet icon()}<Users size={20} />{/snippet}
			</StatCard>
			<StatCard
				label="Acceptance Rate"
				value={`${(($currentReport?.reply_acceptance_rate ?? 0) * 100).toFixed(0)}%`}
			>
				{#snippet icon()}<Percent size={20} />{/snippet}
			</StatCard>
			<StatCard
				label="Avg Engagement"
				value={avgScore().toFixed(1)}
			>
				{#snippet icon()}<TrendingUp size={20} />{/snippet}
			</StatCard>
			<StatCard
				label="Follow Conversion"
				value={`${(($currentReport?.estimated_follow_conversion ?? 0) * 100).toFixed(1)}%`}
			>
				{#snippet icon()}<BarChart3 size={20} />{/snippet}
			</StatCard>
			<StatCard
				label="Output Volume"
				value={totalOutput}
			>
				{#snippet icon()}<MessageSquare size={20} />{/snippet}
			</StatCard>
			<StatCard
				label="Top Topic"
				value={topTopicName}
			>
				{#snippet icon()}<Star size={20} />{/snippet}
			</StatCard>
		</div>
	</section>

	<!-- 3. Recommendations -->
	<section class="section">
		<h2 class="section-title">Recommendations</h2>
		{#if $recommendations.length === 0}
			<div class="empty-section">
				<span>No recommendations yet — keep posting and check back next week.</span>
			</div>
		{:else}
			<div class="rec-grid">
				{#each $recommendations as rec}
					<RecommendationCard recommendation={rec} />
				{/each}
			</div>
		{/if}
	</section>

	<!-- 4. Historical Trend -->
	<section class="section">
		<WeeklyTrendChart reports={$reportHistory} />
	</section>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.header-left {
		display: flex;
		flex-direction: column;
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

	.refresh-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background-color: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition:
			background-color 0.15s,
			color 0.15s;
	}

	.refresh-btn:hover:not(:disabled) {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}

	.refresh-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
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

	.section {
		margin-bottom: 24px;
	}

	.section-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 12px;
	}

	.stat-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 12px;
	}

	.rec-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 12px;
	}

	.empty-section {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		color: var(--color-text-subtle);
		font-size: 13px;
	}

	.skeleton-card {
		height: 80px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-card.tall {
		height: 120px;
		margin-bottom: 24px;
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
