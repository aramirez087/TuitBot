<script lang="ts">
	import { onMount } from 'svelte';
	import { BarChart3, Loader2 } from 'lucide-svelte';
	import { loadDashboard, summary, followerSnapshots, recentPerformance, loading, error } from '$lib/stores/analytics';
	import EngagementChart from './charts/EngagementChart.svelte';
	import FollowerGrowthChart from './charts/FollowerGrowthChart.svelte';
	import ReachChart from './charts/ReachChart.svelte';
	import BestTimeHeatmap from './charts/BestTimeHeatmap.svelte';

	onMount(() => {
		loadDashboard(30);
	});
</script>

<div class="analytics-dashboard">
	<header class="dashboard-header">
		<h1>Analytics</h1>
		<p class="subtitle">Content performance & engagement metrics</p>
	</header>

	{#if $loading}
		<div class="loading-state">
			<Loader2 size={32} style="animation: spin 1s linear infinite;" />
			<p>Loading analytics...</p>
		</div>
	{:else if $error}
		<div class="error-state">
			<p class="error-message">{$error}</p>
			<button class="retry-btn" onclick={() => loadDashboard(30)}>Retry</button>
		</div>
	{:else}
		<!-- Summary cards -->
		{#if $summary}
			<section class="summary-cards">
				<div class="card stat-card">
					<div class="stat-label">Followers</div>
					<div class="stat-value">{$summary.followers.current.toLocaleString()}</div>
					<div class="stat-change">
						{#if $summary.followers.change_7d >= 0}
							<span class="positive">+{$summary.followers.change_7d}</span> this week
						{:else}
							<span class="negative">{$summary.followers.change_7d}</span> this week
						{/if}
					</div>
				</div>

				<div class="card stat-card">
					<div class="stat-label">Tweets Posted</div>
					<div class="stat-value">{$summary.actions_today.tweets}</div>
					<div class="stat-change">today</div>
				</div>

				<div class="card stat-card">
					<div class="stat-label">Replies</div>
					<div class="stat-value">{$summary.actions_today.replies}</div>
					<div class="stat-change">today</div>
				</div>

				<div class="card stat-card">
					<div class="stat-label">Avg Engagement</div>
					<div class="stat-value">
						{(($summary.engagement.avg_tweet_score + $summary.engagement.avg_reply_score) / 2).toFixed(1)}
					</div>
					<div class="stat-change">score</div>
				</div>
			</section>
		{/if}

		<!-- Charts grid -->
		<section class="charts-grid">
			<div class="chart-container">
				<h2 class="chart-title">Engagement Over Time</h2>
				<EngagementChart items={$recentPerformance} />
			</div>

			<div class="chart-container">
				<h2 class="chart-title">Follower Growth (30 days)</h2>
				<FollowerGrowthChart snapshots={$followerSnapshots} />
			</div>

			<div class="chart-container">
				<h2 class="chart-title">Reach & Impact</h2>
				<ReachChart items={$recentPerformance} />
			</div>

			<div class="chart-container">
				<h2 class="chart-title">Best Time to Post</h2>
				<BestTimeHeatmap items={$recentPerformance} />
			</div>
		</section>
	{/if}
</div>

<style>
	.analytics-dashboard {
		width: 100%;
		max-width: 1400px;
		margin: 0 auto;
	}

	.dashboard-header {
		margin-bottom: 32px;
	}

	.dashboard-header h1 {
		font-size: 28px;
		font-weight: 600;
		margin: 0 0 8px 0;
		color: var(--color-text);
	}

	.subtitle {
		font-size: 14px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.loading-state,
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 300px;
		gap: 16px;
		color: var(--color-text-muted);
	}

	.loading-state p,
	.error-state p {
		margin: 0;
		font-size: 14px;
	}

	.error-message {
		color: rgb(220, 38, 38);
		font-weight: 500;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.retry-btn {
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background-color: var(--color-surface-hover);
		color: var(--color-text);
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		transition: background-color 0.15s ease;
	}

	.retry-btn:hover {
		background-color: var(--color-surface-active);
	}

	.summary-cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 16px;
		margin-bottom: 32px;
	}

	.card {
		padding: 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background-color: var(--color-surface);
	}

	.stat-card {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.stat-label {
		font-size: 12px;
		color: var(--color-text-muted);
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.stat-value {
		font-size: 24px;
		font-weight: 600;
		color: var(--color-text);
	}

	.stat-change {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.positive {
		color: rgb(34, 197, 94);
		font-weight: 500;
	}

	.negative {
		color: rgb(239, 68, 68);
		font-weight: 500;
	}

	.charts-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(600px, 1fr));
		gap: 24px;
		margin-bottom: 32px;
	}

	.chart-container {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.chart-title {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	@media (max-width: 768px) {
		.analytics-dashboard {
			padding: 0;
		}

		.summary-cards {
			grid-template-columns: 1fr 1fr;
		}

		.charts-grid {
			grid-template-columns: 1fr;
		}

		.dashboard-header h1 {
			font-size: 24px;
		}
	}
</style>
