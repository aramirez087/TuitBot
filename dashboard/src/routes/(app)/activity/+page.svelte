<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { ChevronDown } from 'lucide-svelte';
	import RateLimitBar from '$lib/components/RateLimitBar.svelte';
	import ActivityFilter from '$lib/components/ActivityFilter.svelte';
	import ActivityItem from '$lib/components/ActivityItem.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import {
		actions,
		rateLimits,
		loading,
		error,
		totalCount,
		hasMore,
		selectedFilter,
		loadActivity,
		loadMore,
		setFilter,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/activity';

	let loadingMore = $state(false);

	async function handleLoadMore() {
		loadingMore = true;
		await loadMore();
		loadingMore = false;
	}

	onMount(() => {
		loadActivity(true);
		startAutoRefresh();
	});

	onDestroy(() => {
		stopAutoRefresh();
	});
</script>

<svelte:head>
	<title>Activity — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>Activity</h1>
	<p class="subtitle">Real-time feed of automation actions</p>
</div>

{#if $error && $actions.length === 0}
	<ErrorState message={$error} onretry={() => loadActivity(true)} />
{:else if $error}
	<div class="error-banner">
		<span>{$error}</span>
		<button onclick={() => loadActivity(true)}>Retry</button>
	</div>
{/if}

<div class="rate-limits-section">
	<h2>Daily Limits</h2>
	{#if $rateLimits}
		<div class="rate-limits-grid">
			<RateLimitBar label="Replies" usage={$rateLimits.replies} />
			<RateLimitBar label="Tweets" usage={$rateLimits.tweets} />
			<RateLimitBar label="Threads" usage={$rateLimits.threads} />
		</div>
	{:else if !$loading}
		<p class="muted">Rate limits not available.</p>
	{:else}
		<div class="skeleton-bar"></div>
	{/if}
</div>

<div class="activity-section">
	<div class="activity-header">
		<h2>Activity Feed</h2>
		<ActivityFilter selected={$selectedFilter} onselect={setFilter} />
	</div>

	{#if $loading && $actions.length === 0}
		<div class="feed-container">
			{#each { length: 5 } as _}
				<div class="skeleton-item"></div>
			{/each}
		</div>
	{:else if $actions.length === 0}
		<div class="feed-container">
			<EmptyState
				title="No activity recorded yet"
				description="Actions will appear here as the automation runs."
			/>
		</div>
	{:else}
		<div class="feed-container">
			{#each $actions as action (action.id)}
				<ActivityItem {action} />
			{/each}
		</div>

		{#if $hasMore}
			<div class="load-more">
				<button class="load-more-btn" disabled={loadingMore} onclick={handleLoadMore}>
					{#if loadingMore}
						Loading...
					{:else}
						<ChevronDown size={16} />
						Load more ({$totalCount - $actions.length} remaining)
					{/if}
				</button>
			</div>
		{/if}
	{/if}
</div>

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

	h2 {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.muted {
		font-size: 13px;
		color: var(--color-text-subtle);
		margin: 0;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
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
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.rate-limits-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 20px;
		margin-bottom: 24px;
	}

	.rate-limits-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 20px;
		margin-top: 12px;
	}

	.activity-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 20px;
	}

	.activity-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
		flex-wrap: wrap;
		gap: 12px;
	}

	.feed-container {
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background-color: var(--color-base);
		overflow: hidden;
	}


	.load-more {
		display: flex;
		justify-content: center;
		margin-top: 16px;
	}

	.load-more-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background-color: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.load-more-btn:hover:not(:disabled) {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
		border-color: var(--color-accent);
	}

	.load-more-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.skeleton-bar {
		height: 40px;
		margin-top: 12px;
		background-color: var(--color-surface-active);
		border-radius: 6px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-item {
		height: 56px;
		border-bottom: 1px solid var(--color-border-subtle);
		background-color: var(--color-surface-active);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-item:last-child {
		border-bottom: none;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.4;
		}
	}

	@media (max-width: 640px) {
		.rate-limits-grid {
			grid-template-columns: 1fr;
		}

		.activity-header {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
