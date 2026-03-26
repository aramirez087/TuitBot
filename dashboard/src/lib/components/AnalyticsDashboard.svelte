<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Loader2, TrendingUp, TrendingDown, MessageSquare, Repeat2, BarChart2, Users, RefreshCw } from 'lucide-svelte';
	import { loadDashboard, summary, followerSnapshots, recentPerformance, loading, error, startAutoRefresh, stopAutoRefresh } from '$lib/stores/analytics';
	import EngagementChart from './charts/EngagementChart.svelte';
	import FollowerGrowthChart from './charts/FollowerGrowthChart.svelte';
	import ReachChart from './charts/ReachChart.svelte';
	import BestTimeHeatmap from './charts/BestTimeHeatmap.svelte';

	onMount(() => {
		loadDashboard(30);
		startAutoRefresh(60_000);
	});

	onDestroy(() => {
		stopAutoRefresh();
	});

	const hasActivity = $derived(
		($summary?.actions_today.tweets ?? 0) > 0 ||
		($summary?.actions_today.replies ?? 0) > 0 ||
		($summary?.engagement.total_replies_sent ?? 0) > 0 ||
		($summary?.engagement.total_tweets_posted ?? 0) > 0
	);

	const hasFollowerData = $derived(($followerSnapshots?.length ?? 0) > 0);

	function followerDelta(n: number): string {
		if (n === 0) return '—';
		return n > 0 ? `+${n}` : `${n}`;
	}
</script>

<div class="analytics-page">
	<!-- Header -->
	<div class="page-header">
		<div class="header-left">
			<h1>Analytics</h1>
			<p class="subtitle">Content performance & engagement metrics</p>
		</div>
		{#if !$loading}
			<button class="refresh-btn" onclick={() => loadDashboard(30)} title="Refresh">
				<RefreshCw size={14} />
				Refresh
			</button>
		{/if}
	</div>

	{#if $loading}
		<div class="loading-state">
			<Loader2 size={28} class="spin" />
			<span>Loading analytics…</span>
		</div>
	{:else if $error}
		<div class="error-state">
			<p class="error-msg">{$error}</p>
			<button class="retry-btn" onclick={() => loadDashboard(30)}>Retry</button>
		</div>
	{:else}
		<!-- Stat cards — always shown, with skeleton zeros if no data -->
		<div class="stat-grid">
			<div class="stat-card">
				<div class="stat-label">
					<Users size={13} />
					Followers
				</div>
				<div class="stat-value">{($summary?.followers.current ?? 0).toLocaleString()}</div>
				<div class="stat-delta" class:positive={($summary?.followers.change_7d ?? 0) > 0} class:negative={($summary?.followers.change_7d ?? 0) < 0}>
					{#if ($summary?.followers.change_7d ?? 0) > 0}
						<TrendingUp size={11} />
					{:else if ($summary?.followers.change_7d ?? 0) < 0}
						<TrendingDown size={11} />
					{/if}
					{followerDelta($summary?.followers.change_7d ?? 0)} this week
				</div>
			</div>

			<div class="stat-card">
				<div class="stat-label">
					<BarChart2 size={13} />
					Tweets Today
				</div>
				<div class="stat-value">{$summary?.actions_today.tweets ?? 0}</div>
				<div class="stat-meta">{($summary?.engagement.total_tweets_posted ?? 0).toLocaleString()} total posted</div>
			</div>

			<div class="stat-card">
				<div class="stat-label">
					<MessageSquare size={13} />
					Replies Today
				</div>
				<div class="stat-value">{$summary?.actions_today.replies ?? 0}</div>
				<div class="stat-meta">{($summary?.engagement.total_replies_sent ?? 0).toLocaleString()} total sent</div>
			</div>

			<div class="stat-card">
				<div class="stat-label">
					<Repeat2 size={13} />
					Avg Engagement
				</div>
				<div class="stat-value">
					{(($summary?.engagement.avg_tweet_score ?? 0) + ($summary?.engagement.avg_reply_score ?? 0) > 0
						? (($summary!.engagement.avg_tweet_score + $summary!.engagement.avg_reply_score) / 2)
						: 0
					).toFixed(1)}
				</div>
				<div class="stat-meta">score</div>
			</div>
		</div>

		{#if !hasActivity && !hasFollowerData}
			<div class="empty-notice">
				<BarChart2 size={20} />
				<div class="empty-notice-text">
					<strong>No activity data yet</strong>
					<span>Analytics populate as your automation runs. Follower snapshots are recorded when the bot syncs your profile.</span>
				</div>
			</div>
		{/if}

		<!-- Charts -->
		<div class="chart-grid">
			<div class="chart-section">
				<h2 class="chart-title">Engagement Over Time</h2>
				<EngagementChart items={$recentPerformance} />
			</div>

			<div class="chart-section">
				<h2 class="chart-title">Follower Growth (30 days)</h2>
				<FollowerGrowthChart snapshots={$followerSnapshots} />
			</div>

			<div class="chart-section">
				<h2 class="chart-title">Reach & Impact</h2>
				<ReachChart items={$recentPerformance} />
			</div>

			<div class="chart-section">
				<h2 class="chart-title">Best Time to Post</h2>
				<BestTimeHeatmap items={$recentPerformance} />
			</div>
		</div>
	{/if}
</div>

<style>
	.analytics-page {
		width: 100%;
		max-width: 1200px;
	}

	/* Header */
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 24px;
		gap: 16px;
	}

	h1 {
		font-size: 22px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
		letter-spacing: -0.01em;
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
		padding: 6px 12px;
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-muted);
		background: transparent;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s, background 0.15s;
		white-space: nowrap;
	}

	.refresh-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
		background: var(--color-surface-hover);
	}

	/* Loading / error */
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 280px;
		gap: 12px;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.loading-state :global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 280px;
		gap: 12px;
	}

	.error-msg {
		font-size: 13px;
		color: var(--color-danger);
		margin: 0;
	}

	.retry-btn {
		padding: 7px 16px;
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		cursor: pointer;
		transition: background 0.15s;
	}

	.retry-btn:hover {
		background: var(--color-surface-hover);
	}

	/* Stat grid */
	.stat-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
		margin-bottom: 20px;
	}

	@media (max-width: 900px) {
		.stat-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	@media (max-width: 560px) {
		.stat-grid {
			grid-template-columns: 1fr;
		}
	}

	.stat-card {
		padding: 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		transition: border-color 0.15s;
	}

	.stat-card:hover {
		border-color: var(--color-border);
	}

	.stat-label {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
	}

	.stat-value {
		font-size: 26px;
		font-weight: 700;
		color: var(--color-text);
		letter-spacing: -0.02em;
		line-height: 1;
	}

	.stat-delta {
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.stat-delta.positive {
		color: var(--color-success);
	}

	.stat-delta.negative {
		color: var(--color-danger);
	}

	.stat-meta {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	/* Empty notice */
	.empty-notice {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px 16px;
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 8px;
		color: var(--color-text-muted);
		margin-bottom: 20px;
	}

	.empty-notice-text {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-size: 13px;
	}

	.empty-notice-text strong {
		color: var(--color-text);
		font-weight: 600;
	}

	.empty-notice-text span {
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	/* Charts */
	.chart-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 20px;
	}

	@media (max-width: 900px) {
		.chart-grid {
			grid-template-columns: 1fr;
		}
	}

	.chart-section {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.chart-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
		letter-spacing: -0.01em;
	}
</style>
