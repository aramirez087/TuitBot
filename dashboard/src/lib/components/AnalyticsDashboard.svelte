<script lang="ts">
	import { onMount } from 'svelte';
	import { Loader2 } from 'lucide-svelte';
	import { loadDashboard, summary, followerSnapshots, recentPerformance, loading, error } from '$lib/stores/analytics';
	import EngagementChart from './charts/EngagementChart.svelte';
	import FollowerGrowthChart from './charts/FollowerGrowthChart.svelte';
	import ReachChart from './charts/ReachChart.svelte';
	import BestTimeHeatmap from './charts/BestTimeHeatmap.svelte';

	onMount(() => {
		loadDashboard(30);
	});
</script>

<div class="w-full max-w-6xl mx-auto">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-semibold mb-2 text-gray-900">Analytics</h1>
		<p class="text-sm text-gray-600">Content performance & engagement metrics</p>
	</div>

	{#if $loading}
		<div class="flex flex-col items-center justify-center min-h-80 gap-4 text-gray-500">
			<Loader2 size={32} style="animation: spin 1s linear infinite;" />
			<p>Loading analytics...</p>
		</div>
	{:else if $error}
		<div class="flex flex-col items-center justify-center min-h-80 gap-4">
			<p class="text-red-600 font-medium">{$error}</p>
			<button
				class="px-4 py-2 border border-gray-300 rounded-md bg-gray-100 hover:bg-gray-200 text-gray-900 text-sm font-medium transition-colors"
				onclick={() => loadDashboard(30)}
			>
				Retry
			</button>
		</div>
	{:else}
		<!-- Summary cards -->
		{#if $summary}
			<div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
				<div class="p-4 border border-gray-200 rounded-lg bg-white">
					<div class="text-xs font-semibold uppercase text-gray-600 tracking-wide">Followers</div>
					<div class="text-2xl font-bold text-gray-900 mt-2">{$summary.followers.current.toLocaleString()}</div>
					<div class="text-xs text-gray-600 mt-2">
						{#if $summary.followers.change_7d >= 0}
							<span class="text-green-600 font-medium">+{$summary.followers.change_7d}</span> this week
						{:else}
							<span class="text-red-600 font-medium">{$summary.followers.change_7d}</span> this week
						{/if}
					</div>
				</div>

				<div class="p-4 border border-gray-200 rounded-lg bg-white">
					<div class="text-xs font-semibold uppercase text-gray-600 tracking-wide">Tweets Posted</div>
					<div class="text-2xl font-bold text-gray-900 mt-2">{$summary.actions_today.tweets}</div>
					<div class="text-xs text-gray-600 mt-2">today</div>
				</div>

				<div class="p-4 border border-gray-200 rounded-lg bg-white">
					<div class="text-xs font-semibold uppercase text-gray-600 tracking-wide">Replies</div>
					<div class="text-2xl font-bold text-gray-900 mt-2">{$summary.actions_today.replies}</div>
					<div class="text-xs text-gray-600 mt-2">today</div>
				</div>

				<div class="p-4 border border-gray-200 rounded-lg bg-white">
					<div class="text-xs font-semibold uppercase text-gray-600 tracking-wide">Avg Engagement</div>
					<div class="text-2xl font-bold text-gray-900 mt-2">
						{(($summary.engagement.avg_tweet_score + $summary.engagement.avg_reply_score) / 2).toFixed(1)}
					</div>
					<div class="text-xs text-gray-600 mt-2">score</div>
				</div>
			</div>
		{/if}

		<!-- Charts grid -->
		<div class="grid lg:grid-cols-2 gap-6 mb-8">
			<div>
				<h2 class="text-lg font-semibold mb-3 text-gray-900">Engagement Over Time</h2>
				<EngagementChart items={$recentPerformance} />
			</div>

			<div>
				<h2 class="text-lg font-semibold mb-3 text-gray-900">Follower Growth (30 days)</h2>
				<FollowerGrowthChart snapshots={$followerSnapshots} />
			</div>

			<div>
				<h2 class="text-lg font-semibold mb-3 text-gray-900">Reach & Impact</h2>
				<ReachChart items={$recentPerformance} />
			</div>

			<div>
				<h2 class="text-lg font-semibold mb-3 text-gray-900">Best Time to Post</h2>
				<BestTimeHeatmap items={$recentPerformance} />
			</div>
		</div>
	{/if}
</div>

<style>
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
